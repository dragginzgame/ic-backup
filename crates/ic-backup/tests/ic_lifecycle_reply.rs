//! Native retained load/status reply replay; no transport or effect settlement.

use candid::{CandidType, Principal};
use ic_backup::{
    model::{
        artifacts::{ArtifactChecksumRecord, ChecksumError},
        attempt_journal::AttemptJournalRecordError,
        ic_lifecycle_reply::{IcLifecycleReply, IcLifecycleReplyError, IcLifecycleReplyKind},
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_json_durable,
        create_operation_plan, read_json, read_operation_plan,
    },
};
use ic_management_canister_types::CanisterStatusType;
use serde_json::json;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(CandidType)]
struct Settings {
    controllers: Vec<Principal>,
}
#[derive(CandidType)]
struct Status {
    status: CanisterStatusType,
    settings: Settings,
}

fn request(
    method: IcManagementMethodRecord,
    snapshot_id: Option<Vec<u8>>,
) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: "renrk-eyaaa-aaaaa-aaada-cai".into(),
        snapshot_id,
    })
    .expect("exact immutable request")
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one public recovery journey retains original load identity, status wire evidence and spent attempts together"
)]
fn status_and_acknowledgement_replay_preserves_pending_load_and_observation() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("parent")
        .join(format!(
            "ic-backup-public-lifecycle-reply-{}-{nonce}",
            std::process::id()
        ));
    fs::create_dir(&root).expect("private fixture layout");
    let layout = BackupLayoutGuard::acquire(&root).expect("layout exclusion");
    let load = request(
        IcManagementMethodRecord::LoadCanisterSnapshot,
        Some(vec![0, 255, 17]),
    );
    let status = request(IcManagementMethodRecord::CanisterStatus, None);
    let plan: OperationPlanRecord = serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": load.target(), "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [load.target()],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": load.target(), "request": load.digest().hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).expect("original intent");
    create_operation_plan(&layout, &plan).expect("retain original plan");
    let intent = plan.digest();
    let authority = plan.attempt_authority(7).expect("original authority");
    load.validate_mutation_binding(authority.binding())
        .expect("original exact load");
    let mut journal = AttemptJournalGuard::create(&layout, authority).expect("original journal");
    let mutation = journal
        .reserve_mutation()
        .expect("reserve original mutation");
    status
        .validate_observation_binding(
            journal.record().expect("record").authority().binding(),
            &status.digest(),
        )
        .expect("independent exact observation");
    let observation = journal
        .reserve_observation(mutation, status.digest().hash())
        .expect("consume original observation allowance");
    // Local immutable integration fixtures, not authenticated replies or receipts.
    let load_wire = b"DIDL\0\0".to_vec();
    let status_wire = candid::encode_one(Status {
        status: CanisterStatusType::Stopped,
        settings: Settings {
            controllers: vec![Principal::anonymous()],
        },
    })
    .expect("local status wire fixture");
    let load_path = root.join("load-reply-bytes.json");
    let status_path = root.join("status-reply-bytes.json");
    create_json_durable(&load_path, &load_wire).expect("retain exact load bytes");
    create_json_durable(&status_path, &status_wire).expect("retain exact status bytes");
    let expected_load = IcLifecycleReply::decode(&load, &load_wire)
        .expect("load admission")
        .digest();
    let expected_status = IcLifecycleReply::decode(&status, &status_wire)
        .expect("status admission")
        .digest();
    let journal_path = journal.path();
    let journal_bytes = fs::read(&journal_path).expect("original spent journal bytes");
    let load_bytes = fs::read(&load_path).expect("load fixture evidence");
    let status_bytes = fs::read(&status_path).expect("status fixture evidence");
    let original_view = journal.record().expect("pending original spending").view();
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).expect("new owner");
    let plan = read_operation_plan(&layout, &intent).expect("original plan");
    let mut journal = AttemptJournalGuard::open(
        &layout,
        &plan.attempt_authority(7).expect("same original authority"),
    )
    .expect("spent pending journal");
    let retained_load: Vec<u8> = read_json(&load_path, 8192).expect("bounded load fixture read");
    let retained_status: Vec<u8> =
        read_json(&status_path, 8192).expect("bounded status fixture read");
    let ack = IcLifecycleReply::decode(&load, &retained_load).expect("effect-free local replay");
    assert_eq!(ack.kind(), &IcLifecycleReplyKind::Acknowledgement);
    assert_eq!(ack.digest(), expected_load);
    let reply = IcLifecycleReply::decode(&status, &retained_status).expect("local status replay");
    assert_eq!(reply.digest(), expected_status);
    assert_eq!(
        reply.payload_checksum(),
        &ArtifactChecksumRecord::from_bytes(&status_wire)
    );
    let IcLifecycleReplyKind::Status(info) = reply.kind() else {
        panic!("status projection")
    };
    assert_eq!(info.status(), CanisterStatusType::Stopped);
    assert_eq!(info.controllers().principals(), ["2vxsx-fae"]);
    let changed = request(
        IcManagementMethodRecord::LoadCanisterSnapshot,
        Some(vec![1]),
    );
    let rebound =
        IcLifecycleReply::decode(&changed, &retained_load).expect("wire has no snapshot identity");
    assert_ne!(rebound.digest(), expected_load);
    assert!(matches!(
        rebound.digest().verify(expected_load.hash()),
        Err(ChecksumError::ChecksumMismatch { .. })
    ));
    assert_eq!(
        IcLifecycleReply::decode(&load, &retained_status).unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
    assert_eq!(
        IcLifecycleReply::decode(&status, &retained_load).unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
    assert_eq!(
        IcLifecycleReply::decode(&status, b"lost status reply").unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
    assert_eq!(
        journal.record().expect("unchanged accounting").view(),
        original_view
    );
    assert_eq!(original_view.pending_mutation, Some(mutation));
    assert_eq!(original_view.pending_observation, Some(observation));
    assert_eq!(original_view.mutations_remaining, 0);
    assert_eq!(original_view.observations_remaining, 0);
    assert!(matches!(journal.reserve_mutation(),
        Err(AttemptJournalError::Record(AttemptJournalRecordError::ObservationPending { attempt }))
        if attempt == observation));
    assert_eq!(
        fs::read(&journal_path).expect("journal bytes preserved"),
        journal_bytes
    );
    assert_eq!(
        fs::read(&load_path).expect("load bytes preserved"),
        load_bytes
    );
    assert_eq!(
        fs::read(&status_path).expect("status bytes preserved"),
        status_bytes
    );
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful owned fixture");
}
