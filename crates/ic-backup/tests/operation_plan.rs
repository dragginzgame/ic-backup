//! Public immutable plan-to-journal binding, with local evidence and no management backend.

use ic_backup::{
    model::{
        attempt_journal::{AttemptBudgetRecord, AttemptJournalRecordError},
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
        operation_plan::{
            OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlanContextRecord,
            PlanContextRequest, PlannedOperationRecord, PlannedOperationRequest,
        },
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_operation_plan,
        read_operation_plan,
    },
};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn original_plan_authority_reopens_consumed_attempts_without_rebinding_or_replenishment() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("parent")
        .join(format!(
            "ic-backup-public-plan-{}-{nonce}",
            std::process::id()
        ));
    fs::create_dir(&root).expect("layout");
    let layout = BackupLayoutGuard::acquire(&root).expect("exclusion");
    let plan = OperationPlanRecord::new(OperationPlanRequest {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: "ab".repeat(32),
            caller: "2vxsx-fae".into(),
            release: "cd".repeat(32),
        })
        .expect("declared context"),
        inventory: InventoryRecord::new(vec![
            InventoryTargetRecord::new(&InventoryTargetRequest {
                canister_id: "aaaaa-aa".into(),
                parent_canister_id: None,
                role: None,
                module_hash: None,
            })
            .expect("row"),
        ])
        .expect("explicit inventory"),
        selected_targets: vec!["aaaaa-aa".into()],
        graph: EffectGraphRecord::new(vec![
            EffectNodeRecord::new(EffectNodeRequest {
                operation_sequence: 7,
                depends_on: vec![],
            })
            .expect("node"),
        ])
        .expect("graph"),
        operations: vec![
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: 7,
                target: "aaaaa-aa".into(),
                request: "ef".repeat(32),
                budget: AttemptBudgetRecord::new(1, 1).expect("original limits"),
            })
            .expect("binding"),
        ],
        budget: PlanBudgetRecord::new(1, 1).expect("aggregate ceilings"),
    })
    .expect("bound declaration");
    let intent = plan.digest();
    create_operation_plan(&layout, &plan).expect("retain exact intent");
    let original = fs::read(root.join("operation-plan.json")).expect("original bytes");
    let authority = plan
        .attempt_authority(7)
        .expect("derive exact identity and original limits");
    let mut guard =
        AttemptJournalGuard::create(&layout, authority.clone()).expect("one original journal");
    assert_eq!(
        guard
            .reserve_mutation()
            .expect("consume locally before caller-owned call"),
        1
    );
    drop(guard);
    let retained = read_operation_plan(&layout, &intent).expect("original reviewed digest");
    let repeated = retained
        .attempt_authority(7)
        .expect("repeat declaration only");
    assert_eq!(authority, repeated);
    let mut guard =
        AttemptJournalGuard::open(&layout, &repeated).expect("original pending journal");
    let view = guard.record().expect("retained accounting").view();
    assert_eq!(view.mutations_used, 1);
    assert_eq!(view.mutations_remaining, 0);
    assert_eq!(view.pending_mutation, Some(1));
    assert!(matches!(
        guard.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { attempt: 1 }
        ))
    ));
    drop(guard);
    assert!(AttemptJournalGuard::create(&layout, repeated).is_err());
    assert_eq!(
        fs::read(root.join("operation-plan.json")).expect("unchanged plan"),
        original
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}
