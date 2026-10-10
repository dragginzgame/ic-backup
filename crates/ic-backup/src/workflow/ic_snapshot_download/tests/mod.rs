//! Complete original streaming, explicit receipts and retained failed transfer evidence.

use super::*;
use crate::test_support::ready::ready;
use crate::{
    model::{
        download_journal::{ArtifactStateRecord, DownloadArtifactRequest},
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_snapshot_download::tests::{source, source_plan, values, workflow},
        ic_snapshot_metadata::IcSnapshotMetadataReply,
        ic_snapshot_transfer_read::IcSnapshotTransferReadResponseInput,
    },
    ops::persistence::{BackupLayoutGuard, DownloadJournalGuard, create_execution_workflow},
    ports::ic_observation::IcObservationProviderError,
    test_support::temp_dir,
};
use ic_management_canister_types::{ReadCanisterSnapshotDataResult, SnapshotDataKind};
use std::{convert::Infallible, fs, io, path::PathBuf};

const TOKEN: &str = "qualified-original";

fn prepare(
    workflow: &ExecutionWorkflowRecord,
    metadata: &IcSnapshotMetadataReply<'_>,
    download: &IcSnapshotDownloadPlan<'_, '_>,
) -> (PathBuf, BackupLayoutGuard, ExecutionStageBindingRecord) {
    let root = temp_dir("ic-backup-complete-download");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, workflow).unwrap();
    let original = source_plan(metadata.request());
    let binding = ExecutionStageBindingRecord::new(workflow, 0, &original, vec![]).unwrap();
    let metadata_stage =
        ExecutionStageGuard::prepare(&layout, binding.clone(), original.clone()).unwrap();
    let mut journal = AttemptJournalGuard::open(
        metadata_stage.layout().unwrap(),
        &original.attempt_authority(42).unwrap(),
    )
    .unwrap();
    journal
        .reserve_planned_mutation(&original.digest())
        .unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt: 1,
            request: metadata.request().digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: metadata.digest().hash().into(),
        })
        .unwrap();
    drop(journal);
    let predecessor = metadata_stage.checkpoint(metadata.digest()).unwrap();
    drop(metadata_stage);
    let binding = download
        .bind(&binding, &original, vec![predecessor])
        .unwrap();
    let stage =
        ExecutionStageGuard::prepare(&layout, binding.clone(), download.plan().unwrap().clone())
            .unwrap();
    fs::create_dir(stage.layout().unwrap().root().join("artifacts")).unwrap();
    stage
        .layout()
        .unwrap()
        .retain_restore(
            &root.join("unfinished-restore.json"),
            stage.plan().digest().hash(),
        )
        .unwrap();
    drop(stage);
    (root, layout, binding)
}
fn artifacts<'a>(
    stage: &'a ExecutionStageGuard<'_>,
    metadata: &IcSnapshotMetadataReply<'_>,
    intent: &str,
) -> DownloadJournalGuard<'a> {
    DownloadJournalGuard::create(
        stage.layout().unwrap(),
        intent,
        vec![DownloadArtifactRequest {
            canister_id: metadata.request().target().into(),
            snapshot_id: TOKEN.into(),
            snapshot_taken_at_timestamp: metadata.metadata().taken_at_timestamp,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap()
}
struct Provider {
    calls: usize,
    failure: Option<(usize, bool)>,
}
impl IcSnapshotTransferReadProvider for Provider {
    fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &crate::model::attempt_journal::AttemptJournalRecord,
    ) -> impl std::future::Future<
        Output = Result<IcSnapshotTransferReadResponse, IcObservationProviderError>,
    > {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            let index = self.calls;
            self.calls += 1;
            let IcSnapshotTransferReadPayload::Data(payload) = request.payload() else {
                panic!("data only")
            };
            let chunk = match payload.kind() {
                SnapshotDataKind::WasmModule { size, .. }
                | SnapshotDataKind::WasmMemory { size, .. }
                | SnapshotDataKind::StableMemory { size, .. } => {
                    vec![7; usize::try_from(*size).unwrap()]
                }
                SnapshotDataKind::WasmChunk { hash }
                    if *hash == payload.metadata().metadata().wasm_chunk_store[0].hash =>
                {
                    vec![0, 255, 17]
                }
                SnapshotDataKind::WasmChunk { .. } => vec![],
            };
            let raw = match self.failure {
                Some((failed, false)) if failed == index => {
                    return Err(IcObservationProviderError::Indeterminate);
                }
                Some((failed, true)) if failed == index => vec![],
                _ => candid::encode_one(ReadCanisterSnapshotDataResult { chunk }).unwrap(),
            };
            Ok(
                IcSnapshotTransferReadResponse::new(IcSnapshotTransferReadResponseInput {
                    authority: request.authority().digest(),
                    mutation_attempt: request.mutation_attempt(),
                    context: request.plan().context().clone(),
                    target: payload.target().into(),
                    evidence: ArtifactChecksumRecord::from_bytes(&raw),
                    reply: raw,
                })
                .unwrap(),
            )
        })())
    }
}
fn admit(
    _: &IcSnapshotTransferReadRequest<'_, '_>,
) -> impl std::future::Future<Output = Result<(), Infallible>> {
    std::future::ready(Ok(()))
}
#[expect(
    clippy::unnecessary_wraps,
    reason = "fixture qualification explicitly returns a receipt under the required fallible integration contract"
)]
fn qualify(
    request: &IcSnapshotTransferReadRequest<'_, '_>,
    response: &IcSnapshotTransferReadResponse,
) -> Result<MutationReceiptRequest, Infallible> {
    Ok(MutationReceiptRequest {
        attempt: request.mutation_attempt(),
        request: request.payload().digest().hash().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: response.input().evidence.hash().into(),
    })
}
fn raw_metadata() -> Vec<u8> {
    candid::encode_one(values()).unwrap()
}

