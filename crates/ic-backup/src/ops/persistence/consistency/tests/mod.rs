//! Retained original guarantees, exact local recovery and unsafe/bounded evidence denial.

use super::*;
use crate::{
    model::consistency::ConsistencyGuaranteeRecord,
    test_support::{membership::plan, temp_dir},
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
};

fn layout() -> (std::path::PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-consistency-requirement");
    fs::create_dir(&root).unwrap();
    let guard = BackupLayoutGuard::acquire(&root).unwrap();
    (root, guard)
}
#[test]
fn original_guarantee_reconciles_exact_create_and_cannot_be_downgraded_or_replaced() {
    let (root, layout) = layout();
    let plan = plan();
    let original = ConsistencyRequirementRecord::new(
        &plan,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    );
    assert!(matches!(
        create_consistency_requirement(&layout, &plan, &original),
        Err(ConsistencyPersistenceError::Plan(_))
    ));
    assert!(!root.join("consistency-requirement.json").exists());
    super::super::create_operation_plan(&layout, &plan).unwrap();
    create_consistency_requirement(&layout, &plan, &original).unwrap();
    let path = root.join("consistency-requirement.json");
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        read_consistency_requirement(&layout, &plan, &original.digest()).unwrap(),
        original
    );
    let weaker = ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    assert!(
        matches!(create_consistency_requirement(&layout,&plan,&weaker),Err(ConsistencyPersistenceError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref e, .. }))) if e.kind()==io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        read_consistency_requirement(&layout, &plan, &weaker.digest()),
        Err(ConsistencyPersistenceError::DigestMismatch)
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let lock = JournalLock::acquire(&path).unwrap();
    assert!(matches!(
        read_consistency_requirement(&layout, &plan, &original.digest()),
        Err(ConsistencyPersistenceError::Lock(
            JournalLockError::Locked { .. }
        ))
    ));
    drop(lock);
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    assert_eq!(
        read_consistency_requirement(&layout, &plan, &original.digest()).unwrap(),
        original
    );
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn bounded_invalid_records_and_other_plan_preserve_exact_rejected_bytes() {
    let (root, layout) = layout();
    let plan = plan();
    super::super::create_operation_plan(&layout, &plan).unwrap();
    let original =
        ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    let path = root.join("consistency-requirement.json");
    for bytes in [
        vec![b' '; usize::try_from(MAX_CONSISTENCY_REQUIREMENT_BYTES).unwrap() + 1],
        br#"{"version":1,"plan_intent":"invalid","guarantee":"per_canister"}"#.to_vec(),
    ] {
        fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            read_consistency_requirement(&layout, &plan, &original.digest()),
            Err(ConsistencyPersistenceError::Persistence(
                PersistenceError::RecordTooLarge { .. } | PersistenceError::Json(_)
            ))
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    let mut other = serde_json::to_value(&plan).unwrap();
    other["context"]["release"] = serde_json::json!("90".repeat(32));
    let other = serde_json::from_value(other).unwrap();
    let different =
        ConsistencyRequirementRecord::new(&other, ConsistencyGuaranteeRecord::PerCanister);
    fs::write(&path, serde_json::to_vec(&different).unwrap()).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert!(matches!(
        read_consistency_requirement(&layout, &plan, &different.digest()),
        Err(ConsistencyPersistenceError::Requirement(
            ConsistencyRequirementError::PlanMismatch
        ))
    ));
    assert!(matches!(
        create_consistency_requirement(&layout, &plan, &different),
        Err(ConsistencyPersistenceError::Requirement(
            ConsistencyRequirementError::PlanMismatch
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn symlink_and_replaced_layout_reject_without_repair_or_new_guarantee() {
    let (root, layout) = layout();
    let plan = plan();
    super::super::create_operation_plan(&layout, &plan).unwrap();
    let original =
        ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    create_consistency_requirement(&layout, &plan, &original).unwrap();
    let path = root.join("consistency-requirement.json");
    let retained = root.join("retained.json");
    fs::rename(&path, &retained).unwrap();
    symlink(&retained, &path).unwrap();
    assert!(read_consistency_requirement(&layout, &plan, &original.digest()).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let old = root.with_extension("retained");
    fs::rename(&root, &old).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(read_consistency_requirement(&layout, &plan, &original.digest()).is_err());
    assert!(create_consistency_requirement(&layout, &plan, &original).is_err());
    assert!(!path.exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(old).unwrap();
}
