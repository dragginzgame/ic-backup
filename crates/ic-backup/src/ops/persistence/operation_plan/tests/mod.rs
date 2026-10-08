//! Native original plan admission, retained evidence and independent metadata bounds.

use super::*;
use crate::{
    model::{
        attempt_journal::AttemptBudgetRecord,
        effect_graph::{
            EffectGraphRecord, EffectNodeRecord, EffectNodeRequest, MAX_EFFECT_OPERATIONS,
        },
        operation_plan::{
            PlanBudgetRecord, PlannedOperationRecord, PlannedOperationRequest, tests::request,
        },
    },
    test_support::temp_dir,
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
};
fn plan() -> OperationPlanRecord {
    OperationPlanRecord::new(request()).expect("plan")
}
fn layout() -> (std::path::PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-operation-plan");
    fs::create_dir(&root).expect("layout");
    let guard = BackupLayoutGuard::acquire(&root).expect("layout exclusion");
    (root, guard)
}

#[test]
fn immutable_plan_admits_original_intent_and_rejects_different_ceilings_without_replacement() {
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
fn maximum_model_count_and_independent_persistence_bounds_preserve_original_absence() {
    let (root, layout) = layout();
    let mut input = request();
    input.graph = EffectGraphRecord::new(
        (0..u64::try_from(MAX_EFFECT_OPERATIONS).expect("bound"))
            .map(|id| {
                EffectNodeRecord::new(EffectNodeRequest {
                    operation_sequence: id,
                    depends_on: vec![],
                })
                .expect("node")
            })
            .collect(),
    )
    .expect("maximum graph");
    input.operations = (0..u64::try_from(MAX_EFFECT_OPERATIONS).expect("bound"))
        .map(|id| {
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: id,
                target: input.selected_targets[0].clone(),
                request: "ef".repeat(32),
                budget: AttemptBudgetRecord::new(0, 0).expect("zero"),
            })
            .expect("operation")
        })
        .collect();
    input.budget = PlanBudgetRecord::new(0, 0).expect("zero ceilings");
    let excessive = OperationPlanRecord::new(input).expect("valid maximum table");
    assert_eq!(excessive.operations().len(), MAX_EFFECT_OPERATIONS);
    let compact = serde_json::to_vec(&excessive).expect("encode maximum");
    assert_eq!(
        serde_json::from_slice::<OperationPlanRecord>(&compact).expect("maximum decode"),
        excessive
    );
    assert!(matches!(
        create_operation_plan(&layout, &excessive),
        Err(OperationPlanPersistenceError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_OPERATION_PLAN_BYTES
            }
        ))
    ));
    let path = root.join("operation-plan.json");
    assert!(!path.exists());
    create_operation_plan(&layout, &plan()).expect("absence retained");
    fs::write(
        &path,
        vec![b' '; usize::try_from(MAX_OPERATION_PLAN_BYTES).expect("bound") + 1],
    )
    .expect("oversized evidence");
    assert!(matches!(
        read_operation_plan(&layout, &plan().digest()),
        Err(OperationPlanPersistenceError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_OPERATION_PLAN_BYTES
            }
        ))
    ));
    assert_eq!(
        fs::metadata(&path).expect("retained input").len(),
        MAX_OPERATION_PLAN_BYTES + 1
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn malformed_original_binding_unsafe_file_and_replaced_layout_reject_without_repair() {
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
