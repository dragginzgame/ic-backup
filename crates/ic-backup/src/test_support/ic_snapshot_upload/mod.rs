//! Declared originals and independent wire vectors; no simulated IC effects.

use crate::model::{
    ic_snapshot_upload::IcSnapshotUploadRequest, operation_plan::OperationPlanRecord,
};
use serde_json::json;

pub const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
pub const SOURCE_ID: &[u8] = &[0, 255, 17];
pub const DESTINATION_ID: &[u8] = &[21, 0, 255];

pub fn unhex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

pub fn raw(name: &str) -> Vec<u8> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    unhex(
        cases.iter().find(|case| case["name"] == name).unwrap()["reply_hex"]
            .as_str()
            .unwrap(),
    )
}

pub fn plan(request: &str) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1, "context":{"network":"ab".repeat(32), "caller":"2vxsx-fae", "release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":TARGET,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[TARGET], "graph":{"version":1,"nodes":[{"operation_sequence":7,"depends_on":[]}]},
        "operations":[{"operation_sequence":7,"target":TARGET,"request":request,"budget":{"mutations":1,"observations":1}}],
        "budget":{"mutations":1,"observations":1}
    })).unwrap()
}

pub fn source_plan() -> OperationPlanRecord {
    plan(&"ef".repeat(32))
}
pub fn upload_plan(payload: &IcSnapshotUploadRequest<'_>) -> OperationPlanRecord {
    plan(payload.binding_digest().hash())
}

/// Borrow independent original declarations without returning self-referential fixtures.
pub fn with_observation(
    check: impl FnOnce(
        &OperationPlanRecord,
        &IcSnapshotUploadRequest<'_>,
        &crate::model::ic_request::IcManagementRequestRecord,
        &crate::model::attempt_journal::AttemptJournalRecord,
    ),
) {
    use crate::model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
    };
    let source = source_plan();
    let original = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let bytes = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&original, &bytes).unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"retained declared source");
    let upload = IcSnapshotUploadRequest::metadata(&source, &metadata, &checksum).unwrap();
    let plan = upload_plan(&upload);
    let list = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::ListCanisterSnapshots,
        target: TARGET.into(),
        snapshot_id: None,
    })
    .unwrap();
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    journal.reserve_mutation().unwrap();
    journal
        .reserve_observation(1, list.digest().hash())
        .unwrap();
    check(&plan, &upload, &list, &journal);
}

pub fn observation_input(
    request: &crate::model::ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest<
        '_,
        '_,
    >,
) -> crate::model::ic_observation::IcObservationResponseInput {
    crate::model::ic_observation::IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: request.payload().digest(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply: candid::encode_one(Vec::<ic_management_canister_types::Snapshot>::new()).unwrap(),
        evidence: crate::model::artifacts::ArtifactChecksumRecord::from_bytes(
            b"passive list evidence",
        ),
    }
}
