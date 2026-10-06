use super::*;
use crate::{
    model::{artifacts::ArtifactChecksumRecord, ic_observation::IcObservationResponse},
    test_support::{
        ic_snapshot_upload::unhex,
        ic_snapshot_upload_data_observation::{input, with_original},
    },
};
use ic_management_canister_types::SnapshotDataKind;

#[test]
fn equal_preexisting_zero_bytes_and_different_regions_produce_no_write_outcome() {
    for kind in [
        SnapshotDataKind::WasmModule { offset: 0, size: 2 },
        SnapshotDataKind::WasmMemory { offset: 0, size: 2 },
        SnapshotDataKind::StableMemory { offset: 0, size: 2 },
    ] {
        with_original(kind, &[0; 2], |plan, upload, read, journal| {
            let before = journal.clone();
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            for chunk in [&[0; 2][..], &[42; 2][..]] {
                let response =
                    IcSnapshotUploadDataObservationResponse::new(input(&request, chunk)).unwrap();
                let view = validate_response(&request, journal, &response).unwrap();
                assert!(std::ptr::eq(view.response(), &raw const response));
                assert_eq!(view.reply().chunk(), chunk);
                assert_eq!(view.matches_original_chunk(), chunk == [0; 2]);
                assert_eq!(
                    view.reply().payload_checksum(),
                    &ArtifactChecksumRecord::from_bytes(&response.input().reply)
                );
                assert_eq!(
                    view.reply().digest(),
                    IcSnapshotDataReply::decode(read, &response.input().reply)
                        .unwrap()
                        .digest()
                );
                assert_eq!(journal, &before);
                assert_eq!(
                    (
                        journal.view().pending_mutation,
                        journal.view().pending_observation
                    ),
                    (Some(1), Some(2))
                );
                assert!(!journal.view().applied);
            }
        });
    }
}

#[test]
fn exact_known_empty_and_nonempty_chunks_reuse_hash_admission_without_receipts() {
    for chunk in [&[][..], &[42; 2][..]] {
        let kind = SnapshotDataKind::WasmChunk {
            hash: unhex(ArtifactChecksumRecord::from_bytes(chunk).hash()),
        };
        with_original(kind, chunk, |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, chunk)).unwrap();
            assert!(
                validate_response(&request, journal, &response)
                    .unwrap()
                    .matches_original_chunk()
            );
            assert!(!journal.view().applied);
            let other =
                IcSnapshotUploadDataObservationResponse::new(input(&request, b"other")).unwrap();
            assert!(matches!(
                validate_response(&request, journal, &other),
                Err(IcSnapshotUploadDataObservationAssociationError::Data(
                    IcSnapshotDataError::ChunkHashMismatch
                ))
            ));
        });
    }
}

#[test]
fn actual_authority_attempt_digest_context_and_target_drift_reuse_typed_claim_denials() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            for change in 0..8 {
                let mut fields = input(&request, &[42; 2]);
                match change {
                    0 => fields.authority = ArtifactChecksumRecord::from_bytes(b"other"),
                    1 => {
                        fields.mutation_attempt = 2;
                        fields.observation_attempt = 3;
                    }
                    2 => fields.observation_attempt = 3,
                    3 => fields.request = ArtifactChecksumRecord::from_bytes(b"other"),
                    4..=6 => {
                        let mut context = serde_json::to_value(&fields.context).unwrap();
                        let (key, value) = match change {
                            4 => ("network", "12".repeat(32)),
                            5 => ("caller", "aaaaa-aa".into()),
                            _ => ("release", "12".repeat(32)),
                        };
                        context[key] = serde_json::json!(value);
                        fields.context = serde_json::from_value(context).unwrap();
                    }
                    _ => fields.target = "aaaaa-aa".into(),
                }
                let response = IcSnapshotUploadDataObservationResponse::new(fields).unwrap();
                let IcSnapshotUploadDataObservationAssociationError::Association(error) =
                    validate_response(&request, journal, &response).unwrap_err()
                else {
                    panic!("canonical claim denial")
                };
                assert!(match change {
                    0 => matches!(error, IcObservationAssociationError::AuthorityMismatch),
                    1..=2 => matches!(error, IcObservationAssociationError::AttemptMismatch),
                    3 => matches!(error, IcObservationAssociationError::RequestMismatch),
                    4..=6 => matches!(error, IcObservationAssociationError::ContextMismatch),
                    _ => matches!(error, IcObservationAssociationError::TargetMismatch),
                });
            }
        },
    );
}

#[test]
fn malformed_inexact_excess_and_stale_data_evidence_cannot_bypass_existing_owners() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            for raw in [
                vec![],
                b"DIDL\0\0".to_vec(),
                b"invalid".to_vec(),
                candid::encode_one(
                    ic_management_canister_types::ReadCanisterSnapshotDataResult {
                        chunk: vec![42],
                    },
                )
                .unwrap(),
                candid::encode_one(
                    ic_management_canister_types::ReadCanisterSnapshotDataResult {
                        chunk: vec![42; 3],
                    },
                )
                .unwrap(),
                candid::encode_one(
                    ic_management_canister_types::ReadCanisterSnapshotDataResult {
                        chunk: vec![42; 1024 * 1024 + 1],
                    },
                )
                .unwrap(),
            ] {
                let mut fields = input(&request, &[42; 2]);
                fields.reply = raw;
                let response = IcSnapshotUploadDataObservationResponse::new(fields).unwrap();
                assert!(matches!(
                    validate_response(&request, journal, &response),
                    Err(IcSnapshotUploadDataObservationAssociationError::Data(_))
                ));
            }
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[42; 2])).unwrap();
            let empty = AttemptJournalRecord::new(journal.authority().clone());
            assert!(matches!(
                validate_response(&request, &empty, &response),
                Err(
                    IcSnapshotUploadDataObservationAssociationError::Association(
                        IcObservationAssociationError::Reservation(_)
                    )
                )
            ));
        },
    );
}

#[test]
fn maximum_data_readback_preserves_full_chunk_and_existing_list_response_ceiling() {
    let chunk = vec![42; 1024 * 1024];
    with_original(
        SnapshotDataKind::WasmModule {
            offset: 0,
            size: 1024 * 1024,
        },
        &chunk,
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let fields = input(&request, &chunk);
            assert!(fields.reply.len() > 1024 * 1024);
            assert_eq!(
                IcObservationResponse::new(fields.clone()).unwrap_err(),
                crate::model::ic_observation::IcObservationResponseError::ReplyTooLarge
            );
            let response = IcSnapshotUploadDataObservationResponse::new(fields).unwrap();
            let view = validate_response(&request, journal, &response).unwrap();
            assert_eq!(view.reply().chunk(), chunk);
            assert!(view.matches_original_chunk());
            assert!(!journal.view().applied);
        },
    );
}
