//! Public typed byte binding and local recovery; no remote snapshot effect is claimed.

use ic_backup::{
    model::{
        attempt_journal::{
            AttemptJournalRecordError, ObservationOutcomeRecord, ObservationReceiptRequest,
        },
        ic_request::{
            IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord,
            MAX_IC_REQUEST_RECORD_BYTES,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_json_durable,
        create_operation_plan, read_json, read_operation_plan,
    },
};
use serde_json::json;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn request(method: IcManagementMethodRecord) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: "renrk-eyaaa-aaaaa-aaada-cai".into(),
        snapshot_id: None,
    })
    .expect("typed closed host-ingress request")
}
fn plan(request: &IcManagementRequestRecord) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": request.target(), "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [request.target()],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": request.target(), "request": request.digest().hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).expect("original byte-bound local plan")
}

#[test]
fn preserves_typed_payload_and_independent_observation_allowance_through_reopen() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("parent")
        .join(format!(
            "ic-backup-public-wire-{}-{nonce}",
            std::process::id()
        ));
    fs::create_dir(&root).expect("layout");
    let layout = BackupLayoutGuard::acquire(&root).expect("local exclusion");
    let wire = request(IcManagementMethodRecord::TakeCanisterSnapshot);
    let plan = plan(&wire);
    let intent = plan.digest();
    create_operation_plan(&layout, &plan).expect("retained original intent");
    let path = root.join("request.json");
    // Generic bounded JSON ops retain codec declarations. This fixture does not
    // introduce a runner request layout or an authority/dispatch persistence API.
    create_json_durable(&path, &wire).expect("retain exact typed declaration");
    let original_bytes = fs::read(&path).expect("retained declaration bytes");
    let authority = plan.attempt_authority(7).expect("original authority");
    wire.validate_mutation_binding(authority.binding())
        .expect("exact bytes bound before local reservation");
    let mut journal =
        AttemptJournalGuard::create(&layout, authority).expect("original finite ledger");
    let mutation = journal
        .reserve_mutation()
        .expect("consume locally, without dispatch");
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).expect("new local owner");
    let plan = read_operation_plan(&layout, &intent).expect("same original plan");
    let retained: IcManagementRequestRecord =
        read_json(&path, MAX_IC_REQUEST_RECORD_BYTES).expect("bounded current wire owner");
    assert_eq!(retained, wire);
    assert_eq!(retained.arguments(), wire.arguments());
    let authority = plan.attempt_authority(7).expect("same issued allowance");
    retained
        .validate_mutation_binding(authority.binding())
        .expect("same payload method/routing/bytes");
    let mut journal =
        AttemptJournalGuard::open(&layout, &authority).expect("spent pending original mutation");
    let observer = request(IcManagementMethodRecord::ListCanisterSnapshots);
    let digest = observer.digest();
    observer
        .validate_observation_binding(authority.binding(), &digest)
        .expect("same target with independent observer payload");
    let observation = journal
        .reserve_observation(mutation, digest.hash())
        .expect("consume separate allowance without remote observation");
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: observation,
            request: digest.hash().into(),
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: "12".repeat(32),
        })
        .expect("caller-qualified settled local fixture, without IC proof");
    let view = journal.record().expect("retained accounting").view();
    assert_eq!(view.mutations_used, 1);
    assert_eq!(view.observations_used, 1);
    assert_eq!(view.mutations_remaining, 0);
    assert_eq!(view.observations_remaining, 0);
    assert_eq!(view.pending_mutation, Some(mutation));
    assert_eq!(view.pending_observation, None);
    assert!(
        matches!(journal.reserve_mutation(), Err(AttemptJournalError::Record(AttemptJournalRecordError::MutationPending { attempt })) if attempt == mutation)
    );
    assert!(create_json_durable(&path, &observer).is_err());
    assert_eq!(
        fs::read(&path).expect("unchanged wire evidence"),
        original_bytes
    );
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful local fixture");
}