#[test]
fn cancelled_second_qualification_retains_partial_bytes_first_receipt_and_pending_reply() {
    use std::{
        cell::Cell,
        io::Write,
        task::{Context, Poll, Waker},
    };
    let workflow = workflow(256);
    let request = source();
    let raw = raw_metadata();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
    let (root, layout, binding) = prepare(&workflow, &metadata, &download);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
    let partial = stage
        .layout()
        .unwrap()
        .root()
        .join(artifacts.record().unwrap().artifacts()[0].staging_path())
        .join("wasm-module.bin");
    let writer = artifacts
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    let mut provider = Provider {
        calls: 0,
        failure: None,
    };
    let qualified = Cell::new(0);
    let mut future = Box::pin(download_snapshot(
        &stage,
        &download,
        writer,
        &mut provider,
        admit,
        async |request, response| {
            let sequence = request.authority().binding().operation_sequence();
            let file = fs::File::create(root.join(format!("reply-{sequence}.candid"))).unwrap();
            (&file).write_all(&response.input().reply).unwrap();
            file.sync_all().unwrap();
            fs::File::open(&root).unwrap().sync_all().unwrap();
            qualified.set(qualified.get() + 1);
            if sequence == 1 {
                std::future::pending::<()>().await;
            }
            qualify(request, response)
        },
    ));
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    let authority = stage.plan().attempt_authority(1).unwrap();
    assert!(AttemptJournalGuard::open(stage.layout().unwrap(), &authority).is_err());
    let path = stage.layout().unwrap().root().join("attempt-1.json");
    let before = fs::read(&path).unwrap();
    let reply = fs::read(root.join("reply-1.candid")).unwrap();
    let original_partial = fs::read(&partial).unwrap();
    assert_eq!(original_partial, [7; 32]);
    drop(future);
    assert_eq!(provider.calls, 2);
    assert_eq!(qualified.get(), 2);
    let progress =
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
    assert_eq!(progress.applied_operations, 1);
    assert_eq!(progress.attempts.mutations_used, 2);
    let journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
    drop(journal);
    assert_eq!(
        artifacts.record().unwrap().artifacts()[0].state(),
        ArtifactStateRecord::Created
    );
    assert!(
        artifacts
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .is_err()
    );
    assert!(
        ready(read_snapshot(
            &stage,
            1,
            IcSnapshotTransferReadPayload::Data(&download.requests()[1]),
            &mut provider,
            admit,
        ))
        .is_err()
    );
    assert_eq!(provider.calls, 2);
    assert_eq!(fs::read(path).unwrap(), before);
    assert_eq!(fs::read(root.join("reply-1.candid")).unwrap(), reply);
    assert_eq!(fs::read(partial).unwrap(), original_partial);
}

