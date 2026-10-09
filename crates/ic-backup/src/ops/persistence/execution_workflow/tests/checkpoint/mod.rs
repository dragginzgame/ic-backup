//! Stage-owned predecessor publication at the original workflow boundary.

use super::*;
use crate::ops::persistence::{
    ExecutionSettlementCheckpointError, ExecutionSettlementPersistenceError,
    read_execution_settlement,
};

fn apply(stage: &ExecutionStageGuard<'_>) {
    let mut journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(42).unwrap(),
    )
    .unwrap();
    let attempt = journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: stage.plan().operation(42).unwrap().request().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: "12".repeat(32),
        })
        .unwrap();
}
#[test]
fn exact_predecessor_admits_successor_and_retains_original_checkpoint_and_spending() {
    let (root, layout, workflow) = prepare();
    let plan = child(0);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap();
    apply(&stage);
    let journal_path = stage.stage_layout.root().join("attempt-42.json");
    let bytes = fs::read(&journal_path).unwrap();
    let learned = ArtifactChecksumRecord::from_bytes(b"original learned input");
    let row = stage.checkpoint(learned.clone()).unwrap();
    assert_eq!(row.stage_sequence(), 0);
    assert_eq!(row.binding(), &binding.digest());
    assert_eq!(row.learned_evidence(), &learned);
    assert_eq!(
        read_execution_settlement(
            &stage.stage_layout,
            &stage.plan().digest(),
            row.settlement()
        )
        .unwrap()
        .plan_intent(),
        &stage.plan().digest()
    );
    let checkpoint_path = stage.stage_layout.root().join("execution-settlement.json");
    let checkpoint = fs::read(&checkpoint_path).unwrap();
    assert!(stage.checkpoint(learned).is_err());
    assert_eq!(fs::read(&checkpoint_path).unwrap(), checkpoint);
    assert_eq!(fs::read(&journal_path).unwrap(), bytes);
    drop(stage);
    let next = child(7);
    let next_binding = ExecutionStageBindingRecord::new(&workflow, 7, &next, vec![row]).unwrap();
    let successor = ExecutionStageGuard::prepare(&layout, next_binding, next).unwrap();
    drop(successor);
    let (resumed, view) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    assert_eq!(view.applied_operations, 1);
    assert_eq!(fs::read(journal_path).unwrap(), bytes);
    drop(resumed);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn held_pending_and_missing_journals_cannot_publish_a_predecessor() {
    let (root, layout, workflow) = prepare();
    let plan = child(0);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
    let mut held = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(42).unwrap(),
    )
    .unwrap();
    let learned = ArtifactChecksumRecord::from_bytes(b"unqualified learned input");
    assert!(matches!(
        stage.checkpoint(learned.clone()),
        Err(ExecutionStageCheckpointError::Settlement(
            ExecutionSettlementCheckpointError::Persistence(
                ExecutionSettlementPersistenceError::Journal(AttemptJournalError::Lock(_))
            )
        ))
    ));
    held.reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    drop(held);
    let path = stage.stage_layout.root().join("attempt-42.json");
    let original = fs::read(&path).unwrap();
    assert!(matches!(
        stage.checkpoint(learned.clone()),
        Err(ExecutionStageCheckpointError::Settlement(
            ExecutionSettlementCheckpointError::Persistence(
                ExecutionSettlementPersistenceError::Policy(_)
            )
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::rename(&path, root.join("retained-pending-journal")).unwrap();
    assert!(matches!(
        stage.checkpoint(learned),
        Err(ExecutionStageCheckpointError::Settlement(
            ExecutionSettlementCheckpointError::Persistence(
                ExecutionSettlementPersistenceError::Journal(_)
            )
        ))
    ));
    assert!(
        !stage
            .stage_layout
            .root()
            .join("execution-settlement.json")
            .exists()
    );
    assert_eq!(
        fs::read(root.join("retained-pending-journal")).unwrap(),
        original
    );
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn changed_workflow_or_binding_refuses_before_checkpoint_publication() {
    for original in [WORKFLOW_FILE, BINDING_FILE] {
        let (root, layout, workflow) = prepare();
        let plan = child(0);
        let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
        apply(&stage);
        let path = if original == WORKFLOW_FILE {
            root.join(original)
        } else {
            stage.stage_layout.root().join(original)
        };
        let original_bytes = fs::read(&path).unwrap();
        fs::write(root.join("retained-original"), &original_bytes).unwrap();
        let mut changed: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
        if original == WORKFLOW_FILE {
            changed["allocation"]["context"]["network"] = serde_json::json!("89".repeat(32));
        } else {
            changed["stage_sequence"] = serde_json::json!(7);
        }
        fs::write(&path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        let changed_bytes = fs::read(&path).unwrap();
        assert!(matches!(
            stage.checkpoint(ArtifactChecksumRecord::from_bytes(b"input")),
            Err(ExecutionStageCheckpointError::Stage(
                ExecutionWorkflowPersistenceError::DigestMismatch
            ))
        ));
        assert_eq!(fs::read(&path).unwrap(), changed_bytes);
        assert!(
            !stage
                .stage_layout
                .root()
                .join("execution-settlement.json")
                .exists()
        );
        assert!(root.join("retained-original").exists());
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn lost_post_publication_admission_retains_exact_checkpoint_without_republication() {
    let (root, layout, workflow) = prepare();
    let plan = child(0);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
    apply(&stage);
    let before = fs::read(stage.stage_layout.root().join("attempt-42.json")).unwrap();
    let error = stage
        .checkpoint_with(ArtifactChecksumRecord::from_bytes(b"input"), || {
            fs::rename(root.join(WORKFLOW_FILE), root.join("retained-workflow")).unwrap();
        })
        .unwrap_err();
    assert!(matches!(error, ExecutionStageCheckpointError::Stage(_)));
    let path = stage.stage_layout.root().join("execution-settlement.json");
    let bytes = fs::read(&path).unwrap();
    let record: crate::model::execution_settlement::ExecutionSettlementRecord =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        read_execution_settlement(
            &stage.stage_layout,
            &stage.plan().digest(),
            &record.digest(),
        )
        .unwrap(),
        record
    );
    assert!(
        stage
            .checkpoint(ArtifactChecksumRecord::from_bytes(b"input"))
            .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(
        fs::read(stage.stage_layout.root().join("attempt-42.json")).unwrap(),
        before
    );
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
