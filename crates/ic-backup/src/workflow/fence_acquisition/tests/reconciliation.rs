//! Reuse original capture/restore fixtures to qualify one observation without settlement.

use super::{AfterReplyFault, Fixture, OriginalFault, Purpose, ready};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        fence_acquisition::FenceAcquisitionPayload,
        fence_reconciliation::{
            FenceReconciliationEvidence, FenceReconciliationObservation,
            FenceReconciliationObservationInput, FenceReconciliationRequest,
        },
    },
    ops::persistence::{AttemptJournalGuard, ExecutionStageGuard},
    ports::fence_reconciliation::{FenceReconciliationProvider, FenceReconciliationProviderError},
    workflow::fence_reconciliation::{
        FenceReconciliationExecutionError, FenceReconciliationReplyError, observe_fence_acquisition,
    },
};
use std::{convert::Infallible, fs, path::PathBuf};

mod settlement;

fn challenge() -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_bytes(b"fresh native observation challenge")
}
fn admit(_: &FenceReconciliationRequest<'_, '_>) -> impl Future<Output = Result<(), Infallible>> {
    std::future::ready(Ok(()))
}
fn pending(fixture: &Fixture, stage: &ExecutionStageGuard<'_>, planned: bool) {
    let mut journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &fixture.plan.attempt_authority(42).unwrap(),
    )
    .unwrap();
    if planned {
        journal
            .reserve_planned_mutation(&fixture.plan.digest())
            .unwrap();
    } else {
        journal.reserve_mutation().unwrap();
    }
}
fn evidence(stage: &ExecutionStageGuard<'_>, fixture: &Fixture) -> Vec<u8> {
    let journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &fixture.plan.attempt_authority(42).unwrap(),
    )
    .unwrap();
    let view = journal.record().unwrap().view();
    assert_eq!(view.mutations_used, 1);
    assert!(view.pending_mutation.is_some());
    assert_eq!(view.observations_used, 1);
    assert!(view.pending_observation.is_some());
    assert!(!view.applied);
    fs::read(journal.path()).unwrap()
}
struct Provider {
    calls: usize,
    failure: Option<FenceReconciliationProviderError>,
    mismatch: bool,
    change: Option<PathBuf>,
    replace: Option<PathBuf>,
    unresolved: bool,
}
impl Provider {
    const fn new(failure: Option<FenceReconciliationProviderError>) -> Self {
        Self {
            calls: 0,
            failure,
            mismatch: false,
            change: None,
            replace: None,
            unresolved: false,
        }
    }
}
impl FenceReconciliationProvider for Provider {
    fn observe_fence_acquisition(
        &mut self,
        request: &FenceReconciliationRequest<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> impl Future<Output = Result<FenceReconciliationObservation, FenceReconciliationProviderError>>
    {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            self.calls += 1;
            if let Some(error) = self.failure {
                return Err(error);
            }
            let observation =
                FenceReconciliationObservation::new(FenceReconciliationObservationInput {
                    request: if self.mismatch {
                        challenge()
                    } else {
                        request.digest()
                    },
                    mutation_attempt: request.intent().mutation_attempt(),
                    observation_attempt: request.observation_attempt(),
                    context: request.intent().plan().context().clone(),
                    inventory: request.intent().plan().inventory().clone(),
                    selected_targets: request.intent().plan().selected_targets().to_vec(),
                    settlement: if self.unresolved {
                        FenceReconciliationEvidence::Unresolved {
                            uncertainty: challenge(),
                        }
                    } else {
                        FenceReconciliationEvidence::NotAcquired {
                            exclusion: challenge(),
                        }
                    },
                    evidence: ArtifactChecksumRecord::from_bytes(
                        b"original native observation reply",
                    ),
                    remote_observations: 1,
                })
                .unwrap();
            if let Some(path) = &self.change {
                fs::write(path, b"{}").unwrap();
            }
            if let Some(path) = &self.replace {
                fs::rename(path, path.with_extension("retained-original")).unwrap();
                fs::create_dir(path).unwrap();
            }
            Ok(observation)
        })())
    }
}

