//! Exact SDK fields, source/wire identity, bounds and original pending spending.

use super::*;

#[test]
fn independently_registered_upload_wire_and_hash_goldens_match_every_case() {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!("golden.json")).unwrap();
    let source_plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"independent declared source tree");
    let mut seen = std::collections::BTreeSet::new();
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        assert!(seen.insert(name));
        let metadata =
            IcSnapshotMetadataReply::decode(&request, &raw(case["source"].as_str().unwrap()))
                .unwrap();
        assert_eq!(
            metadata.digest().hash(),
            case["metadata_digest"].as_str().unwrap()
        );
        let original =
            IcSnapshotUploadRequest::metadata(&source_plan, &metadata, &checksum).unwrap();
        let payload = if let Some(kind) = case.get("source_kind") {
            let kind: SnapshotDataKind = serde_json::from_value(kind.clone()).unwrap();
            let chunk: Vec<u8> = serde_json::from_value(case["chunk"].clone()).unwrap();
            let data =
                IcSnapshotUploadRequest::data(&original, DESTINATION_ID, kind, &chunk).unwrap();
            assert_eq!(
                ArtifactChecksumRecord::from_bytes(&chunk).hash(),
                case["chunk_checksum"].as_str().unwrap()
            );
            data
        } else {
            original
        };
        assert_eq!(payload.method(), case["method"].as_str().unwrap());
        assert_eq!(
            source_plan.digest().hash(),
            case["source_plan_digest"].as_str().unwrap()
        );
        assert_eq!(checksum.hash(), case["source_checksum"].as_str().unwrap());
        assert_eq!(
            payload.binding_digest().hash(),
            case["binding_digest"].as_str().unwrap(),
            "{name}"
        );
        assert_eq!(
            payload.arguments(),
            unhex(case["arguments_hex"].as_str().unwrap()),
            "{name}"
        );
        assert_eq!(
            payload.digest().hash(),
            case["request_digest"].as_str().unwrap(),
            "{name}"
        );
        let reply =
            IcSnapshotUploadReply::decode(&payload, &unhex(case["reply_hex"].as_str().unwrap()))
                .unwrap();
        assert_eq!(
            reply.payload_checksum().hash(),
            case["payload_checksum"].as_str().unwrap()
        );
        assert_eq!(
            reply.digest().hash(),
            case["digest"].as_str().unwrap(),
            "{name}"
        );
    }
    let registered: std::collections::BTreeSet<_> = cases
        .iter()
        .map(|case| case["name"].as_str().unwrap())
        .collect();
    assert_eq!(seen, registered);
}
use crate::{
    model::{
        attempt_journal::{AttemptJournalRecord, MutationOutcomeRecord, MutationReceiptRequest},
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
    },
    test_support::ic_snapshot_upload::{
        DESTINATION_ID, SOURCE_ID, TARGET, raw, source_plan, unhex, upload_plan,
    },
};
use ic_management_canister_types::UploadCanisterSnapshotMetadataResult;

#[derive(candid::CandidType, serde::Deserialize)]
struct ExtendedUploadReply {
    snapshot_id: Vec<u8>,
    extra: u64,
}

#[test]
fn maximum_globals_and_exact_source_evidence_bind_without_optional_coercion_or_decoder_extensions()
{
    let source_plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let bytes = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&request, &bytes).unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"tree");
    let upload = IcSnapshotUploadRequest::metadata(&source_plan, &metadata, &checksum).unwrap();
    let alternate_raw = candid::encode_one(metadata.metadata()).unwrap();
    let alternate = IcSnapshotMetadataReply::decode(&request, &alternate_raw).unwrap();
    let alternate_upload =
        IcSnapshotUploadRequest::metadata(&source_plan, &alternate, &checksum).unwrap();
    assert_ne!(metadata.digest(), alternate.digest());
    assert_eq!(upload.digest(), alternate_upload.digest());
    assert_ne!(upload.binding_digest(), alternate_upload.binding_digest());
    let other_plan = crate::test_support::ic_snapshot_upload::plan(&"12".repeat(32));
    let other = IcSnapshotUploadRequest::metadata(&other_plan, &metadata, &checksum).unwrap();
    assert_eq!(upload.digest(), other.digest());
    assert_ne!(upload.binding_digest(), other.binding_digest());
    let mut values = metadata.metadata().clone();
    values.globals = vec![values.globals[4].clone(); 4096];
    let raw = candid::encode_one(values).unwrap();
    let maximum = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let maximum = IcSnapshotUploadRequest::metadata(&source_plan, &maximum, &checksum).unwrap();
    let args: UploadCanisterSnapshotMetadataArgs = candid::decode_one(maximum.arguments()).unwrap();
    assert_eq!(args.globals.len(), 4096);
    assert!(maximum.arguments().len() <= MAX_IC_SNAPSHOT_UPLOAD_ARGUMENT_BYTES);
    let extra = candid::encode_one(ExtendedUploadReply {
        snapshot_id: DESTINATION_ID.to_vec(),
        extra: u64::MAX,
    })
    .unwrap();
    assert!(matches!(
        IcSnapshotUploadReply::decode(&upload, &extra),
        Err(IcSnapshotUploadError::InvalidReply)
    ));
    // Seventeen real type-table entries exceed the decoder's quota before value admission.
    let mut table = b"DIDL".to_vec();
    table.push(17);
    for _ in 0..17 {
        table.extend_from_slice(&[0x6c, 0]);
    }
    table.extend_from_slice(&[1, 0]);
    assert!(matches!(
        IcSnapshotUploadReply::decode(&upload, &table),
        Err(IcSnapshotUploadError::InvalidReply)
    ));
}

