//! Public learned-stage binding and pending recovery; native receipts prove no IC effects.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_settlement::{ExecutionSettlementJournalRecord, ExecutionSettlementRecord},
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
        },
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, ExecutionStageGuard, create_execution_settlement,
        create_execution_workflow,
    },
};
use serde_json::json;
use std::fs;

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";

fn plan(request: &ArtifactChecksumRecord) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": TARGET, "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [TARGET],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 42, "depends_on": []}]},
        "operations": [{"operation_sequence": 42, "target": TARGET, "request": request.hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).unwrap()
}
fn workflow() -> ExecutionWorkflowRecord {
    let mut allocation = serde_json::to_value(plan(&ArtifactChecksumRecord::from_bytes(
        b"capture contract",
    )))
    .unwrap();
    allocation["graph"]["nodes"] = json!([
        {"operation_sequence": 0, "depends_on": []},
        {"operation_sequence": 7, "depends_on": [0]}
    ]);
    allocation["operations"] = json!([
        {"operation_sequence": 0, "target": TARGET, "request": ArtifactChecksumRecord::from_bytes(b"capture contract").hash(), "budget": {"mutations": 1, "observations": 1}},
        {"operation_sequence": 7, "target": TARGET, "request": ArtifactChecksumRecord::from_bytes(b"metadata contract").hash(), "budget": {"mutations": 1, "observations": 1}}
    ]);
    // Unassigned workflow headroom cannot enlarge a stage or grant paid retry.
    allocation["budget"] = json!({"mutations": 10, "observations": 10});
    ExecutionWorkflowRecord::new(serde_json::from_value(allocation).unwrap())
}
fn retain_capture(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
) -> ExecutionStagePredecessorRecord {
    let capture = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::TakeCanisterSnapshot,
        target: TARGET.into(),
        snapshot_id: None,
    })
    .unwrap();
    let plan = plan(&capture.digest());
    let binding = ExecutionStageBindingRecord::new(workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(layout, binding.clone(), plan).unwrap();
    let mut journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(42).unwrap(),
    )
    .unwrap();
    capture
        .validate_mutation_binding(journal.record().unwrap().authority().binding())
        .unwrap();
    let attempt = journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: capture.digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: ArtifactChecksumRecord::from_bytes(b"test-only capture claim")
                .hash()
                .into(),
        })
        .unwrap();
    let settlement = ExecutionSettlementRecord::new(
        stage.plan().digest(),
        vec![ExecutionSettlementJournalRecord::from_journal(
            journal.record().unwrap(),
        )],
    )
    .unwrap();
    drop(journal);
    create_execution_settlement(stage.layout().unwrap(), &settlement).unwrap();
    ExecutionStagePredecessorRecord::new(
        0,
        binding.digest(),
        settlement.digest(),
        ArtifactChecksumRecord::from_bytes(b"test-owned learned raw ID [0,255,17]"),
    )
}

#[test]
fn learned_snapshot_payload_is_bound_once_and_pending_read_retains_original_allowance() {
    let root = support::temp_root("ic-backup-public-workflow");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let workflow = workflow();
    create_execution_workflow(&layout, &workflow).unwrap();
    let predecessor = retain_capture(&layout, &workflow);
    let metadata = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let metadata_plan = plan(&metadata.digest());
    let binding =
        ExecutionStageBindingRecord::new(&workflow, 7, &metadata_plan, vec![predecessor]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), metadata_plan).unwrap();
    let authority = stage.plan().attempt_authority(42).unwrap();
    assert_eq!(authority.binding().request(), metadata.digest().hash());
    let mut journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    let attempt = journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    let bytes = fs::read(journal.path()).unwrap();
    drop(journal);
    drop(stage);
    // A lost read reply leaves the original reservation pending. Reopen is local.
    let (stage, view) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    assert_eq!(view.attempts.mutations_used, 1);
    assert_eq!(view.attempts.mutations_remaining, 0);
    let mut journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    assert_eq!(
        journal.record().unwrap().view().pending_mutation,
        Some(attempt)
    );
    assert!(journal.reserve_mutation().is_err());
    assert_eq!(fs::read(journal.path()).unwrap(), bytes);
    drop(journal);
    drop(stage);
    assert!(ExecutionStageGuard::create(&layout, binding, plan(&metadata.digest())).is_err());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