#[test]
fn both_purposes_match_passive_claims_and_preserve_failures_without_settlement_or_reentry() {
    for purpose in Purpose::ALL {
        for failure in [
            None,
            Some(FenceReconciliationProviderError::Unavailable),
            Some(FenceReconciliationProviderError::Unsupported),
            Some(FenceReconciliationProviderError::Indeterminate),
        ] {
            let fixture = Fixture::new(purpose, false);
            let stage = fixture.stage();
            pending(&fixture, &stage, true);
            let originals =
                fs::read(stage.layout().unwrap().root().join("fence-obligation.json")).unwrap();
            let references = fixture.source_layout.restore_references().unwrap();
            let mut provider = Provider::new(failure);
            provider.unresolved = purpose == Purpose::Restore;
            let result = ready(observe_fence_acquisition(
                &stage,
                fixture.requirement(),
                &fixture.obligation,
                &fixture.payload,
                challenge(),
                &mut provider,
                async |request| {
                    assert!(
                        AttemptJournalGuard::open(
                            stage.layout().unwrap(),
                            request.intent().authority()
                        )
                        .is_err()
                    );
                    Ok::<(), Infallible>(())
                },
            ));
            match (failure, result) {
                (None, Ok(observation)) => assert_eq!(
                    observation.input().evidence,
                    ArtifactChecksumRecord::from_bytes(b"original native observation reply")
                ),
                (Some(expected), Err(FenceReconciliationExecutionError::Provider(actual))) => {
                    assert_eq!(actual, expected);
                }
                other => panic!("unexpected observation result: {other:?}"),
            }
            let before = evidence(&stage, &fixture);
            assert!(
                ready(observe_fence_acquisition(
                    &stage,
                    fixture.requirement(),
                    &fixture.obligation,
                    &fixture.payload,
                    ArtifactChecksumRecord::from_bytes(b"new challenge"),
                    &mut provider,
                    admit
                ))
                .is_err()
            );
            assert_eq!(provider.calls, 1);
            assert_eq!(evidence(&stage, &fixture), before);
            assert_eq!(
                fixture.source_layout.restore_references().unwrap(),
                references
            );
            assert_eq!(
                fs::read(stage.layout().unwrap().root().join("fence-obligation.json")).unwrap(),
                originals
            );
        }
    }
}