#[test]
fn exact_sdk_metadata_globals_bits_options_and_source_bound_intent() {
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"declared exact artifact tree");
    for name in ["upload-source", "upload-source-absent"] {
        let bytes = raw(name);
        let metadata = IcSnapshotMetadataReply::decode(&request, &bytes).unwrap();
        let upload = IcSnapshotUploadRequest::metadata(&plan, &metadata, &checksum).unwrap();
        let values: UploadCanisterSnapshotMetadataArgs =
            candid::decode_one(upload.arguments()).unwrap();
        let original = metadata.metadata();
        assert_eq!(values.canister_id.to_text(), TARGET);
        assert!(values.replace_snapshot.is_none());
        assert_eq!(
            (
                values.wasm_module_size,
                values.wasm_memory_size,
                values.stable_memory_size
            ),
            (
                original.wasm_module_size,
                original.wasm_memory_size,
                original.stable_memory_size
            )
        );
        assert_eq!(
            candid::encode_one(values.globals).unwrap(),
            candid::encode_one(
                original
                    .globals
                    .iter()
                    .cloned()
                    .collect::<Option<Vec<_>>>()
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            candid::encode_one(values.global_timer).unwrap(),
            candid::encode_one(&original.global_timer).unwrap()
        );
        assert_eq!(
            candid::encode_one(values.on_low_wasm_memory_hook_status).unwrap(),
            candid::encode_one(&original.on_low_wasm_memory_hook_status).unwrap()
        );
        assert_eq!(values.certified_data, original.certified_data);
        assert_eq!(upload.receiver(), "aaaaa-aa");
        assert_eq!(upload.method(), "upload_canister_snapshot_metadata");
        let other = IcSnapshotUploadRequest::metadata(
            &plan,
            &metadata,
            &ArtifactChecksumRecord::from_bytes(b"different tree"),
        )
        .unwrap();
        assert_eq!(other.digest(), upload.digest());
        assert_ne!(other.binding_digest(), upload.binding_digest());
        assert!(!format!("{upload:?}").contains("snapshot_id"));
    }
    let bytes = raw("data-source");
    let missing = IcSnapshotMetadataReply::decode(&request, &bytes).unwrap();
    assert!(matches!(
        IcSnapshotUploadRequest::metadata(&plan, &missing, &checksum),
        Err(IcSnapshotUploadError::UnavailableGlobal)
    ));
}

#[test]
fn every_region_known_chunks_zero_chunks_and_maximum_arguments_keep_exact_bytes() {
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let bytes = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&request, &bytes).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"tree"),
    )
    .unwrap();
    for kind in [
        SnapshotDataKind::WasmModule { offset: 5, size: 3 },
        SnapshotDataKind::WasmMemory {
            offset: 17,
            size: 3,
        },
        SnapshotDataKind::StableMemory {
            offset: u64::MAX - 3,
            size: 3,
        },
        SnapshotDataKind::WasmChunk {
            hash: metadata.metadata().wasm_chunk_store[0].hash.clone(),
        },
    ] {
        let data = IcSnapshotUploadRequest::data(&upload, DESTINATION_ID, kind, SOURCE_ID).unwrap();
        let args: UploadCanisterSnapshotDataArgs = candid::decode_one(data.arguments()).unwrap();
        assert_eq!(args.chunk, SOURCE_ID);
        assert_eq!(args.snapshot_id, DESTINATION_ID);
        assert_eq!(args.canister_id.to_text(), TARGET);
        assert_eq!(data.method(), "upload_canister_snapshot_data");
        assert_eq!(data.source_checksum(), upload.source_checksum());
        assert!(
            matches!(data.kind(), IcSnapshotUploadKind::Data { metadata_request, .. } if *metadata_request == upload.digest())
        );
    }
    let empty = IcSnapshotUploadRequest::data(
        &upload,
        DESTINATION_ID,
        SnapshotDataKind::WasmChunk {
            hash: metadata.metadata().wasm_chunk_store[1].hash.clone(),
        },
        &[],
    )
    .unwrap();
    let args: UploadCanisterSnapshotDataArgs = candid::decode_one(empty.arguments()).unwrap();
    assert_eq!(args.chunk, [] as [u8; 0]);
    assert!(matches!(
        IcSnapshotUploadRequest::data(
            &empty,
            DESTINATION_ID,
            SnapshotDataKind::StableMemory { offset: 0, size: 1 },
            &[1]
        ),
        Err(IcSnapshotUploadError::MetadataRequestRequired)
    ));
    let large = vec![42; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES];
    let data = IcSnapshotUploadRequest::data(
        &upload,
        &[21; MAX_IC_SNAPSHOT_ID_BYTES],
        SnapshotDataKind::StableMemory {
            offset: 0,
            size: large.len() as u64,
        },
        &large,
    )
    .unwrap();
    assert!(data.arguments().len() > MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES);
    assert!(data.arguments().len() <= MAX_IC_SNAPSHOT_UPLOAD_ARGUMENT_BYTES);
    let args: UploadCanisterSnapshotDataArgs = candid::decode_one(data.arguments()).unwrap();
    assert_eq!(args.chunk, large);
}

