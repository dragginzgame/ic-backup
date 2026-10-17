//! Real retained-source IO and original journal failures; no IC effect simulation.
use super::*;
use crate::model::ic_update_response::IcUpdateResponseInput;
use crate::test_support::ready::ready;
use crate::{
    model::{
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_mutation::IcMutationAcknowledgement,
        ic_snapshot_upload::{
            IcSnapshotDataUploadPlan, IcSnapshotDataUploadPlanningError, IcSnapshotUploadAttempt,
            IcSnapshotUploadReply,
        },
    },
    ops::persistence::{
        AttemptJournalGuard, ExecutionStageGuard, IcSnapshotDataUploadPreparationError,
        create_execution_workflow, read_execution_progress,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
    test_support::ic_snapshot_upload::{data_workflow, unhex, upload_plan},
    workflow::ic_snapshot_upload::{
        IcSnapshotDataUploadExecutionError, IcSnapshotDataUploadReplyError, upload_snapshot_data,
    },
};
use std::io;

fn with_data_stage(
    check: impl FnOnce(
        &ExecutionStageGuard<'_>,
        &IcSnapshotDataUploadPlan<'_, '_, '_>,
        &DownloadJournalGuard<'_>,
    ),
) {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let original = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(upload_values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let source = retained(&layout, &original, &metadata, &raw);
    let payload = source
        .prepare_ic_snapshot_upload_metadata(&original, TOKEN, &metadata)
        .unwrap();
    let workflow = data_workflow(&payload, 12);
    let path = root.join("workflow");
    fs::create_dir(&path).unwrap();
    let workflow_layout = BackupLayoutGuard::acquire(&path).unwrap();
    create_execution_workflow(&workflow_layout, &workflow).unwrap();
    let mut fields = serde_json::to_value(upload_plan(&payload)).unwrap();
    fields["operations"][0]["operation_sequence"] = serde_json::json!(0);
    fields["graph"]["nodes"][0]["operation_sequence"] = serde_json::json!(0);
    fields["operations"][0]["budget"]["observations"] = serde_json::json!(0);
    fields["budget"]["observations"] = serde_json::json!(0);
    let plan = serde_json::from_value(fields).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&workflow_layout, binding, plan).unwrap();
    let raw = unhex("4449444c026c01b6b897890f016d7b0100031500ff");
    let allocation = IcSnapshotUploadReply::decode(&payload, &raw).unwrap();
    // Local receipt/persistence setup only; authentic IC attribution is tested separately.
    let mut journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(0).unwrap(),
    )
    .unwrap();
    journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt: 1,
            request: payload.binding_digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: allocation.digest().hash().into(),
        })
        .unwrap();
    drop(journal);
    let predecessor = stage.reconcile_checkpoint(allocation.digest()).unwrap();
    let data = source
        .prepare_ic_snapshot_data_upload_plan(&workflow, 7, TOKEN, &allocation, 4)
        .unwrap();
    let binding = data
        .bind(stage.binding(), stage.plan(), vec![predecessor])
        .unwrap();
    drop(stage);
    let stage =
        ExecutionStageGuard::prepare(&workflow_layout, binding, data.plan().unwrap().clone())
            .unwrap();
    stage
        .layout()
        .unwrap()
        .retain_restore(
            &root.join("unfinished-upload.json"),
            original.digest().hash(),
        )
        .unwrap();
    check(&stage, &data, &source);
}
#[derive(Clone, Copy, PartialEq)]
enum Failure {
    None,
    LostSecond,
    QualificationSecond,
    WrongReceiptSecond,
    Admission,
    SourceAfterFirst,
    OccupiedCheckpoint,
}
struct Provider {
    calls: usize,
    failure: Failure,
}
impl IcSnapshotUploadProvider for Provider {
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
        journal: &crate::model::attempt_journal::AttemptJournalRecord,
    ) -> impl std::future::Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>>
    {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            self.calls += 1;
            if self.failure == Failure::LostSecond && self.calls == 2 {
                return Err(IcMutationProviderError::Indeterminate);
            }
            let reply = crate::model::ic_lifecycle_reply::EMPTY_CANDID_REPLY.to_vec();
            Ok(IcMutationAcknowledgement::new(IcUpdateResponseInput {
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                context: request.plan().context().clone(),
                target: request.payload().target().into(),
                evidence: ArtifactChecksumRecord::from_bytes(&reply),
                reply,
            })
            .unwrap())
        })())
    }
}
#[test]
fn complete_stage_checkpoints_exact_replies_and_rejects_whole_stage_replay() {
    with_data_stage(|stage, data, source| {
        let original = fs::read(source.path()).unwrap();
        let references = stage.layout().unwrap().restore_references().unwrap();
        let mut provider = Provider {
            calls: 0,
            failure: Failure::None,
        };
        let mut evidence = b"ic-backup/ic-snapshot-data-upload/v1\0".to_vec();
        evidence.extend_from_slice(stage.plan().digest().hash().as_bytes());
        let predecessor = ready(upload_snapshot_data(
            stage,
            data,
            source,
            TOKEN,
            &mut provider,
            async |_| Ok::<_, io::Error>(()),
            async |request, ack| {
                let digest = IcSnapshotUploadReply::decode(request.payload(), &ack.input().reply)
                    .unwrap()
                    .digest();
                evidence.extend_from_slice(digest.hash().as_bytes());
                Ok(MutationReceiptRequest {
                    attempt: request.mutation_attempt(),
                    request: request.payload().binding_digest().hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: digest.hash().into(),
                })
            },
        ))
        .unwrap();
        assert_eq!(
            predecessor.learned_evidence(),
            &ArtifactChecksumRecord::from_bytes(&evidence)
        );
        assert_eq!(provider.calls, data.kinds().len());
        let before: Vec<_> = (0..provider.calls)
            .map(|i| {
                fs::read(
                    stage
                        .layout()
                        .unwrap()
                        .root()
                        .join(format!("attempt-{i}.json")),
                )
                .unwrap()
            })
            .collect();
        let progress =
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
        assert_eq!(progress.applied_operations, data.kinds().len());
        assert_eq!(progress.attempts.observations_used, 0);
        assert!(matches!(
            ready(upload_snapshot_data(
                stage,
                data,
                source,
                TOKEN,
                &mut provider,
                async |_| -> Result<(), io::Error> { panic!("Applied cannot reissue") },
                async |_, _| -> Result<MutationReceiptRequest, io::Error> {
                    panic!("Applied cannot resettle")
                }
            )),
            Err(IcSnapshotDataUploadExecutionError::AlreadyAttempted)
        ));
        for (i, original) in before.iter().enumerate() {
            assert_eq!(
                &fs::read(
                    stage
                        .layout()
                        .unwrap()
                        .root()
                        .join(format!("attempt-{i}.json"))
                )
                .unwrap(),
                original
            );
        }
        assert_eq!(fs::read(source.path()).unwrap(), original);
        assert_eq!(
            stage.layout().unwrap().restore_references().unwrap(),
            references
        );
    });
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one original interruption trace retaining source, receipts and exact request/reply evidence"
)]
fn cancelled_second_write_qualification_retains_first_receipt_pending_reply_and_source() {
    use std::{
        io::Write,
        task::{Context, Poll, Waker},
    };
    with_data_stage(|stage, data, source| {
        let original_source = fs::read(source.path()).unwrap();
        let original_references = stage.layout().unwrap().restore_references().unwrap();
        let before = source
            .prepare_ic_snapshot_upload_metadata(
                data.metadata().source_plan(),
                TOKEN,
                data.metadata().source(),
            )
            .unwrap()
            .binding_digest();
        let mut provider = Provider {
            calls: 0,
            failure: Failure::None,
        };
        let root = stage.layout().unwrap().root().to_path_buf();
        let mut future = Box::pin(upload_snapshot_data(
            stage,
            data,
            source,
            TOKEN,
            &mut provider,
            async |_| Ok::<(), io::Error>(()),
            async |request, acknowledgement| {
                let sequence = request.authority().binding().operation_sequence();
                for (label, bytes) in [
                    ("request", request.payload().arguments()),
                    ("reply", acknowledgement.input().reply.as_slice()),
                ] {
                    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(root.join(format!("retained-{label}-{sequence}.candid")))?;
                    file.write_all(bytes)?;
                    file.sync_all()?;
                }
                fs::File::open(&root)?.sync_all()?;
                if sequence == 1 {
                    std::future::pending::<()>().await;
                }
                Ok(MutationReceiptRequest {
                    attempt: request.mutation_attempt(),
                    request: request.payload().binding_digest().hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: acknowledgement.input().evidence.hash().into(),
                })
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
        let path = root.join("attempt-1.json");
        let pending_original = fs::read(&path).unwrap();
        let retained_reply = fs::read(root.join("retained-reply-1.candid")).unwrap();
        let retained_request = fs::read(root.join("retained-request-1.candid")).unwrap();
        drop(future);
        assert_eq!(provider.calls, 2);
        let progress =
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
        assert_eq!(progress.applied_operations, 1);
        assert_eq!(progress.attempts.mutations_used, 2);
        let journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
        assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
        drop(journal);
        assert!(
            ready(upload_snapshot_data(
                stage,
                data,
                source,
                TOKEN,
                &mut provider,
                async |_| Ok::<(), io::Error>(()),
                async |_, _| -> Result<MutationReceiptRequest, io::Error> { panic!("no reentry") }
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 2);
        assert_eq!(fs::read(path).unwrap(), pending_original);
        assert_eq!(
            fs::read(root.join("retained-reply-1.candid")).unwrap(),
            retained_reply
        );
        assert_eq!(
            fs::read(root.join("retained-request-1.candid")).unwrap(),
            retained_request
        );
        assert_eq!(fs::read(source.path()).unwrap(), original_source);
        assert_eq!(
            stage.layout().unwrap().restore_references().unwrap(),
            original_references
        );
        let after = source
            .prepare_ic_snapshot_upload_metadata(
                data.metadata().source_plan(),
                TOKEN,
                data.metadata().source(),
            )
            .unwrap()
            .binding_digest();
        assert_eq!(after, before);
        assert!(
            stage
                .reconcile_checkpoint(ArtifactChecksumRecord::from_bytes(b"not complete"))
                .is_err()
        );
    });
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one original-source/journal chronology compares the six independent stop and preservation boundaries"
)]
fn failures_stop_before_successors_and_preserve_pending_applied_and_occupied_evidence() {
    for failure in [
        Failure::LostSecond,
        Failure::QualificationSecond,
        Failure::WrongReceiptSecond,
        Failure::Admission,
        Failure::SourceAfterFirst,
        Failure::OccupiedCheckpoint,
    ] {
        with_data_stage(|stage, data, source| {
            let source_before = fs::read(source.path()).unwrap();
            let references = stage.layout().unwrap().restore_references().unwrap();
            if failure == Failure::OccupiedCheckpoint {
                fs::write(
                    stage
                        .layout()
                        .unwrap()
                        .root()
                        .join("execution-settlement.json"),
                    b"occupied original checkpoint",
                )
                .unwrap();
            }
            let mut provider = Provider { calls: 0, failure };
            let mut qualified = 0;
            let result = ready(upload_snapshot_data(
                stage,
                data,
                source,
                TOKEN,
                &mut provider,
                async |_| {
                    if failure == Failure::Admission {
                        Err(io::Error::other("fresh admission refused"))
                    } else {
                        Ok(())
                    }
                },
                async |request, ack| {
                    qualified += 1;
                    if failure == Failure::QualificationSecond && qualified == 2 {
                        return Err(io::Error::other("original reply retention refused"));
                    }
                    let digest =
                        IcSnapshotUploadReply::decode(request.payload(), &ack.input().reply)
                            .unwrap()
                            .digest();
                    if failure == Failure::SourceAfterFirst {
                        fs::write(
                            source
                                .layout
                                .root()
                                .join(format!("artifacts/{TARGET}/wasm-module.bin")),
                            [1; 5],
                        )
                        .unwrap();
                    }
                    Ok(MutationReceiptRequest {
                        attempt: request.mutation_attempt(),
                        request: request.payload().binding_digest().hash().into(),
                        outcome: if failure == Failure::WrongReceiptSecond && qualified == 2 {
                            MutationOutcomeRecord::NotApplied
                        } else {
                            MutationOutcomeRecord::Applied
                        },
                        evidence: digest.hash().into(),
                    })
                },
            ));
            let (used, applied, calls) = match failure {
                Failure::LostSecond => {
                    assert!(matches!(result,Err(IcSnapshotDataUploadExecutionError::Upload(crate::workflow::ic_snapshot_upload::IcSnapshotUploadExecutionError::Provider(_)))));
                    (2, 1, 2)
                }
                Failure::QualificationSecond => {
                    assert!(
                        matches!(result,Err(IcSnapshotDataUploadExecutionError::AfterReply{operation_sequence:1,source:IcSnapshotDataUploadReplyError::Qualification(_),acknowledgement}) if acknowledgement.input().reply==crate::model::ic_lifecycle_reply::EMPTY_CANDID_REPLY)
                    );
                    (2, 1, 2)
                }
                Failure::WrongReceiptSecond => {
                    assert!(matches!(
                        result,
                        Err(IcSnapshotDataUploadExecutionError::AfterReply {
                            operation_sequence: 1,
                            source: IcSnapshotDataUploadReplyError::ReceiptRequired,
                            ..
                        })
                    ));
                    (2, 1, 2)
                }
                Failure::Admission => {
                    assert!(result.is_err());
                    (1, 0, 0)
                }
                Failure::SourceAfterFirst => {
                    assert!(matches!(
                        result,
                        Err(IcSnapshotDataUploadExecutionError::Source(_))
                    ));
                    (1, 1, 1)
                }
                Failure::OccupiedCheckpoint => {
                    assert!(matches!(
                        result,
                        Err(IcSnapshotDataUploadExecutionError::Checkpoint(_))
                    ));
                    assert_eq!(
                        fs::read(
                            stage
                                .layout()
                                .unwrap()
                                .root()
                                .join("execution-settlement.json")
                        )
                        .unwrap(),
                        b"occupied original checkpoint"
                    );
                    (
                        u32::try_from(data.kinds().len()).unwrap(),
                        data.kinds().len(),
                        data.kinds().len(),
                    )
                }
                Failure::None => unreachable!(),
            };
            let progress =
                read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
            assert_eq!(progress.attempts.mutations_used, used);
            assert_eq!(progress.applied_operations, applied);
            assert_eq!(provider.calls, calls);
            let before: Vec<_> = (0..data.kinds().len())
                .map(|i| {
                    fs::read(
                        stage
                            .layout()
                            .unwrap()
                            .root()
                            .join(format!("attempt-{i}.json")),
                    )
                    .unwrap()
                })
                .collect();
            assert!(matches!(
                ready(upload_snapshot_data(
                    stage,
                    data,
                    source,
                    TOKEN,
                    &mut provider,
                    async |_| -> Result<(), io::Error> { panic!("consumed stage cannot reissue") },
                    async |_, _| -> Result<MutationReceiptRequest, io::Error> {
                        panic!("no new settlement")
                    }
                )),
                Err(IcSnapshotDataUploadExecutionError::AlreadyAttempted)
            ));
            for (i, original) in before.iter().enumerate() {
                assert_eq!(
                    &fs::read(
                        stage
                            .layout()
                            .unwrap()
                            .root()
                            .join(format!("attempt-{i}.json"))
                    )
                    .unwrap(),
                    original
                );
            }
            assert_eq!(provider.calls, calls);
            assert_eq!(fs::read(source.path()).unwrap(), source_before);
            assert_eq!(
                stage.layout().unwrap().restore_references().unwrap(),
                references
            );
        });
    }
}
#[test]
fn insufficient_allocation_refuses_before_any_source_verification_or_journal_changes() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let original = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(upload_values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let source = retained(&layout, &original, &metadata, &raw);
    let payload = source
        .prepare_ic_snapshot_upload_metadata(&original, TOKEN, &metadata)
        .unwrap();
    let reply = IcSnapshotUploadReply::decode(
        &payload,
        &unhex("4449444c026c01b6b897890f016d7b0100031500ff"),
    )
    .unwrap();
    let workflow: ExecutionWorkflowRecord = data_workflow(&payload, 1);
    let before = fs::read(source.path()).unwrap();
    fs::rename(
        root.join(format!("artifacts/{TARGET}")),
        root.join("retained-missing-tree"),
    )
    .unwrap();
    assert!(matches!(
        source.prepare_ic_snapshot_data_upload_plan(&workflow, 7, TOKEN, &reply, 4),
        Err(IcSnapshotDataUploadPreparationError::Planning(
            IcSnapshotDataUploadPlanningError::InsufficientAllowance {
                required: 6,
                original: 1
            }
        ))
    ));
    assert_eq!(fs::read(source.path()).unwrap(), before);
}

#[test]
fn missing_original_journal_refuses_without_creating_or_dispatching() {
    with_data_stage(|stage, data, source| {
        let root = stage.layout().unwrap().root();
        let path = root.join("attempt-1.json");
        let retained = root.join("retained-attempt-1.json");
        fs::rename(&path, &retained).unwrap();
        let bytes = fs::read(&retained).unwrap();
        let mut provider = Provider {
            calls: 0,
            failure: Failure::None,
        };
        assert!(
            ready(upload_snapshot_data(
                stage,
                data,
                source,
                TOKEN,
                &mut provider,
                async |_| -> Result<(), io::Error> {
                    panic!("missing originals grant no admission")
                },
                async |_, _| -> Result<MutationReceiptRequest, io::Error> {
                    panic!("missing originals grant no receipt")
                }
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 0);
        assert!(!path.exists());
        assert_eq!(fs::read(retained).unwrap(), bytes);
        assert!(source.record().unwrap().resume_view().is_complete);
    });
}

#[test]
fn batch_bindings_match_canonical_payloads_and_preserve_retained_records() {
    with_data_stage(|_stage, data, source| {
        let before = fs::read(source.path()).unwrap();
        let bindings: Vec<_> = data
            .kinds()
            .iter()
            .map(|kind| {
                source
                    .prepare_ic_snapshot_upload_data(
                        data.metadata().source_plan(),
                        TOKEN,
                        data.metadata(),
                        data.destination(),
                        kind.clone(),
                    )
                    .unwrap()
                    .binding_digest()
            })
            .collect();
        // Each original operation names the exact separately prepared payload.
        assert_eq!(data.plan().unwrap().operations().len(), bindings.len());
        for (operation, binding) in data.plan().unwrap().operations().iter().zip(&bindings) {
            assert_eq!(operation.request(), binding.hash());
        }
        assert_eq!(fs::read(source.path()).unwrap(), before);
    });
}

#[test]
fn batch_plans_empty_regions_and_known_empty_chunks_without_placeholder_writes() {
    for known_empty in [false, true] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let plan = source_plan();
        create_operation_plan(&layout, &plan).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let mut values = upload_values();
        values.wasm_module_size = 0;
        values.wasm_memory_size = 0;
        values.stable_memory_size = 0;
        let empty = values.wasm_chunk_store.pop().unwrap();
        values.wasm_chunk_store = if known_empty { vec![empty] } else { vec![] };
        let raw = candid::encode_one(values).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut source = DownloadJournalGuard::create(
            &layout,
            plan.digest().hash(),
            vec![DownloadArtifactRequest {
                canister_id: TARGET.into(),
                snapshot_id: TOKEN.into(),
                snapshot_taken_at_timestamp: metadata.metadata().taken_at_timestamp,
                snapshot_total_size_bytes: u64::MAX,
            }],
        )
        .unwrap();
        let writer = source
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let writer = if known_empty {
            append(
                writer,
                SnapshotDataKind::WasmChunk {
                    hash: metadata.metadata().wasm_chunk_store[0].hash.clone(),
                },
                vec![],
            )
            .unwrap()
        } else {
            writer
        };
        writer.finish().unwrap();
        let before = fs::read(source.path()).unwrap();
        let upload = source
            .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
            .unwrap();
        let workflow = data_workflow(&upload, 12);
        let wire = unhex("4449444c026c01b6b897890f016d7b0100031500ff");
        let allocation = IcSnapshotUploadReply::decode(&upload, &wire).unwrap();
        let data = source
            .prepare_ic_snapshot_data_upload_plan(&workflow, 7, TOKEN, &allocation, 4)
            .unwrap();
        assert_eq!(data.kinds().len(), usize::from(known_empty));
        assert_eq!(data.plan().is_some(), known_empty);
        if let Some(plan) = data.plan() {
            let payload = source
                .prepare_ic_snapshot_upload_data(
                    upload.source_plan(),
                    TOKEN,
                    &upload,
                    DESTINATION_ID,
                    data.kinds()[0].clone(),
                )
                .unwrap();
            assert_eq!(
                plan.operations()[0].request(),
                payload.binding_digest().hash()
            );
        }
        assert_eq!(fs::read(source.path()).unwrap(), before);
    }
}
