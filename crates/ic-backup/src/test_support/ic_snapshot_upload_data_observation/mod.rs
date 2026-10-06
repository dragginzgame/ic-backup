//! Independently declared original source/destination data; no simulated IC effects.

use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        ic_observation::IcObservationResponseInput,
        ic_snapshot_data::IcSnapshotDataRequest,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_upload::IcSnapshotUploadRequest,
        ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationRequest,
        operation_plan::OperationPlanRecord,
    },
    test_support::ic_snapshot_upload::{
        DESTINATION_ID, SOURCE_ID, TARGET, raw, source_plan, upload_plan,
    },
};
use ic_management_canister_types::{
    ReadCanisterSnapshotMetadataResult, SnapshotDataKind, SnapshotSource,
};

pub fn with_original(
    kind: SnapshotDataKind,
    chunk: &[u8],
    check: impl FnOnce(
        &OperationPlanRecord,
        &IcSnapshotUploadRequest<'_>,
        &IcSnapshotDataRequest<'_>,
        &AttemptJournalRecord,
    ),
) {
    let source = source_plan();
    let source_request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let mut values: ReadCanisterSnapshotMetadataResult =
        candid::decode_one(&raw("upload-source")).unwrap();
    values.wasm_module_size = 1024 * 1024;
    values.wasm_memory_size = 1024 * 1024;
    values.stable_memory_size = 1024 * 1024;
    if let SnapshotDataKind::WasmChunk { hash } = &kind {
        values.wasm_chunk_store.clear();
        values
            .wasm_chunk_store
            .push(ic_management_canister_types::ChunkHash { hash: hash.clone() });
    }
    let bytes = candid::encode_one(values.clone()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&source_request, &bytes).unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"retained source declaration");
    let metadata_upload = IcSnapshotUploadRequest::metadata(&source, &metadata, &checksum).unwrap();
    let upload =
        IcSnapshotUploadRequest::data(&metadata_upload, DESTINATION_ID, kind.clone(), chunk)
            .unwrap();
    let plan = upload_plan(&upload);
    let destination_request = IcSnapshotMetadataRequest::new(TARGET, DESTINATION_ID).unwrap();
    values.source = Some(SnapshotSource::MetadataUpload(candid::Reserved));
    let destination_bytes = candid::encode_one(values).unwrap();
    let destination =
        IcSnapshotMetadataReply::decode(&destination_request, &destination_bytes).unwrap();
    let read = IcSnapshotDataRequest::new(&destination, kind).unwrap();
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    journal.reserve_mutation().unwrap();
    journal
        .reserve_observation(1, read.digest().hash())
        .unwrap();
    check(&plan, &upload, &read, &journal);
}

pub fn input(
    request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
    chunk: &[u8],
) -> IcObservationResponseInput {
    IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: request.payload().digest(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply: candid::encode_one(
            ic_management_canister_types::ReadCanisterSnapshotDataResult {
                chunk: chunk.to_vec(),
            },
        )
        .unwrap(),
        evidence: ArtifactChecksumRecord::from_bytes(b"passive retained readback declaration"),
    }
}
