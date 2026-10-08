use super::*;
use crate::{
    model::consistency::ConsistencyGuaranteeRecord,
    test_support::{
        membership::{hash, plan},
        restore_safety, temp_dir,
    },
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
};

fn layout() -> (PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-fence-obligation");
    fs::create_dir(&root).unwrap();
    let guard = BackupLayoutGuard::acquire(&root).unwrap();
    (root, guard)
}
fn fence() -> ApplicationFenceBinding {
    ApplicationFenceBinding {
        identity: hash("56"),
        membership_revision: hash("78"),
    }
}
#[test]
fn immutable_original_obligation_requires_retained_plan_and_requirement() {
    let (root, layout) = layout();
    let plan = plan();
    let requirement = ConsistencyRequirementRecord::new(
        &plan,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    );
    let fence = fence();
    let input = FenceObligationRequirement::Capture {
        requirement: &requirement,
        fence: &fence,
    };
    let record = FenceObligationRecord::for_capture(&plan, &requirement, 0, &fence).unwrap();
    assert!(matches!(
        create_fence_obligation(&layout, &plan, input, &record),
        Err(FenceObligationPersistenceError::Consistency(_))
    ));
    super::super::create_operation_plan(&layout, &plan).unwrap();
    assert!(create_fence_obligation(&layout, &plan, input, &record).is_err());
    assert!(!root.join("fence-obligation.json").exists());
    super::super::create_consistency_requirement(&layout, &plan, &requirement).unwrap();
    create_fence_obligation(&layout, &plan, input, &record).unwrap();
    let path = root.join("fence-obligation.json");
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        read_fence_obligation(&layout, &plan, input, &record.digest()).unwrap(),
        record
    );
    assert!(
        matches!(create_fence_obligation(&layout, &plan, input, &record), Err(FenceObligationPersistenceError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }))) if error.kind() == io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        read_fence_obligation(&layout, &plan, input, &hash("90")),
        Err(FenceObligationPersistenceError::DigestMismatch)
    ));
    let lock = JournalLock::acquire(&path).unwrap();
    assert!(matches!(
        read_fence_obligation(&layout, &plan, input, &record.digest()),
        Err(FenceObligationPersistenceError::Lock(
            JournalLockError::Locked { .. }
        ))
    ));
    drop(lock);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    assert_eq!(
        read_fence_obligation(&layout, &plan, input, &record.digest()).unwrap(),
        record
    );
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn oversized_invalid_and_rebound_fence_bytes_are_rejected_without_repair() {
    let (root, layout) = layout();
    let plan = plan();
    let requirement = ConsistencyRequirementRecord::new(
        &plan,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    );
    let fence = fence();
    let input = FenceObligationRequirement::Capture {
        requirement: &requirement,
        fence: &fence,
    };
    let record = FenceObligationRecord::for_capture(&plan, &requirement, 0, &fence).unwrap();
    super::super::create_operation_plan(&layout, &plan).unwrap();
    super::super::create_consistency_requirement(&layout, &plan, &requirement).unwrap();
    let path = root.join("fence-obligation.json");
    for bytes in [
        vec![b' '; usize::try_from(MAX_FENCE_OBLIGATION_BYTES).unwrap() + 1],
        b"{}".to_vec(),
    ] {
        fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            read_fence_obligation(&layout, &plan, input, &record.digest()),
            Err(FenceObligationPersistenceError::Persistence(
                PersistenceError::RecordTooLarge { .. } | PersistenceError::Json(_)
            ))
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    let rebound = FenceObligationRecord::for_capture(
        &plan,
        &requirement,
        0,
        &ApplicationFenceBinding {
            identity: hash("90"),
            ..fence.clone()
        },
    )
    .unwrap();
    fs::write(&path, serde_json::to_vec(&rebound).unwrap()).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert!(matches!(
        read_fence_obligation(&layout, &plan, input, &rebound.digest()),
        Err(FenceObligationPersistenceError::Obligation(
            FenceObligationError::BindingMismatch
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn symlink_and_replaced_layout_preserve_original_obligation() {
    let (root, layout) = layout();
    let plan = plan();
    let requirement = ConsistencyRequirementRecord::new(
        &plan,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    );
    let fence = fence();
    let input = FenceObligationRequirement::Capture {
        requirement: &requirement,
        fence: &fence,
    };
    let record = FenceObligationRecord::for_capture(&plan, &requirement, 0, &fence).unwrap();
    super::super::create_operation_plan(&layout, &plan).unwrap();
    super::super::create_consistency_requirement(&layout, &plan, &requirement).unwrap();
    create_fence_obligation(&layout, &plan, input, &record).unwrap();
    let path = root.join("fence-obligation.json");
    let retained = root.join("retained.json");
    fs::rename(&path, &retained).unwrap();
    symlink(&retained, &path).unwrap();
    assert!(read_fence_obligation(&layout, &plan, input, &record.digest()).is_err());
    let old = root.with_extension("retained");
    fs::rename(&root, &old).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(read_fence_obligation(&layout, &plan, input, &record.digest()).is_err());
    assert!(create_fence_obligation(&layout, &plan, input, &record).is_err());
    assert!(!path.exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(old).unwrap();
}
#[test]
fn restoration_requires_both_original_plans_and_unchanged_source_layout() {
    let (root, layout) = layout();
    let (source_root, source_layout) = self::layout();
    let plan = plan();
    let source = restore_safety::source();
    let requirement = restore_safety::requirement(
        &plan,
        &source,
        crate::model::restore_safety::RestoreSafetyLaneRecord::ApplicationFenced,
    );
    let input = FenceObligationRequirement::Restore {
        source_layout: &source_layout,
        source: &source,
        requirement: &requirement,
    };
    let record = FenceObligationRecord::for_restore(&plan, &source, &requirement, 0).unwrap();
    super::super::create_operation_plan(&layout, &plan).unwrap();
    super::super::create_operation_plan(&source_layout, &source).unwrap();
    assert!(matches!(
        create_fence_obligation(&layout, &plan, input, &record),
        Err(FenceObligationPersistenceError::Restore(_))
    ));
    super::super::create_restore_safety_requirement(
        &layout,
        &source_layout,
        &plan,
        &source,
        &requirement,
    )
    .unwrap();
    create_fence_obligation(&layout, &plan, input, &record).unwrap();
    assert_eq!(
        read_fence_obligation(&layout, &plan, input, &record.digest()).unwrap(),
        record
    );
    let bytes = fs::read(root.join("fence-obligation.json")).unwrap();
    let old = source_root.with_extension("retained");
    fs::rename(&source_root, &old).unwrap();
    fs::create_dir(&source_root).unwrap();
    assert!(read_fence_obligation(&layout, &plan, input, &record.digest()).is_err());
    assert_eq!(fs::read(root.join("fence-obligation.json")).unwrap(), bytes);
    drop(layout);
    drop(source_layout);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(source_root).unwrap();
    fs::remove_dir_all(old).unwrap();
}