#[test]
fn invalid_ranges_destinations_chunk_hashes_lengths_and_source_targets_reject() {
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let bytes = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&request, &bytes).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"tree"),
    )
    .unwrap();
    for (id, expected) in [(&[][..], false), (&[21; 257][..], false), (SOURCE_ID, true)] {
        let error = IcSnapshotUploadRequest::data(
            &upload,
            id,
            SnapshotDataKind::WasmModule { offset: 0, size: 1 },
            &[1],
        )
        .unwrap_err();
        assert!(if expected {
            matches!(error, IcSnapshotUploadError::DestinationReusesSource)
        } else {
            matches!(error, IcSnapshotUploadError::InvalidDestination)
        });
    }
    for kind in [
        SnapshotDataKind::WasmModule { offset: 0, size: 0 },
        SnapshotDataKind::WasmMemory {
            offset: 63,
            size: 2,
        },
        SnapshotDataKind::StableMemory {
            offset: u64::MAX,
            size: 1,
        },
        SnapshotDataKind::WasmModule {
            offset: 0,
            size: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64 + 1,
        },
        SnapshotDataKind::WasmChunk { hash: vec![42; 32] },
    ] {
        assert!(matches!(
            IcSnapshotUploadRequest::data(&upload, DESTINATION_ID, kind, &[1]),
            Err(IcSnapshotUploadError::Data(_))
        ));
    }
    assert!(matches!(
        IcSnapshotUploadRequest::data(
            &upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmModule { offset: 0, size: 2 },
            &[1]
        ),
        Err(IcSnapshotUploadError::ChunkLengthMismatch)
    ));
    assert!(matches!(
        IcSnapshotUploadRequest::data(
            &upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmChunk {
                hash: metadata.metadata().wasm_chunk_store[0].hash.clone()
            },
            &[1, 2, 3]
        ),
        Err(IcSnapshotUploadError::Checksum(_))
    ));
    assert!(matches!(
        IcSnapshotUploadRequest::data(
            &upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmChunk {
                hash: metadata.metadata().wasm_chunk_store[0].hash.clone()
            },
            &vec![0; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES + 1]
        ),
        Err(IcSnapshotUploadError::ChunkTooLarge)
    ));
    let other_request = IcSnapshotMetadataRequest::new("aaaaa-aa", SOURCE_ID).unwrap();
    let other = IcSnapshotMetadataReply::decode(&other_request, &bytes).unwrap();
    assert!(matches!(
        IcSnapshotUploadRequest::metadata(&plan, &other, upload.source_checksum()),
        Err(IcSnapshotUploadError::SourceTargetMismatch)
    ));
}

