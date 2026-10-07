//! Original accounting and typed association denial; no fake IC qualification.

use super::*;
use crate::{
    model::{
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        ic_snapshot_data::{IcSnapshotDataRequest, MAX_IC_SNAPSHOT_DATA_REPLY_BYTES},
        ic_snapshot_metadata::{IcSnapshotMetadataRequest, MAX_IC_SNAPSHOT_METADATA_BYTES},
        ic_snapshot_transfer_read::IcSnapshotTransferReadResponseInput,
        operation_plan::OperationPlanRecord,
    },
    test_support::{control_authority, membership::hash},
};
use ic_management_canister_types::{ReadCanisterSnapshotDataResult, SnapshotDataKind};
use sha2::{Digest, Sha256};

fn metadata_wire() -> Vec<u8> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    cases
        .iter()
        .find(|case| case["name"] == "data-source")
        .unwrap()["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn plan(payload: IcSnapshotTransferReadPayload<'_, '_>) -> OperationPlanRecord {
    let mut value =
        serde_json::to_value(control_authority::plan(&control_authority::stop())).unwrap();
    value["operations"][1]["request"] = serde_json::json!(payload.digest().hash());
    serde_json::from_value(value).unwrap()
}

fn fields(
    request: &IcSnapshotTransferReadRequest<'_, '_>,
    reply: Vec<u8>,
) -> IcSnapshotTransferReadResponseInput {
    IcSnapshotTransferReadResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply,
        evidence: hash("34"),
    }
}

#[test]
fn original_metadata_and_data_associate_without_receipts_or_new_spending() {
    let metadata_request =
        IcSnapshotMetadataRequest::new(control_authority::stop().target(), &[0, 255, 128]).unwrap();
    let raw = metadata_wire();
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &raw).unwrap();
    let data = IcSnapshotDataRequest::new(
        &metadata,
        SnapshotDataKind::WasmModule { offset: 0, size: 3 },
    )
    .unwrap();
    for (payload, bytes) in [
        (
            IcSnapshotTransferReadPayload::Metadata(&metadata_request),
            raw,
        ),
        (
            IcSnapshotTransferReadPayload::Data(&data),
            candid::encode_one(ReadCanisterSnapshotDataResult {
                chunk: vec![7, 8, 9],
            })
            .unwrap(),
        ),
    ] {
        let plan = plan(payload);
        let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
        assert!(matches!(
            IcSnapshotTransferReadRequest::new(&plan, 7, &journal, payload),
            Err(IcSnapshotTransferReadError::NoPendingMutation)
        ));
        journal.reserve_mutation().unwrap();
        let before = journal.clone();
        let request = IcSnapshotTransferReadRequest::new(&plan, 7, &journal, payload).unwrap();
        let response = IcSnapshotTransferReadResponse::new(fields(&request, bytes)).unwrap();
        let view = validate_response(&request, &journal, &response).unwrap();
        assert!(std::ptr::eq(view.response(), &raw const response));
        assert_eq!(view.response().input().evidence, hash("34"));
        match view.reply() {
            IcSnapshotTransferReadReply::Metadata(reply) => {
                assert_eq!(reply.digest(), metadata.digest());
            }
            IcSnapshotTransferReadReply::Data(reply) => assert_eq!(reply.chunk(), [7, 8, 9]),
        }
        assert_eq!(journal, before);
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert!(!journal.view().applied);
        assert!(matches!(
            journal.reserve_mutation(),
            Err(
                crate::model::attempt_journal::AttemptJournalRecordError::MutationPending {
                    attempt: 1
                }
            )
        ));
        assert_eq!(journal, before);
    }
}

