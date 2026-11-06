//! Complete upload execution, cancellation and refusal over real retained sources.
use super::*;

#[derive(Clone, Copy, PartialEq)]
enum Failure {
    None,
    LostSecond,
    MalformedSecond,
    QualificationSecond,
    WrongReceiptSecond,
    Admission,
    SourceAfterFirst,
    SourceReplacedAfterFirst,
    OccupiedCheckpoint,
}
impl Failure {
    fn prepare(self, stage: &ExecutionStageGuard<'_>) {
        if self == Self::OccupiedCheckpoint {
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
    }

    fn admit(self) -> io::Result<()> {
        if self == Self::Admission {
            Err(io::Error::other("fresh admission refused"))
        } else {
            Ok(())
        }
    }

    fn qualify(
        self,
        source: &DownloadJournalGuard<'_>,
        request: &IcSnapshotUploadAttempt<'_, '_>,
        ack: &IcMutationAcknowledgement,
        qualified: usize,
    ) -> io::Result<MutationReceiptRequest> {
        if self == Failure::QualificationSecond && qualified == 2 {
            return Err(io::Error::other("original reply retention refused"));
        }
        let digest = IcSnapshotUploadReply::decode(request.payload(), &ack.input().reply)
            .unwrap()
            .digest();
        if self == Failure::SourceAfterFirst {
            fs::write(
                source
                    .layout
                    .root()
                    .join(format!("artifacts/{TARGET}/wasm-module.bin")),
                [1; 5],
            )
            .unwrap();
        }
        if self == Failure::SourceReplacedAfterFirst {
            let path = source
                .layout
                .root()
                .join(format!("artifacts/{TARGET}/wasm-module.bin"));
            let original = fs::read(&path).unwrap();
            fs::rename(&path, source.layout.root().join("original-wasm-module.bin")).unwrap();
            fs::write(&path, original).unwrap();
        }
        Ok(MutationReceiptRequest {
            attempt: request.mutation_attempt(),
            request: request.payload().binding_digest().hash().into(),
            outcome: if self == Failure::WrongReceiptSecond && qualified == 2 {
                MutationOutcomeRecord::NotApplied
            } else {
                MutationOutcomeRecord::Applied
            },
            evidence: digest.hash().into(),
        })
    }

    fn assert_result(
        self,
        result: Result<
            ExecutionStagePredecessorRecord,
            IcSnapshotDataUploadExecutionError<io::Error>,
        >,
        stage: &ExecutionStageGuard<'_>,
        operations: usize,
    ) -> ExpectedProgress {
        match self {
            Failure::MalformedSecond => {
                assert!(matches!(result,
                    Err(IcSnapshotDataUploadExecutionError::Upload(
                        IcSnapshotUploadExecutionError::AfterReply { acknowledgement, .. }
                    )) if acknowledgement.input().reply.is_empty()
                ));
                ExpectedProgress {
                    mutations_used: 2,
                    applied_operations: 1,
                    provider_calls: 2,
                }
            }
            Failure::LostSecond => {
                assert!(matches!(
                    result,
                    Err(IcSnapshotDataUploadExecutionError::Upload(
                        IcSnapshotUploadExecutionError::Provider(_)
                    ))
                ));
                ExpectedProgress {
                    mutations_used: 2,
                    applied_operations: 1,
                    provider_calls: 2,
                }
            }
            Failure::QualificationSecond => {
                assert!(
                    matches!(result, Err(IcSnapshotDataUploadExecutionError::AfterReply {
                        operation_sequence: 1,
                        source: IcSnapshotDataUploadReplyError::Qualification(_),
                        acknowledgement,
                    }) if acknowledgement.input().reply == crate::model::ic_lifecycle_reply::EMPTY_CANDID_REPLY)
                );
                ExpectedProgress {
                    mutations_used: 2,
                    applied_operations: 1,
                    provider_calls: 2,
                }
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
                ExpectedProgress {
                    mutations_used: 2,
                    applied_operations: 1,
                    provider_calls: 2,
                }
            }
            Failure::Admission => {
                assert!(result.is_err());
                ExpectedProgress {
                    mutations_used: 1,
                    applied_operations: 0,
                    provider_calls: 0,
                }
            }
            Failure::SourceAfterFirst | Failure::SourceReplacedAfterFirst => {
                assert!(matches!(
                    result,
                    Err(IcSnapshotDataUploadExecutionError::Source(_))
                ));
                ExpectedProgress {
                    mutations_used: 1,
                    applied_operations: 1,
                    provider_calls: 1,
                }
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
                ExpectedProgress {
                    mutations_used: u32::try_from(operations).unwrap(),
                    applied_operations: operations,
                    provider_calls: operations,
                }
            }
            Failure::None => unreachable!(),
        }
    }
}

struct ExpectedProgress {
    mutations_used: u32,
    applied_operations: usize,
    provider_calls: usize,
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
            let reply = if self.failure == Failure::MalformedSecond && self.calls == 2 {
                vec![]
            } else {
                crate::model::ic_lifecycle_reply::EMPTY_CANDID_REPLY.to_vec()
            };
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
        let progress =
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
        assert_eq!(progress.applied_operations, data.kinds().len());
        assert_eq!(progress.attempts.observations_used, 0);
        assert_not_reissued(stage, data, source, &mut provider);
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

fn journal_bytes(stage: &ExecutionStageGuard<'_>, operations: usize) -> Vec<Vec<u8>> {
    (0..operations)
        .map(|sequence| {
            fs::read(
                stage
                    .layout()
                    .unwrap()
                    .root()
                    .join(format!("attempt-{sequence}.json")),
            )
            .unwrap()
        })
        .collect()
}

fn assert_not_reissued(
    stage: &ExecutionStageGuard<'_>,
    data: &IcSnapshotDataUploadPlan<'_, '_, '_>,
    source: &DownloadJournalGuard<'_>,
    provider: &mut Provider,
) {
    let before = journal_bytes(stage, data.kinds().len());
    let calls = provider.calls;
    assert!(matches!(
        ready(upload_snapshot_data(
            stage,
            data,
            source,
            TOKEN,
            provider,
            async |_| -> Result<(), io::Error> { panic!("consumed stage cannot reissue") },
            async |_, _| -> Result<MutationReceiptRequest, io::Error> {
                panic!("no new settlement")
            },
        )),
        Err(IcSnapshotDataUploadExecutionError::AlreadyAttempted)
    ));
    assert_eq!(journal_bytes(stage, data.kinds().len()), before);
    assert_eq!(provider.calls, calls);
}

#[test]
fn failures_stop_before_successors_and_preserve_pending_applied_and_occupied_evidence() {
    for failure in [
        Failure::LostSecond,
        Failure::MalformedSecond,
        Failure::QualificationSecond,
        Failure::WrongReceiptSecond,
        Failure::Admission,
        Failure::SourceAfterFirst,
        Failure::SourceReplacedAfterFirst,
        Failure::OccupiedCheckpoint,
    ] {
        with_data_stage(|stage, data, source| {
            let source_before = fs::read(source.path()).unwrap();
            let references = stage.layout().unwrap().restore_references().unwrap();
            failure.prepare(stage);
            let mut provider = Provider { calls: 0, failure };
            let mut qualified = 0;
            let result = ready(upload_snapshot_data(
                stage,
                data,
                source,
                TOKEN,
                &mut provider,
                async |_| failure.admit(),
                async |request, ack| {
                    qualified += 1;
                    failure.qualify(source, request, ack, qualified)
                },
            ));
            let expected = failure.assert_result(result, stage, data.kinds().len());
            let progress =
                read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
            assert_eq!(progress.attempts.mutations_used, expected.mutations_used);
            assert_eq!(progress.applied_operations, expected.applied_operations);
            assert_eq!(provider.calls, expected.provider_calls);
            assert_not_reissued(stage, data, source, &mut provider);
            assert_eq!(fs::read(source.path()).unwrap(), source_before);
            assert_eq!(
                stage.layout().unwrap().restore_references().unwrap(),
                references
            );
        });
    }
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

pub(super) fn complete_data(
    stage: &ExecutionStageGuard<'_>,
    data: &IcSnapshotDataUploadPlan<'_, '_, '_>,
    source: &DownloadJournalGuard<'_>,
) -> ExecutionStagePredecessorRecord {
    let mut provider = Provider {
        calls: 0,
        failure: Failure::None,
    };
    ready(upload_snapshot_data(
        stage,
        data,
        source,
        TOKEN,
        &mut provider,
        async |_| Ok::<_, io::Error>(()),
        async |request, acknowledgement| {
            let reply =
                IcSnapshotUploadReply::decode(request.payload(), &acknowledgement.input().reply)
                    .unwrap();
            Ok(MutationReceiptRequest {
                attempt: request.mutation_attempt(),
                request: request.payload().binding_digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: reply.digest().hash().into(),
            })
        },
    ))
    .unwrap()
}