#[test]
fn streams_every_exact_region_and_known_chunk_with_explicit_receipts_then_publishes() {
    let workflow = workflow(256);
    let request = source();
    let raw = raw_metadata();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
    let (root, layout, binding) = prepare(&workflow, &metadata, &download);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    let references = stage.layout().unwrap().restore_references().unwrap();
    let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
    let writer = artifacts
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    let mut provider = Provider {
        calls: 0,
        failure: None,
    };
    let checksum = ready(download_snapshot(
        &stage,
        &download,
        writer,
        &mut provider,
        admit,
        async |request, response| {
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            qualify(request, response)
        },
    ))
    .unwrap();
    assert_eq!(provider.calls, download.requests().len());
    assert_eq!(
        artifacts.record().unwrap().artifacts()[0].state(),
        ArtifactStateRecord::Durable
    );
    assert_eq!(
        artifacts
            .verify_ic_snapshot_artifact(stage.plan(), TOKEN, &metadata)
            .unwrap(),
        checksum
    );
    let manifest = artifacts.publish_download_manifest(stage.plan()).unwrap();
    drop(artifacts);
    let checkpoint = stage.checkpoint(manifest.clone()).unwrap();
    let progress =
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
    assert_eq!(progress.applied_operations, download.requests().len());
    assert_eq!(progress.attempts.mutations_remaining, 0);
    assert_eq!(
        stage.layout().unwrap().restore_references().unwrap(),
        references
    );
    assert_eq!(checkpoint.learned_evidence(), &manifest);
    drop(stage);
    let (stage, current) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    assert_eq!(current, progress);
    assert_eq!(provider.calls, download.requests().len());
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lost_or_malformed_second_read_preserves_first_receipt_partial_bytes_and_pending_original() {
    for malformed in [false, true] {
        let workflow = workflow(256);
        let request = source();
        let raw = raw_metadata();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
        let (root, layout, binding) = prepare(&workflow, &metadata, &download);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
        let references = stage.layout().unwrap().restore_references().unwrap();
        let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
        let writer = artifacts
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let path = writer.path().to_path_buf();
        let mut provider = Provider {
            calls: 0,
            failure: Some((1, malformed)),
        };
        let error = ready(download_snapshot(
            &stage,
            &download,
            writer,
            &mut provider,
            admit,
            async |request, response| qualify(request, response),
        ))
        .unwrap_err();
        if malformed {
            assert!(matches!(
                error,
                IcSnapshotDownloadExecutionError::Read(
                    IcSnapshotTransferReadExecutionError::Association { .. }
                )
            ));
        } else {
            assert!(matches!(
                error,
                IcSnapshotDownloadExecutionError::Read(
                    IcSnapshotTransferReadExecutionError::Provider(
                        IcObservationProviderError::Indeterminate
                    )
                )
            ));
        }
        assert_eq!(provider.calls, 2);
        assert_eq!(fs::read(path.join("wasm-module.bin")).unwrap(), vec![7; 32]);
        assert_eq!(
            artifacts.record().unwrap().artifacts()[0].state(),
            ArtifactStateRecord::Created
        );
        assert!(
            artifacts
                .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
                .is_err()
        );
        let progress =
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
        assert_eq!(progress.applied_operations, 1);
        assert_eq!(progress.attempts.mutations_used, 2);
        let original = fs::read(stage.layout().unwrap().root().join("attempt-1.json")).unwrap();
        assert!(
            stage
                .checkpoint(ArtifactChecksumRecord::from_bytes(b"not complete"))
                .is_err()
        );
        drop(artifacts);
        drop(stage);
        let (stage, current) =
            ExecutionStageGuard::resume(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
        assert_eq!(current, progress);
        assert_eq!(
            fs::read(stage.layout().unwrap().root().join("attempt-1.json")).unwrap(),
            original
        );
        assert_eq!(
            stage.layout().unwrap().restore_references().unwrap(),
            references
        );
        assert_eq!(provider.calls, 2);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn qualification_rejection_retains_exact_response_and_does_not_append_or_record_a_receipt() {
    let workflow = workflow(256);
    let request = source();
    let raw = raw_metadata();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
    let (root, layout, binding) = prepare(&workflow, &metadata, &download);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
    let writer = artifacts
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    let path = writer.path().to_path_buf();
    let mut provider = Provider {
        calls: 0,
        failure: None,
    };
    let error = ready(download_snapshot(
        &stage,
        &download,
        writer,
        &mut provider,
        async |_| Ok::<(), io::Error>(()),
        async |_, _| Err(io::Error::other("authentication rejected")),
    ))
    .unwrap_err();
    let IcSnapshotDownloadExecutionError::AfterReply {
        operation_sequence,
        source: IcSnapshotDownloadReplyError::Qualification(_),
        response,
    } = error
    else {
        panic!("retained qualification rejection")
    };
    assert_eq!(operation_sequence, 0);
    assert_ne!(response.input().reply, [] as [u8; 0]);
    assert_eq!(provider.calls, 1);
    assert_eq!(
        fs::read(path.join("wasm-module.bin")).unwrap(),
        [] as [u8; 0]
    );
    let progress =
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
    assert_eq!(progress.applied_operations, 0);
    assert_eq!(progress.attempts.mutations_used, 1);
    drop(artifacts);
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_receipt_or_negative_outcome_rejects_before_append_and_retains_pending_spending() {
    for variant in 0..3 {
        let workflow = workflow(256);
        let request = source();
        let raw = raw_metadata();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
        let (root, layout, binding) = prepare(&workflow, &metadata, &download);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
        let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
        let writer = artifacts
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let path = writer.path().to_path_buf();
        let mut provider = Provider {
            calls: 0,
            failure: None,
        };
        let error = ready(download_snapshot(
            &stage,
            &download,
            writer,
            &mut provider,
            admit,
            async |request, response| {
                let mut receipt = qualify(request, response)?;
                match variant {
                    0 => receipt.attempt += 1,
                    1 => receipt.request = "33".repeat(32),
                    _ => receipt.outcome = MutationOutcomeRecord::NotApplied,
                }
                Ok::<_, Infallible>(receipt)
            },
        ))
        .unwrap_err();
        assert!(matches!(
            error,
            IcSnapshotDownloadExecutionError::AfterReply {
                source: IcSnapshotDownloadReplyError::ReceiptRequired,
                ..
            }
        ));
        assert_eq!(provider.calls, 1);
        assert_eq!(
            fs::read(path.join("wasm-module.bin")).unwrap(),
            [] as [u8; 0]
        );
        let progress =
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
        assert_eq!(progress.applied_operations, 0);
        assert_eq!(progress.attempts.mutations_used, 1);
        drop(artifacts);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn consumed_missing_or_held_originals_never_enter_fresh_admission_or_provider() {
    for variant in 0..3 {
        let workflow = workflow(256);
        let request = source();
        let raw = raw_metadata();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
        let (root, layout, binding) = prepare(&workflow, &metadata, &download);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
        let authority = stage.plan().attempt_authority(0).unwrap();
        let mut held = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
        if variant == 0 {
            held.reserve_planned_mutation(&stage.plan().digest())
                .unwrap();
        }
        let held = if variant == 2 {
            Some(held)
        } else {
            drop(held);
            None
        };
        if variant == 1 {
            let path = stage.layout().unwrap().root().join("attempt-0.json");
            fs::rename(&path, path.with_extension("retained-original")).unwrap();
        }
        let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
        let writer = artifacts
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let mut provider = Provider {
            calls: 0,
            failure: None,
        };
        let error = ready(download_snapshot(
            &stage,
            &download,
            writer,
            &mut provider,
            async |_| -> Result<(), Infallible> { panic!("original rejection") },
            async |request, response| qualify(request, response),
        ))
        .unwrap_err();
        assert!(matches!(
            error,
            IcSnapshotDownloadExecutionError::AlreadyAttempted
                | IcSnapshotDownloadExecutionError::Progress(_)
        ));
        assert_eq!(provider.calls, 0);
        drop(held);
        drop(artifacts);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn stage_or_local_byte_drift_during_qualification_retains_reply_and_pending_attempt() {
    for stage_changed in [false, true] {
        let workflow = workflow(256);
        let request = source();
        let raw = raw_metadata();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
        let (root, layout, binding) = prepare(&workflow, &metadata, &download);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
        let stage_root = stage.layout().unwrap().root().to_path_buf();
        let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
        let writer = artifacts
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let path = writer.path().to_path_buf();
        let mut provider = Provider {
            calls: 0,
            failure: None,
        };
        let error = ready(download_snapshot(
            &stage,
            &download,
            writer,
            &mut provider,
            admit,
            async |request, response| {
                if stage_changed {
                    fs::write(stage_root.join("stage-binding.json"), b"{}").unwrap();
                } else {
                    fs::write(path.join("wasm-module.bin"), b"changed").unwrap();
                }
                qualify(request, response)
            },
        ))
        .unwrap_err();
        let IcSnapshotDownloadExecutionError::AfterReply {
            source, response, ..
        } = error
        else {
            panic!("retained post-reply error")
        };
        assert!(matches!(
            source,
            IcSnapshotDownloadReplyError::Stage(_) | IcSnapshotDownloadReplyError::Artifact(_)
        ));
        assert_ne!(response.input().reply, [] as [u8; 0]);
        assert_eq!(provider.calls, 1);
        let journal: crate::model::attempt_journal::AttemptJournalRecord =
            serde_json::from_slice(&fs::read(stage_root.join("attempt-0.json")).unwrap()).unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert!(!journal.view().applied);
        drop(artifacts);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn changed_writer_metadata_intent_prior_coverage_and_read_free_plan_reject_without_spending() {
    for variant in 0..4 {
        let workflow = workflow(256);
        let request = source();
        let raw = raw_metadata();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
        let (root, layout, binding) = prepare(&workflow, &metadata, &download);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
        let mut other = values();
        other.wasm_module_size += 1;
        let other_raw = candid::encode_one(other).unwrap();
        let other_metadata = IcSnapshotMetadataReply::decode(&request, &other_raw).unwrap();
        let (selected_metadata, selected_raw) = if variant == 0 {
            (&other_metadata, &other_raw)
        } else {
            (&metadata, &raw)
        };
        let intent = if variant == 1 {
            "44".repeat(32)
        } else {
            stage.plan().digest().hash().into()
        };
        let mut artifacts = artifacts(&stage, selected_metadata, &intent);
        let mut writer = artifacts
            .stage_ic_snapshot_artifact(TOKEN, selected_metadata, selected_raw)
            .unwrap();
        if variant == 2 {
            let raw =
                candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![7; 32] }).unwrap();
            let reply = crate::model::ic_snapshot_data::IcSnapshotDataReply::decode(
                &download.requests()[0],
                &raw,
            )
            .unwrap();
            writer = writer.append(&reply).unwrap();
        }
        let mut empty = values();
        empty.wasm_module_size = 0;
        empty.wasm_memory_size = 0;
        empty.stable_memory_size = 0;
        empty.wasm_chunk_store.clear();
        let empty_raw = candid::encode_one(empty).unwrap();
        let empty_metadata = IcSnapshotMetadataReply::decode(&request, &empty_raw).unwrap();
        let empty_download =
            IcSnapshotDownloadPlan::new(&workflow, 7, &empty_metadata, 32).unwrap();
        let selected = if variant == 3 {
            &empty_download
        } else {
            &download
        };
        let mut provider = Provider {
            calls: 0,
            failure: None,
        };
        let error = ready(download_snapshot(
            &stage,
            selected,
            writer,
            &mut provider,
            async |_| -> Result<(), Infallible> {
                panic!("invalid original must reject before admission")
            },
            async |request, response| qualify(request, response),
        ))
        .unwrap_err();
        assert!(matches!(
            error,
            IcSnapshotDownloadExecutionError::OriginalMismatch
                | IcSnapshotDownloadExecutionError::Artifact(_)
                | IcSnapshotDownloadExecutionError::Planning(
                    IcSnapshotDownloadPlanningError::NoDataReads
                )
        ));
        assert_eq!(provider.calls, 0);
        assert_eq!(
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest())
                .unwrap()
                .attempts
                .mutations_used,
            0
        );
        drop(artifacts);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn receipt_persistence_failure_retains_appended_bytes_reply_and_original_pending_reservation() {
    let workflow = workflow(256);
    let request = source();
    let raw = raw_metadata();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
    let (root, layout, binding) = prepare(&workflow, &metadata, &download);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    let mut artifacts = artifacts(&stage, &metadata, stage.plan().digest().hash());
    let writer = artifacts
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    let path = writer.path().to_path_buf();
    let journal_path = stage.layout().unwrap().root().join("attempt-0.json");
    let retained = journal_path.with_extension("retained-original");
    let mut provider = Provider {
        calls: 0,
        failure: None,
    };
    let error = ready(download_snapshot(
        &stage,
        &download,
        writer,
        &mut provider,
        admit,
        async |request, response| {
            fs::rename(&journal_path, &retained).unwrap();
            fs::create_dir(&journal_path).unwrap();
            qualify(request, response)
        },
    ))
    .unwrap_err();
    let IcSnapshotDownloadExecutionError::AfterReply {
        source: IcSnapshotDownloadReplyError::Journal(_),
        response,
        ..
    } = error
    else {
        panic!("retained receipt-write rejection")
    };
    assert_ne!(response.input().reply, [] as [u8; 0]);
    assert_eq!(provider.calls, 1);
    assert_eq!(fs::read(path.join("wasm-module.bin")).unwrap(), vec![7; 32]);
    assert_eq!(
        artifacts.record().unwrap().artifacts()[0].state(),
        ArtifactStateRecord::Created
    );
    let original: crate::model::attempt_journal::AttemptJournalRecord =
        serde_json::from_slice(&fs::read(retained).unwrap()).unwrap();
    assert_eq!(original.view().pending_mutation, Some(1));
    assert!(!original.view().applied);
    drop(artifacts);
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
