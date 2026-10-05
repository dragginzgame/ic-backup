//! Exact immutable original source/safety recovery under both native layouts.

use super::*;
use crate::{
    model::restore_safety::RestoreSafetyLaneRecord,
    test_support::{
        restore_safety::{plan, requirement, source},
        temp_dir,
    },
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
};

fn layouts() -> (std::path::PathBuf, BackupLayoutGuard, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-restore-safety");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("restore")).unwrap();
    fs::create_dir(root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    (root, layout, source)
}
#[test]
fn requires_both_retained_original_plans_and_reconciles_creation_without_replacement() {
    let (root, layout, source_layout) = layouts();
    let plan = plan();
    let source = source();
    let original = requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
    assert!(matches!(
        create_restore_safety_requirement(&layout, &source_layout, &plan, &source, &original),
        Err(RestoreSafetyPersistenceError::Plan(_))
    ));
    super::super::create_operation_plan(&layout, &plan).unwrap();
    assert!(matches!(
        create_restore_safety_requirement(&layout, &source_layout, &plan, &source, &original),
        Err(RestoreSafetyPersistenceError::Plan(_))
    ));
    assert!(
        !layout
            .root()
            .join("restore-safety-requirement.json")
            .exists()
    );
    super::super::create_operation_plan(&source_layout, &source).unwrap();
    create_restore_safety_requirement(&layout, &source_layout, &plan, &source, &original).unwrap();
    let path = layout.root().join("restore-safety-requirement.json");
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        read_restore_safety_requirement(
            &layout,
            &source_layout,
            &plan,
            &source,
            &original.digest()
        )
        .unwrap(),
        original
    );
    let weaker = requirement(
        &plan,
        &source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
    assert!(
        matches!(create_restore_safety_requirement(&layout,&source_layout,&plan,&source,&weaker),Err(RestoreSafetyPersistenceError::Persistence(PersistenceError::Io(ref e))) if e.kind()==io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        read_restore_safety_requirement(&layout, &source_layout, &plan, &source, &weaker.digest()),
        Err(RestoreSafetyPersistenceError::DigestMismatch)
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let lock = JournalLock::acquire(&path).unwrap();
    assert!(matches!(
        read_restore_safety_requirement(
            &layout,
            &source_layout,
            &plan,
            &source,
            &original.digest()
        ),
        Err(RestoreSafetyPersistenceError::Lock(
            JournalLockError::Locked { .. }
        ))
    ));
    drop(lock);
    drop(layout);
    drop(source_layout);
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    assert_eq!(
        read_restore_safety_requirement(
            &layout,
            &source_layout,
            &plan,
            &source,
            &original.digest()
        )
        .unwrap(),
        original
    );
    drop(layout);
    drop(source_layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn oversized_malformed_and_rebound_requirements_preserve_exact_rejected_bytes() {
    let (root, layout, source_layout) = layouts();
    let plan = plan();
    let source = source();
    let original = requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
    super::super::create_operation_plan(&layout, &plan).unwrap();
    super::super::create_operation_plan(&source_layout, &source).unwrap();
    let path = layout.root().join("restore-safety-requirement.json");
    for bytes in [
        vec![b' '; usize::try_from(MAX_RESTORE_SAFETY_REQUIREMENT_BYTES).unwrap() + 1],
        br#"{"version":1,"plan_intent":"invalid"}"#.to_vec(),
    ] {
        fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            read_restore_safety_requirement(
                &layout,
                &source_layout,
                &plan,
                &source,
                &original.digest()
            ),
            Err(RestoreSafetyPersistenceError::Persistence(
                PersistenceError::RecordTooLarge { .. } | PersistenceError::Json(_)
            ))
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    for field in ["plan_intent", "source_plan_intent"] {
        let mut value = serde_json::to_value(&original).unwrap();
        value[field] = serde_json::json!("34".repeat(32));
        let changed: RestoreSafetyRequirementRecord = serde_json::from_value(value).unwrap();
        let bytes = serde_json::to_vec(&changed).unwrap();
        fs::write(&path, &bytes).unwrap();
        let error = read_restore_safety_requirement(
            &layout,
            &source_layout,
            &plan,
            &source,
            &changed.digest(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            RestoreSafetyPersistenceError::Requirement(
                RestoreSafetyRequirementError::PlanMismatch
                    | RestoreSafetyRequirementError::SourcePlanMismatch
            )
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    drop(layout);
    drop(source_layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn rejects_symlinks_and_replaced_restore_or_source_roots_without_repair() {
    for replace_source in [false, true] {
        let (root, layout, source_layout) = layouts();
        let plan = plan();
        let source = source();
        let original = requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
        super::super::create_operation_plan(&layout, &plan).unwrap();
        super::super::create_operation_plan(&source_layout, &source).unwrap();
        create_restore_safety_requirement(&layout, &source_layout, &plan, &source, &original)
            .unwrap();
        let path = layout.root().join("restore-safety-requirement.json");
        let retained = layout.root().join("retained.json");
        fs::rename(&path, &retained).unwrap();
        symlink(&retained, &path).unwrap();
        assert!(
            read_restore_safety_requirement(
                &layout,
                &source_layout,
                &plan,
                &source,
                &original.digest()
            )
            .is_err()
        );
        assert!(
            fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        fs::remove_file(&path).unwrap();
        fs::rename(&retained, &path).unwrap();
        let selected = if replace_source {
            source_layout.root()
        } else {
            layout.root()
        };
        fs::rename(selected, root.join("held-original")).unwrap();
        fs::create_dir(selected).unwrap();
        assert!(
            read_restore_safety_requirement(
                &layout,
                &source_layout,
                &plan,
                &source,
                &original.digest()
            )
            .is_err()
        );
        assert!(
            create_restore_safety_requirement(&layout, &source_layout, &plan, &source, &original)
                .is_err()
        );
        assert!(!selected.join("restore-safety-requirement.json").exists());
        drop(layout);
        drop(source_layout);
        fs::remove_dir_all(root).unwrap();
    }
}
