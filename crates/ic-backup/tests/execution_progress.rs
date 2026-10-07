//! Public local progress recovery; fixture receipts qualify no remote effects.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{
            AttemptJournalRecord, MutationOutcomeRecord, MutationReceiptRequest,
            ObservationOutcomeRecord, ObservationReceiptRequest,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, create_operation_plan, read_operation_plan,
    },
    policy::execution_progress::{
        ExecutionProgressError, ExecutionProgressRequest, ExecutionProgressView,
        OperationProgressState, progress,
    },
};
use serde_json::json;
use std::{fs, path::PathBuf};

struct RetainedFixture {
    root: PathBuf,
    intent: ArtifactChecksumRecord,
    before: ExecutionProgressView,
    plan_bytes: Vec<u8>,
    first_bytes: Vec<u8>,
    second_bytes: Vec<u8>,
    observation: u32,
    request: String,
}

fn retain_pending_fixture() -> RetainedFixture {
    let root = support::temp_root("ic-backup-public-progress");
    let layout = BackupLayoutGuard::acquire(&root).expect("layout exclusion");
    let plan: OperationPlanRecord = serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": "aaaaa-aa", "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": ["aaaaa-aa"],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 0, "depends_on": []}, {"operation_sequence": 7, "depends_on": [0]}]},
        "operations": [
            {"operation_sequence": 0, "target": "aaaaa-aa", "request": "ef".repeat(32), "budget": {"mutations": 1, "observations": 1}},
            {"operation_sequence": 7, "target": "aaaaa-aa", "request": "01".repeat(32), "budget": {"mutations": 1, "observations": 1}}
        ],
        "budget": {"mutations": 10, "observations": 10}
    })).expect("strict original declaration");
    let intent = plan.digest();
    create_operation_plan(&layout, &plan).expect("durable original plan");
    let plan_bytes = fs::read(root.join("operation-plan.json")).expect("original plan bytes");
    let mut first =
        AttemptJournalGuard::create(&layout, plan.attempt_authority(0).expect("first authority"))
            .expect("first original journal");
    let mut second = AttemptJournalGuard::create(
        &layout,
        plan.attempt_authority(7).expect("second authority"),
    )
    .expect("second original journal");
    let mutation = first
        .reserve_mutation()
        .expect("original allowance consumed");
    first
        .record_mutation(MutationReceiptRequest {
            attempt: mutation,
            request: "ef".repeat(32),
            outcome: MutationOutcomeRecord::Applied,
            evidence: "12".repeat(32),
        })
        .expect("caller-qualified local receipt fixture");
    let mutation = second
        .reserve_mutation()
        .expect("dependent original allowance consumed");
    let request = "23".repeat(32);
    let observation = second
        .reserve_observation(mutation, &request)
        .expect("observation consumption retained before reply");
    let references = [
        second.record().expect("second journal"),
        first.record().expect("first journal"),
    ];
    let before = progress(&ExecutionProgressRequest {
        plan: &plan,
        journals: &references,
    })
    .expect("exact bound local progress");
    assert_eq!(before.applied_operations, 1);
    assert_eq!(before.operations[0].state, OperationProgressState::Applied);
    assert_eq!(
        before.operations[1].state,
        OperationProgressState::ObservationUnresolved
    );
    assert_eq!(before.attempts.mutations_used, 2);
    assert_eq!(before.attempts.observations_used, 1);
    assert_eq!(
        before.attempts.mutations_remaining, 0,
        "unassigned plan headroom is excluded"
    );
    let first_bytes = fs::read(first.path()).expect("first journal bytes");
    let second_bytes = fs::read(second.path()).expect("second journal bytes");
    drop(first);
    drop(second);
    drop(layout);

    RetainedFixture {
        root,
        intent,
        before,
        plan_bytes,
        first_bytes,
        second_bytes,
        observation,
        request,
    }
}

fn reopen<'a>(
    layout: &'a BackupLayoutGuard,
    plan: &OperationPlanRecord,
    sequence: u64,
) -> AttemptJournalGuard<'a> {
    AttemptJournalGuard::open(
        layout,
        &plan
            .attempt_authority(sequence)
            .expect("original authority"),
    )
    .expect("retained original journal")
}

#[test]
fn recovers_exact_original_progress_without_calls_or_evidence_replacement() {
    let RetainedFixture {
        root,
        intent,
        before,
        plan_bytes,
        first_bytes,
        second_bytes,
        observation,
        request,
    } = retain_pending_fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("fresh layout exclusion");
    let original = read_operation_plan(&layout, &intent).expect("retain original intent");
    let first = reopen(&layout, &original, 0);
    let mut second = reopen(&layout, &original, 7);
    let references = [
        first.record().expect("first"),
        second.record().expect("second"),
    ];
    assert_eq!(
        progress(&ExecutionProgressRequest {
            plan: &original,
            journals: &references
        })
        .expect("local replay only"),
        before
    );
    assert_eq!(
        fs::read(root.join("operation-plan.json")).expect("retained plan"),
        plan_bytes
    );
    assert_eq!(fs::read(first.path()).expect("retained first"), first_bytes);
    assert_eq!(
        fs::read(second.path()).expect("retained second"),
        second_bytes
    );
    assert_eq!(
        progress(&ExecutionProgressRequest {
            plan: &original,
            journals: &references[..1]
        })
        .expect_err("no missing-journal reset"),
        ExecutionProgressError::MissingJournal(7)
    );
    let empty = AttemptJournalRecord::new(original.attempt_authority(0).expect("declaration only"));
    assert_eq!(
        progress(&ExecutionProgressRequest {
            plan: &original,
            journals: &[&empty, second.record().expect("second")]
        })
        .expect_err("discarding prerequisite evidence rejects"),
        ExecutionProgressError::PrematureAttempt {
            operation_sequence: 7,
            prerequisite: 0
        }
    );
    second
        .record_observation(ObservationReceiptRequest {
            attempt: observation,
            request,
            outcome: ObservationOutcomeRecord::Applied,
            evidence: "34".repeat(32),
        })
        .expect("caller-qualified settled local fixture");
    let references = [
        first.record().expect("first"),
        second.record().expect("second"),
    ];
    let after = progress(&ExecutionProgressRequest {
        plan: &original,
        journals: &references,
    })
    .expect("retained applied operation receipts");
    assert_eq!(after.applied_operations, 2);
    assert!(
        after
            .operations
            .iter()
            .all(|operation| operation.state == OperationProgressState::Applied)
    );
    assert_eq!(after.attempts.mutations_used, 2);
    assert_eq!(after.attempts.observations_used, 1);
    assert_eq!(after.attempts.mutations_remaining, 0);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).expect("unchanged plan"),
        plan_bytes
    );
    drop(first);
    drop(second);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful local fixture");
}