#[test]
fn changed_original_payload_authority_and_current_reservations_reject() {
    let metadata_request =
        IcSnapshotMetadataRequest::new(control_authority::stop().target(), &[0, 255, 128]).unwrap();
    let payload = IcSnapshotTransferReadPayload::Metadata(&metadata_request);
    let plan = plan(payload);
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    journal.reserve_mutation().unwrap();
    let request = IcSnapshotTransferReadRequest::new(&plan, 7, &journal, payload).unwrap();
    let response = IcSnapshotTransferReadResponse::new(fields(&request, metadata_wire())).unwrap();
    for (target, id) in [
        (metadata_request.target(), vec![1]),
        ("aaaaa-aa", vec![0, 255, 128]),
    ] {
        let other = IcSnapshotMetadataRequest::new(target, &id).unwrap();
        assert!(matches!(
            IcSnapshotTransferReadRequest::new(
                &plan,
                7,
                &journal,
                IcSnapshotTransferReadPayload::Metadata(&other)
            ),
            Err(IcSnapshotTransferReadError::PayloadMismatch)
        ));
    }
    let other = AttemptJournalRecord::new(plan.attempt_authority(0).unwrap());
    assert!(matches!(
        request.validate_journal(&other),
        Err(IcSnapshotTransferReadError::AuthorityMismatch)
    ));
    let mut recovery = journal.clone();
    recovery.reserve_observation(1, hash("56").hash()).unwrap();
    assert!(matches!(
        validate_response(&request, &recovery, &response),
        Err(IcSnapshotTransferReadAssociationError::Reservation(
            IcSnapshotTransferReadError::ObservationPending
        ))
    ));
    journal
        .record_mutation(MutationReceiptRequest {
            attempt: 1,
            request: payload.digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: hash("34").hash().into(),
        })
        .unwrap();
    assert!(matches!(
        validate_response(&request, &journal, &response),
        Err(IcSnapshotTransferReadAssociationError::Reservation(
            IcSnapshotTransferReadError::MutationMismatch
        ))
    ));
}

#[test]
fn actual_claims_attempt_and_bounded_wire_fail_without_settlement() {
    let metadata_request =
        IcSnapshotMetadataRequest::new(control_authority::stop().target(), &[0, 255, 128]).unwrap();
    let payload = IcSnapshotTransferReadPayload::Metadata(&metadata_request);
    let plan = plan(payload);
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    journal.reserve_mutation().unwrap();
    let before = journal.clone();
    let request = IcSnapshotTransferReadRequest::new(&plan, 7, &journal, payload).unwrap();
    for change in 0..8 {
        let mut input = fields(&request, metadata_wire());
        match change {
            0 => input.authority = hash("12"),
            1 => input.mutation_attempt = 2,
            2..=4 => {
                let mut context = serde_json::to_value(&input.context).unwrap();
                let (key, value) = match change {
                    2 => ("network", hash("12").hash().to_owned()),
                    3 => ("caller", "aaaaa-aa".into()),
                    _ => ("release", hash("12").hash().to_owned()),
                };
                context[key] = serde_json::json!(value);
                input.context = serde_json::from_value(context).unwrap();
            }
            5 => input.target = "aaaaa-aa".into(),
            6 => input.reply = b"DIDL\0\0".to_vec(),
            _ => input.reply = vec![0; MAX_IC_SNAPSHOT_METADATA_BYTES + 1],
        }
        let response = IcSnapshotTransferReadResponse::new(input).unwrap();
        let error = validate_response(&request, &journal, &response).unwrap_err();
        assert!(match change {
            0 => matches!(
                error,
                IcSnapshotTransferReadAssociationError::AuthorityMismatch
            ),
            1 => matches!(
                error,
                IcSnapshotTransferReadAssociationError::AttemptMismatch
            ),
            2..=4 => matches!(
                error,
                IcSnapshotTransferReadAssociationError::ContextMismatch
            ),
            5 => matches!(
                error,
                IcSnapshotTransferReadAssociationError::TargetMismatch
            ),
            6 => matches!(error, IcSnapshotTransferReadAssociationError::Metadata(_)),
            _ => matches!(
                error,
                IcSnapshotTransferReadAssociationError::Metadata(
                    IcSnapshotMetadataError::ReplyTooLarge
                )
            ),
        });
        assert_eq!(journal, before);
    }
    for attempt in [0, 1025] {
        let mut input = fields(&request, vec![]);
        input.mutation_attempt = attempt;
        assert!(matches!(
            IcSnapshotTransferReadResponse::new(input),
            Err(IcSnapshotTransferReadError::InvalidAttempt)
        ));
    }
    let mut input = fields(&request, vec![0; MAX_IC_SNAPSHOT_DATA_REPLY_BYTES + 1]);
    assert!(matches!(
        IcSnapshotTransferReadResponse::new(input.clone()),
        Err(IcSnapshotTransferReadError::ReplyTooLarge)
    ));
    input.reply.clear();
    input.target = "invalid principal".into();
    assert!(matches!(
        IcSnapshotTransferReadResponse::new(input),
        Err(IcSnapshotTransferReadError::InvalidTarget)
    ));
}