#[test]
fn bounded_reply_identity_exact_empty_tuple_and_wire_extensions_reject() {
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let bytes = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&request, &bytes).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"tree"),
    )
    .unwrap();
    for id in [DESTINATION_ID.to_vec(), vec![21; 256]] {
        let raw = candid::encode_one(UploadCanisterSnapshotMetadataResult {
            snapshot_id: id.clone(),
        })
        .unwrap();
        let reply = IcSnapshotUploadReply::decode(&upload, &raw).unwrap();
        assert_eq!(
            reply.kind(),
            &IcSnapshotUploadReplyKind::Metadata { snapshot_id: id }
        );
        assert_eq!(
            reply.payload_checksum(),
            &ArtifactChecksumRecord::from_bytes(&raw)
        );
        assert_ne!(reply.digest(), upload.binding_digest());
    }
    for id in [vec![], vec![1; 257], SOURCE_ID.to_vec()] {
        let raw =
            candid::encode_one(UploadCanisterSnapshotMetadataResult { snapshot_id: id }).unwrap();
        assert!(IcSnapshotUploadReply::decode(&upload, &raw).is_err());
    }
    let data = IcSnapshotUploadRequest::data(
        &upload,
        DESTINATION_ID,
        SnapshotDataKind::WasmModule { offset: 0, size: 1 },
        &[42],
    )
    .unwrap();
    assert_eq!(
        IcSnapshotUploadReply::decode(&data, b"DIDL\0\0")
            .unwrap()
            .kind(),
        &IcSnapshotUploadReplyKind::DataAcknowledgement
    );
    for raw in [
        b"DIDL\0\0\0".to_vec(),
        candid::encode_one(()).unwrap(),
        vec![0; 4097],
    ] {
        assert!(IcSnapshotUploadReply::decode(&data, &raw).is_err());
    }
    let raw = candid::encode_one(UploadCanisterSnapshotMetadataResult {
        snapshot_id: DESTINATION_ID.to_vec(),
    })
    .unwrap();
    assert!(IcSnapshotUploadReply::decode(&upload, &[raw.as_slice(), &[0]].concat()).is_err());
    assert!(
        IcSnapshotUploadReply::decode(
            &upload,
            &candid::encode_args((
                UploadCanisterSnapshotMetadataResult {
                    snapshot_id: DESTINATION_ID.to_vec()
                },
                1_u64
            ))
            .unwrap()
        )
        .is_err()
    );
}

#[test]
fn original_full_source_intent_pending_mutation_and_independent_data_spending_are_required() {
    let source_plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let raw = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &source_plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"tree"),
    )
    .unwrap();
    let plan = upload_plan(&upload);
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    assert!(matches!(
        IcSnapshotUploadAttempt::new(&plan, 7, &journal, &upload),
        Err(IcSnapshotUploadAttemptError::NoPendingMutation)
    ));
    let mutation = journal.reserve_mutation().unwrap();
    let original = journal.clone();
    let attempt = IcSnapshotUploadAttempt::new(&plan, 7, &journal, &upload).unwrap();
    assert_eq!(attempt.mutation_attempt(), mutation);
    let different = IcSnapshotUploadRequest::metadata(
        &source_plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"changed tree"),
    )
    .unwrap();
    assert!(matches!(
        IcSnapshotUploadAttempt::new(&plan, 7, &journal, &different),
        Err(IcSnapshotUploadAttemptError::PayloadMismatch)
    ));
    let data = IcSnapshotUploadRequest::data(
        &upload,
        DESTINATION_ID,
        SnapshotDataKind::WasmModule { offset: 0, size: 1 },
        &[42],
    )
    .unwrap();
    assert!(matches!(
        IcSnapshotUploadAttempt::new(&plan, 7, &journal, &data),
        Err(IcSnapshotUploadAttemptError::PayloadMismatch)
    ));
    let data_plan = upload_plan(&data);
    let mut data_journal = AttemptJournalRecord::new(data_plan.attempt_authority(7).unwrap());
    data_journal.reserve_mutation().unwrap();
    assert!(matches!(
        IcSnapshotUploadAttempt::new(&plan, 7, &data_journal, &upload),
        Err(IcSnapshotUploadAttemptError::AuthorityMismatch)
    ));
    assert!(matches!(
        IcSnapshotUploadAttempt::new(&plan, 8, &journal, &upload),
        Err(IcSnapshotUploadAttemptError::Plan(_))
    ));
    IcSnapshotUploadAttempt::new(&data_plan, 7, &data_journal, &data).unwrap();
    assert_eq!(journal, original);
    journal
        .reserve_observation(mutation, &"01".repeat(32))
        .unwrap();
    assert!(matches!(
        attempt.validate_journal(&journal),
        Err(IcSnapshotUploadAttemptError::ObservationPending)
    ));
    let mut settled = original;
    settled
        .record_mutation(MutationReceiptRequest {
            attempt: mutation,
            request: upload.binding_digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: "34".repeat(32),
        })
        .unwrap();
    assert!(matches!(
        attempt.validate_journal(&settled),
        Err(IcSnapshotUploadAttemptError::MutationMismatch)
    ));
    assert_eq!(settled.view().mutations_remaining, 0);
    let mut changed = serde_json::to_value(&plan).unwrap();
    changed["context"]["network"] = serde_json::json!("12".repeat(32));
    let changed: OperationPlanRecord = serde_json::from_value(changed).unwrap();
    let mut changed_journal = AttemptJournalRecord::new(changed.attempt_authority(7).unwrap());
    changed_journal.reserve_mutation().unwrap();
    assert!(matches!(
        IcSnapshotUploadAttempt::new(&changed, 7, &changed_journal, &upload),
        Err(IcSnapshotUploadAttemptError::SourceContextMismatch)
    ));
}
