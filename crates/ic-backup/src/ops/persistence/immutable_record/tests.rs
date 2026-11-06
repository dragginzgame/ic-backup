//! Shared immutable IO contract, exercised through its operation-plan caller.

use crate::{
    model::operation_plan::{OperationPlanRecord, PlanBudgetRecord},
    ops::persistence::{
        BackupLayoutGuard, JournalLock, JournalLockError, OperationPlanPersistenceError,
        PersistenceError, create_operation_plan, read_operation_plan,
    },
    tests::support::{
        operation_plan::{plan, request},
        temp_dir,
    },
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
};

fn layout() -> (std::path::PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-immutable-record");
    let guard = BackupLayoutGuard::acquire(&root).unwrap();
    (root, guard)
}

#[test]
fn immutable_publication_is_private_create_only_and_excluded_by_its_record_lock() {
    let (root, layout) = layout();
    let original = plan();
    let path = root.join("operation-plan.json");
    create_operation_plan(&layout, &original).expect("durable original plan");
    let bytes = fs::read(&path).expect("evidence");
    assert_eq!(
        fs::metadata(&path)
            .expect("private file")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        read_operation_plan(&layout, &original.digest()).expect("reconcile lost create response"),
        original
    );
    let mut input = request();
    input.budget = PlanBudgetRecord::new(4, 2).expect("changed ceiling");
    let changed = OperationPlanRecord::new(input).expect("different valid plan");
    assert!(matches!(
        read_operation_plan(&layout, &changed.digest()),
        Err(OperationPlanPersistenceError::DigestMismatch)
    ));
    assert!(
        matches!(create_operation_plan(&layout,&changed),Err(OperationPlanPersistenceError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(&path).expect("unchanged"), bytes);
    let lock = JournalLock::acquire(&path).expect("other cooperating plan owner");
    assert!(matches!(
        read_operation_plan(&layout, &original.digest()),
        Err(OperationPlanPersistenceError::Lock(
            JournalLockError::Locked { .. }
        ))
    ));
    drop(lock);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn immutable_reads_reject_changed_bindings_symlinks_and_replaced_roots() {
    let (root, layout) = layout();
    let original = plan();
    let path = root.join("operation-plan.json");
    create_operation_plan(&layout, &original).expect("plan");
    let mut bad = serde_json::to_value(&original).expect("json");
    bad["operations"][0]["target"] = serde_json::json!("aaaaa-aa");
    fs::write(&path, serde_json::to_vec(&bad).expect("encode")).expect("invalid binding");
    let evidence = fs::read(&path).expect("retain bytes");
    assert!(
        matches!(read_operation_plan(&layout,&original.digest()),Err(OperationPlanPersistenceError::Persistence(PersistenceError::Json(ref error))) if error.is_data())
    );
    assert_eq!(fs::read(&path).expect("unchanged"), evidence);
    let retained = root.join("retained.json");
    fs::rename(&path, &retained).expect("retain fixture");
    symlink(&retained, &path).expect("unsafe entry");
    assert!(read_operation_plan(&layout, &original.digest()).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .expect("retained link")
            .file_type()
            .is_symlink()
    );
    let old = root.with_extension("retained");
    fs::rename(&root, &old).expect("retain root");
    fs::create_dir(&root).expect("replacement");
    assert!(matches!(
        read_operation_plan(&layout, &original.digest()),
        Err(OperationPlanPersistenceError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert!(create_operation_plan(&layout, &original).is_err());
    assert!(!path.exists());
    drop(layout);
    fs::remove_dir_all(root).expect("clean replacement");
    fs::remove_dir_all(old).expect("clean successful retained fixture");
}
