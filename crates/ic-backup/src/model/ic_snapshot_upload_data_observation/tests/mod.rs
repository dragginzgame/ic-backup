use super::*;
use crate::{
    model::{
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
    },
    test_support::{
        ic_snapshot_upload::{SOURCE_ID, TARGET, unhex},
        ic_snapshot_upload_data_observation::{input, with_original},
    },
};
use ic_management_canister_types::{ReadCanisterSnapshotMetadataResult, SnapshotDataKind};

#[test]
fn exact_original_destination_regions_and_chunks_bind_both_spent_attempts() {
    for kind in [
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        SnapshotDataKind::WasmMemory { offset: 1, size: 2 },
        SnapshotDataKind::StableMemory { offset: 1, size: 2 },
        SnapshotDataKind::WasmChunk {
            hash: unhex(ArtifactChecksumRecord::from_bytes(&[42; 2]).hash()),
        },
    ] {
        with_original(kind, &[42; 2], |plan, upload, read, journal| {
            let before = journal.clone();
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            assert_eq!(request.plan(), plan);
            assert!(std::ptr::eq(request.mutation(), upload));
            assert!(std::ptr::eq(request.payload(), read));
            assert_eq!(request.authority(), journal.authority());
            assert_eq!(
                (request.mutation_attempt(), request.observation_attempt()),
                (1, 2)
            );
            assert_eq!(
                request.original_chunk_checksum(),
                &ArtifactChecksumRecord::from_bytes(&[42; 2])
            );
            request.validate_journal(journal).unwrap();
            assert_eq!(journal, &before);
        });
    }
}

#[test]
fn metadata_intent_and_changed_target_id_region_offset_size_or_hash_reject() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let metadata_upload = IcSnapshotUploadRequest::metadata(
                upload.source_plan(),
                upload.source(),
                upload.source_checksum(),
            )
            .unwrap();
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(
                    plan,
                    7,
                    journal,
                    &metadata_upload,
                    read
                ),
                Err(IcSnapshotUploadDataObservationError::DataUploadRequired)
            ));
            for kind in [
                SnapshotDataKind::WasmModule { offset: 0, size: 2 },
                SnapshotDataKind::WasmModule { offset: 1, size: 1 },
                SnapshotDataKind::WasmMemory { offset: 1, size: 2 },
                SnapshotDataKind::StableMemory { offset: 1, size: 2 },
                SnapshotDataKind::WasmChunk {
                    hash: read.metadata().metadata().wasm_chunk_store[0].hash.clone(),
                },
            ] {
                let other = IcSnapshotDataRequest::new(read.metadata(), kind).unwrap();
                assert!(matches!(
                    IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, &other),
                    Err(IcSnapshotUploadDataObservationError::ReadbackMismatch)
                ));
            }
            for (target, id) in [("aaaaa-aa", read.snapshot_id()), (TARGET, SOURCE_ID)] {
                let original = IcSnapshotMetadataRequest::new(target, id).unwrap();
                let bytes = candid::encode_one(read.metadata().metadata()).unwrap();
                let metadata = IcSnapshotMetadataReply::decode(&original, &bytes).unwrap();
                let other = IcSnapshotDataRequest::new(&metadata, read.kind().clone()).unwrap();
                assert!(matches!(
                    IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, &other),
                    Err(IcSnapshotUploadDataObservationError::ReadbackMismatch)
                ));
            }
        },
    );
    with_original(
        SnapshotDataKind::WasmChunk {
            hash: unhex(ArtifactChecksumRecord::from_bytes(&[42; 2]).hash()),
        },
        &[42; 2],
        |plan, upload, read, journal| {
            let mut values = read.metadata().metadata().clone();
            let hash = unhex(ArtifactChecksumRecord::from_bytes(b"other chunk").hash());
            values
                .wasm_chunk_store
                .push(ic_management_canister_types::ChunkHash { hash: hash.clone() });
            let bytes = candid::encode_one(values).unwrap();
            let metadata =
                IcSnapshotMetadataReply::decode(read.metadata().request(), &bytes).unwrap();
            let other = IcSnapshotDataRequest::new(&metadata, SnapshotDataKind::WasmChunk { hash })
                .unwrap();
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, &other),
                Err(IcSnapshotUploadDataObservationError::ReadbackMismatch)
            ));
        },
    );
}

#[test]
fn absent_taken_and_changed_size_destination_metadata_cannot_supply_defaults() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            for change in 0..5 {
                let mut values: ReadCanisterSnapshotMetadataResult =
                    read.metadata().metadata().clone();
                match change {
                    0 => values.source = None,
                    1 => {
                        values.source = Some(
                            ic_management_canister_types::SnapshotSource::TakenFromCanister(
                                candid::Reserved,
                            ),
                        );
                    }
                    2 => values.wasm_module_size += 1,
                    3 => values.wasm_memory_size += 1,
                    _ => values.stable_memory_size += 1,
                }
                let bytes = candid::encode_one(values).unwrap();
                let metadata =
                    IcSnapshotMetadataReply::decode(read.metadata().request(), &bytes).unwrap();
                let other = IcSnapshotDataRequest::new(&metadata, read.kind().clone()).unwrap();
                assert!(matches!(
                    IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, &other),
                    Err(IcSnapshotUploadDataObservationError::DestinationMetadataMismatch)
                ));
            }
        },
    );
}