#[test]
fn changed_payload_missing_originals_and_no_mutation_refuse_before_observation_spending() {
    for boundary in [
        OriginalFault::ChangedPayload,
        OriginalFault::MissingObligation,
        OriginalFault::MissingJournal,
        OriginalFault::NoPendingMutation,
    ] {
        let fixture = Fixture::new(Purpose::Capture, false);
        let stage = fixture.stage();
        if boundary != OriginalFault::NoPendingMutation {
            pending(&fixture, &stage, true);
        }
        let root = stage.layout().unwrap().root();
        let before = fs::read(root.join("attempt-42.json")).unwrap();
        let changed = FenceAcquisitionPayload::new(
            fixture.payload.target(),
            "changed",
            fixture.payload.arguments(),
        )
        .unwrap();
        if boundary == OriginalFault::MissingObligation || boundary == OriginalFault::MissingJournal
        {
            let name = if boundary == OriginalFault::MissingObligation {
                "fence-obligation.json"
            } else {
                "attempt-42.json"
            };
            fs::rename(root.join(name), root.join(format!("retained-{name}"))).unwrap();
        }
        let mut provider = Provider::new(None);
        assert!(
            ready(observe_fence_acquisition(
                &stage,
                fixture.requirement(),
                &fixture.obligation,
                if boundary == OriginalFault::ChangedPayload {
                    &changed
                } else {
                    &fixture.payload
                },
                challenge(),
                &mut provider,
                async |_| -> Result<(), Infallible> {
                    panic!("invalid originals reached admission")
                }
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 0);
        assert_eq!(
            fs::read(root.join(if boundary == OriginalFault::MissingJournal {
                "retained-attempt-42.json"
            } else {
                "attempt-42.json"
            }))
            .unwrap(),
            before
        );
    }
}

#[test]
fn complete_original_coverage_and_applied_prerequisite_are_required() {
    for missing in [false, true] {
        let fixture = Fixture::new(Purpose::Capture, true);
        let stage = fixture.stage();
        pending(&fixture, &stage, false);
        let root = stage.layout().unwrap().root();
        let before = fs::read(root.join("attempt-42.json")).unwrap();
        if missing {
            fs::rename(
                root.join("attempt-43.json"),
                root.join("retained-attempt-43.json"),
            )
            .unwrap();
        }
        let mut provider = Provider::new(None);
        assert!(
            ready(observe_fence_acquisition(
                &stage,
                fixture.requirement(),
                &fixture.obligation,
                &fixture.payload,
                challenge(),
                &mut provider,
                admit
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 0);
        assert_eq!(fs::read(root.join("attempt-42.json")).unwrap(), before);
    }
}

#[test]
fn admission_failure_and_changed_requirement_keep_observation_pending_without_dispatch() {
    for change in [false, true] {
        let fixture = Fixture::new(Purpose::Capture, false);
        let stage = fixture.stage();
        pending(&fixture, &stage, true);
        let mut provider = Provider::new(None);
        let result = ready(observe_fence_acquisition(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            challenge(),
            &mut provider,
            async |_| {
                if change {
                    fs::write(
                        stage
                            .layout()
                            .unwrap()
                            .root()
                            .join("consistency-requirement.json"),
                        b"{}",
                    )
                    .unwrap();
                    Ok(())
                } else {
                    Err(std::io::Error::other("native admission refusal"))
                }
            },
        ));
        assert!(result.is_err());
        assert_eq!(provider.calls, 0);
        evidence(&stage, &fixture);
    }
}

struct SuspendedProvider {
    calls: usize,
}
impl FenceReconciliationProvider for SuspendedProvider {
    async fn observe_fence_acquisition(
        &mut self,
        request: &FenceReconciliationRequest<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> Result<FenceReconciliationObservation, FenceReconciliationProviderError> {
        request.validate_journal(journal).unwrap();
        self.calls += 1;
        std::future::pending().await
    }
}
#[test]
fn cancellation_during_admission_or_observation_holds_exclusion_and_retains_originals() {
    for suspend_admission in [false, true] {
        let fixture = Fixture::new(Purpose::Restore, false);
        let stage = fixture.stage();
        pending(&fixture, &stage, true);
        let references = fixture.source_layout.restore_references().unwrap();
        let mut provider = SuspendedProvider { calls: 0 };
        let mut future = Box::pin(observe_fence_acquisition(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            challenge(),
            &mut provider,
            async |_| {
                if suspend_admission {
                    std::future::pending::<()>().await;
                }
                Ok::<(), Infallible>(())
            },
        ));
        crate::tests::support::ready::assert_pending(future.as_mut());
        assert!(
            AttemptJournalGuard::open(
                stage.layout().unwrap(),
                &fixture.plan.attempt_authority(42).unwrap()
            )
            .is_err()
        );
        let before = fs::read(stage.layout().unwrap().root().join("attempt-42.json")).unwrap();
        drop(future);
        assert_eq!(provider.calls, usize::from(!suspend_admission));
        assert_eq!(evidence(&stage, &fixture), before);
        assert!(
            ready(observe_fence_acquisition(
                &stage,
                fixture.requirement(),
                &fixture.obligation,
                &fixture.payload,
                challenge(),
                &mut provider,
                admit
            ))
            .is_err()
        );
        assert_eq!(
            fixture.source_layout.restore_references().unwrap(),
            references
        );
    }
}

#[test]
fn after_reply_association_stage_and_obligation_errors_keep_returned_evidence() {
    for boundary in [
        AfterReplyFault::Association,
        AfterReplyFault::Stage,
        AfterReplyFault::Obligation,
    ] {
        let fixture = Fixture::new(Purpose::Capture, false);
        let stage = fixture.stage();
        pending(&fixture, &stage, true);
        let root = stage.layout().unwrap().root();
        let mut provider = Provider::new(None);
        provider.mismatch = boundary == AfterReplyFault::Association;
        if boundary != AfterReplyFault::Association {
            provider.change = Some(root.join(match boundary {
                AfterReplyFault::Stage => "stage-binding.json",
                AfterReplyFault::Obligation => "fence-obligation.json",
                AfterReplyFault::Association => {
                    unreachable!("association does not replace evidence")
                }
            }));
        }
        let error = ready(observe_fence_acquisition(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            challenge(),
            &mut provider,
            admit,
        ))
        .unwrap_err();
        let observation = match error {
            FenceReconciliationExecutionError::AfterReply {
                source: FenceReconciliationReplyError::Association(_),
                observation,
                ..
            } if boundary == AfterReplyFault::Association => observation,
            FenceReconciliationExecutionError::AfterReply {
                source: FenceReconciliationReplyError::Stage(_),
                observation,
                ..
            } if boundary == AfterReplyFault::Stage => observation,
            FenceReconciliationExecutionError::AfterReply {
                source: FenceReconciliationReplyError::Obligation(_),
                observation,
                ..
            } if boundary == AfterReplyFault::Obligation => observation,
            other => panic!("unexpected post-reply failure: {other:?}"),
        };
        assert_eq!(
            observation.input().evidence,
            ArtifactChecksumRecord::from_bytes(b"original native observation reply")
        );
        assert_eq!(provider.calls, 1);
    }
}

#[test]
fn replaced_restore_layout_retains_original_observation_and_source_references() {
    let fixture = Fixture::new(Purpose::Restore, false);
    let stage = fixture.stage();
    pending(&fixture, &stage, true);
    let root = stage.layout().unwrap().root().to_path_buf();
    let references = fixture.source_layout.restore_references().unwrap();
    let mut provider = Provider::new(None);
    provider.replace = Some(root.clone());
    let error = ready(observe_fence_acquisition(
        &stage,
        fixture.requirement(),
        &fixture.obligation,
        &fixture.payload,
        challenge(),
        &mut provider,
        admit,
    ))
    .unwrap_err();
    let FenceReconciliationExecutionError::AfterReply {
        source: FenceReconciliationReplyError::Journal(_),
        observation,
        ..
    } = error
    else {
        panic!("unexpected replacement rejection")
    };
    assert_eq!(
        observation.input().evidence,
        ArtifactChecksumRecord::from_bytes(b"original native observation reply")
    );
    let original: AttemptJournalRecord = serde_json::from_slice(
        &fs::read(
            root.with_extension("retained-original")
                .join("attempt-42.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(original.view().mutations_used, 1);
    assert_eq!(original.view().observations_used, 1);
    assert!(original.view().pending_mutation.is_some());
    assert!(original.view().pending_observation.is_some());
    assert_eq!(
        fixture.source_layout.restore_references().unwrap(),
        references
    );
}