#[test]
fn data_length_hash_and_method_mismatch_use_existing_decoder_errors() {
    let metadata_request =
        IcSnapshotMetadataRequest::new(control_authority::stop().target(), &[0, 255, 128]).unwrap();
    let raw = metadata_wire();
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &raw).unwrap();
    for kind in [
        SnapshotDataKind::WasmModule { offset: 0, size: 3 },
        SnapshotDataKind::WasmChunk {
            hash: metadata.metadata().wasm_chunk_store[0].hash.clone(),
        },
    ] {
        let data = IcSnapshotDataRequest::new(&metadata, kind.clone()).unwrap();
        let payload = IcSnapshotTransferReadPayload::Data(&data);
        let plan = plan(payload);
        let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
        journal.reserve_mutation().unwrap();
        assert!(matches!(
            IcSnapshotTransferReadRequest::new(
                &plan,
                7,
                &journal,
                IcSnapshotTransferReadPayload::Metadata(&metadata_request)
            ),
            Err(IcSnapshotTransferReadError::PayloadMismatch)
        ));
        let request = IcSnapshotTransferReadRequest::new(&plan, 7, &journal, payload).unwrap();
        let bytes = candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![99] }).unwrap();
        let response = IcSnapshotTransferReadResponse::new(fields(&request, bytes)).unwrap();
        let error = validate_response(&request, &journal, &response).unwrap_err();
        assert!(match kind {
            SnapshotDataKind::WasmChunk { .. } => matches!(
                error,
                IcSnapshotTransferReadAssociationError::Data(
                    IcSnapshotDataError::ChunkHashMismatch
                )
            ),
            _ => matches!(
                error,
                IcSnapshotTransferReadAssociationError::Data(IcSnapshotDataError::LengthMismatch)
            ),
        });
        assert_eq!(journal.view().pending_mutation, Some(1));
    }
}

#[test]
fn full_bounded_data_and_empty_known_chunks_keep_original_byte_and_hash_limits() {
    let metadata_request =
        IcSnapshotMetadataRequest::new(control_authority::stop().target(), &[0, 255, 128]).unwrap();
    let mut values: ic_management_canister_types::ReadCanisterSnapshotMetadataResult =
        candid::decode_one(&metadata_wire()).unwrap();
    values.wasm_module_size =
        crate::model::ic_snapshot_data::MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64;
    values.wasm_chunk_store = vec![ic_management_canister_types::ChunkHash {
        hash: Sha256::digest([]).to_vec(),
    }];
    let raw = candid::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &raw).unwrap();
    for (kind, bytes) in [
        (
            SnapshotDataKind::WasmModule {
                offset: 0,
                size: metadata.metadata().wasm_module_size,
            },
            vec![42; crate::model::ic_snapshot_data::MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES],
        ),
        (
            SnapshotDataKind::WasmChunk {
                hash: Sha256::digest([]).to_vec(),
            },
            vec![],
        ),
    ] {
        let data = IcSnapshotDataRequest::new(&metadata, kind).unwrap();
        let payload = IcSnapshotTransferReadPayload::Data(&data);
        let plan = plan(payload);
        let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
        journal.reserve_mutation().unwrap();
        let request = IcSnapshotTransferReadRequest::new(&plan, 7, &journal, payload).unwrap();
        let raw = candid::encode_one(ReadCanisterSnapshotDataResult {
            chunk: bytes.clone(),
        })
        .unwrap();
        if !bytes.is_empty() {
            assert!(raw.len() > MAX_IC_SNAPSHOT_METADATA_BYTES);
        }
        let response = IcSnapshotTransferReadResponse::new(fields(&request, raw)).unwrap();
        let view = validate_response(&request, &journal, &response).unwrap();
        let IcSnapshotTransferReadReply::Data(reply) = view.reply() else {
            panic!("data branch")
        };
        assert_eq!(reply.chunk(), bytes);
        assert_eq!(
            reply.digest(),
            IcSnapshotDataReply::decode(&data, &response.input().reply)
                .unwrap()
                .digest()
        );
        assert_eq!(journal.view().pending_mutation, Some(1));
    }
}