#[test]
fn original_authority_payload_and_pending_reservation_drift_are_typed_denials() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(plan, 99, journal, upload, read),
                Err(IcSnapshotUploadDataObservationError::Upload(
                    IcSnapshotUploadAttemptError::Plan(_)
                ))
            ));
            let mut value = serde_json::to_value(plan).unwrap();
            value["operations"][0]["request"] = serde_json::json!("12".repeat(32));
            let other: OperationPlanRecord = serde_json::from_value(value).unwrap();
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(&other, 7, journal, upload, read),
                Err(IcSnapshotUploadDataObservationError::Upload(
                    IcSnapshotUploadAttemptError::AuthorityMismatch
                ))
            ));
            let mut different = AttemptJournalRecord::new(other.attempt_authority(7).unwrap());
            different.reserve_mutation().unwrap();
            different
                .reserve_observation(1, read.digest().hash())
                .unwrap();
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(&other, 7, &different, upload, read),
                Err(IcSnapshotUploadDataObservationError::Upload(
                    IcSnapshotUploadAttemptError::PayloadMismatch
                ))
            ));
            let mut empty = AttemptJournalRecord::new(journal.authority().clone());
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(plan, 7, &empty, upload, read),
                Err(IcSnapshotUploadDataObservationError::Observation(
                    IcObservationRequestError::NoPendingMutation
                ))
            ));
            empty.reserve_mutation().unwrap();
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(plan, 7, &empty, upload, read),
                Err(IcSnapshotUploadDataObservationError::Observation(
                    IcObservationRequestError::NoPendingObservation
                ))
            ));
            empty.reserve_observation(1, &"12".repeat(32)).unwrap();
            assert!(matches!(
                IcSnapshotUploadDataObservationRequest::new(plan, 7, &empty, upload, read),
                Err(IcSnapshotUploadDataObservationError::Observation(
                    IcObservationRequestError::RequestMismatch
                ))
            ));
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let mut settled = journal.clone();
            settled
                .record_observation(ObservationReceiptRequest {
                    attempt: 2,
                    request: read.digest().hash().into(),
                    outcome: ObservationOutcomeRecord::Uncertain,
                    evidence: "12".repeat(32),
                })
                .unwrap();
            assert!(matches!(
                request.validate_journal(&settled),
                Err(IcObservationRequestError::ObservationMismatch)
            ));
            assert_eq!(settled.view().pending_mutation, Some(1));
            assert_eq!(settled.view().observations_remaining, 0);
            assert!(
                settled
                    .reserve_observation(1, read.digest().hash())
                    .is_err()
            );
        },
    );
}

#[test]
fn data_response_keeps_finite_claim_bounds_redaction_and_status_list_limit() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            for (mutation, observation) in [(0, 2), (2, 2), (3, 2), (1, 1025)] {
                let mut fields = input(&request, &[42; 2]);
                fields.mutation_attempt = mutation;
                fields.observation_attempt = observation;
                assert_eq!(
                    IcSnapshotUploadDataObservationResponse::new(fields).unwrap_err(),
                    IcObservationResponseError::InvalidAttempts
                );
            }
            let mut fields = input(&request, &[42; 2]);
            fields.target = "invalid".into();
            assert_eq!(
                IcSnapshotUploadDataObservationResponse::new(fields).unwrap_err(),
                IcObservationResponseError::InvalidTarget
            );
            let mut fields = input(&request, &[42; 2]);
            fields.reply = vec![b'X'; MAX_IC_SNAPSHOT_DATA_REPLY_BYTES + 1];
            assert_eq!(
                IcSnapshotUploadDataObservationResponse::new(fields).unwrap_err(),
                IcObservationResponseError::ReplyTooLarge
            );
            let mut fields = input(&request, &[42; 2]);
            fields.reply = vec![b'X'; MAX_IC_SNAPSHOT_DATA_REPLY_BYTES];
            fields.target = fields.target.to_ascii_uppercase();
            fields.mutation_attempt = 1023;
            fields.observation_attempt = 1024;
            assert_eq!(
                crate::model::ic_observation::IcObservationResponse::new(fields.clone())
                    .unwrap_err(),
                IcObservationResponseError::ReplyTooLarge
            );
            let response = IcSnapshotUploadDataObservationResponse::new(fields).unwrap();
            assert_eq!(response.input().target, TARGET);
            assert_eq!(
                response.input().reply.len(),
                MAX_IC_SNAPSHOT_DATA_REPLY_BYTES
            );
            assert!(!format!("{response:?}").contains("XXXX"));
        },
    );
}
