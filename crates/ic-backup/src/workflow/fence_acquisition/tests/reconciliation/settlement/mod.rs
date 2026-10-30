//! Explicit application qualification and one original receipt, without provider effects.

use super::{Fixture, Provider, admit, challenge, evidence, pending, ready};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{
            MutationOutcomeRecord, MutationReceiptRequest, ObservationOutcomeRecord,
            ObservationReceiptRequest,
        },
        fence_obligation::FenceObligationScopeRecord,
        fence_reconciliation::{FenceReconciliationEvidence, FenceReconciliationObservation},
    },
    ops::persistence::{AttemptJournalGuard, ExecutionStageGuard},
    workflow::fence_reconciliation::{
        FenceReconciliationSettlementError, observe_fence_acquisition,
        settle_fence_acquisition_observation,
    },
};
use std::{convert::Infallible, fs};

fn observe(fixture: &Fixture, stage: &ExecutionStageGuard<'_>) -> FenceReconciliationObservation {
    pending(fixture, stage, true);
    let mut provider = Provider::new(None);
    let observation = ready(observe_fence_acquisition(
        stage,
        fixture.requirement(),
        &fixture.obligation,
        &fixture.payload,
        challenge(),
        &mut provider,
        admit,
    ))
    .unwrap();
    assert_eq!(provider.calls, 1);
    observation
}
fn receipt(
    observation: &FenceReconciliationObservation,
    outcome: ObservationOutcomeRecord,
) -> ObservationReceiptRequest {
    let input = observation.input();
    ObservationReceiptRequest {
        attempt: input.observation_attempt,
        request: input.request.hash().into(),
        outcome,
        evidence: input.evidence.hash().into(),
    }
}
fn applied(
    fixture: &Fixture,
    observation: &FenceReconciliationObservation,
) -> FenceReconciliationObservation {
    let mut input = observation.input().clone();
    input.settlement = match fixture.obligation.scope() {
        FenceObligationScopeRecord::Capture {
            identity,
            membership_revision,
            ..
        } => {
            let mut fence = crate::test_support::consistency::fence();
            fence.identity = identity.clone();
            fence.membership_revision = membership_revision.clone();
            FenceReconciliationEvidence::AcquiredCapture {
                fence: Box::new(fence),
                attribution: challenge(),
            }
        }
        FenceObligationScopeRecord::Restore {
            fence: original, ..
        } => {
            let mut fence = crate::test_support::restore_safety::fence();
            fence.binding = original.clone();
            FenceReconciliationEvidence::AcquiredRestore {
                fence: Box::new(fence),
                attribution: challenge(),
            }
        }
    };
    FenceReconciliationObservation::new(input).unwrap()
}

#[test]
fn exact_qualified_outcomes_survive_reopen_without_refunds_reentry_or_release() {
    for restore in [false, true] {
        for outcome in [
            ObservationOutcomeRecord::Applied,
            ObservationOutcomeRecord::NotApplied,
            ObservationOutcomeRecord::Uncertain,
        ] {
            let fixture = Fixture::new(restore, false);
            let stage = fixture.stage();
            let mut observation = observe(&fixture, &stage);
            if outcome == ObservationOutcomeRecord::Applied {
                observation = applied(&fixture, &observation);
            }
            if outcome == ObservationOutcomeRecord::Uncertain {
                let mut input = observation.input().clone();
                input.settlement = FenceReconciliationEvidence::Unresolved {
                    uncertainty: challenge(),
                };
                observation = FenceReconciliationObservation::new(input).unwrap();
            }
            let references = fixture.source_layout.restore_references().unwrap();
            let obligation =
                fs::read(stage.layout().unwrap().root().join("fence-obligation.json")).unwrap();
            drop(stage);
            let stage = fixture.stage();
            let actual = ready(settle_fence_acquisition_observation(
                &stage,
                fixture.requirement(),
                &fixture.obligation,
                &fixture.payload,
                challenge(),
                &observation,
                async |request, observation| {
                    assert!(
                        AttemptJournalGuard::open(
                            stage.layout().unwrap(),
                            request.intent().authority()
                        )
                        .is_err()
                    );
                    Ok::<_, Infallible>(receipt(observation, outcome))
                },
            ))
            .unwrap();
            assert_eq!(actual, outcome);
            let guard = AttemptJournalGuard::open(
                stage.layout().unwrap(),
                &fixture.plan.attempt_authority(42).unwrap(),
            )
            .unwrap();
            let view = guard.record().unwrap().view();
            assert_eq!((view.mutations_used, view.observations_used), (1, 1));
            assert_eq!(
                (view.pending_mutation, view.pending_observation),
                (
                    (outcome == ObservationOutcomeRecord::Uncertain).then_some(1),
                    None
                )
            );
            assert_eq!(view.mutations_remaining, 0);
            assert_eq!(view.applied, outcome == ObservationOutcomeRecord::Applied);
            let bytes = fs::read(guard.path()).unwrap();
            drop(guard);
            assert!(
                ready(settle_fence_acquisition_observation(
                    &stage,
                    fixture.requirement(),
                    &fixture.obligation,
                    &fixture.payload,
                    challenge(),
                    &observation,
                    async |_, _| -> Result<ObservationReceiptRequest, Infallible> {
                        panic!("settled receipt requalified")
                    }
                ))
                .is_err()
            );
            assert_eq!(
                fs::read(stage.layout().unwrap().root().join("attempt-42.json")).unwrap(),
                bytes
            );
            assert_eq!(
                fixture.source_layout.restore_references().unwrap(),
                references
            );
            assert_eq!(
                fs::read(stage.layout().unwrap().root().join("fence-obligation.json")).unwrap(),
                obligation
            );
            assert!(
                !stage
                    .layout()
                    .unwrap()
                    .root()
                    .join("execution-settlement.json")
                    .exists()
            );
        }
    }
}

#[test]
fn qualification_and_every_receipt_mismatch_preserve_both_pending_attempts() {
    for mismatch in 0..5 {
        let fixture = Fixture::new(true, false);
        let stage = fixture.stage();
        let observation = observe(&fixture, &stage);
        let before = evidence(&stage, &fixture);
        let result = ready(settle_fence_acquisition_observation(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            challenge(),
            &observation,
            async |_, observation| {
                if mismatch == 0 {
                    return Err(std::io::Error::other("qualification refused"));
                }
                let mut receipt = receipt(observation, ObservationOutcomeRecord::NotApplied);
                match mismatch {
                    1 => receipt.attempt += 1,
                    2 => receipt.request = challenge().hash().into(),
                    3 => receipt.outcome = ObservationOutcomeRecord::Applied,
                    _ => receipt.evidence = challenge().hash().into(),
                }
                Ok(receipt)
            },
        ));
        if mismatch == 0 {
            assert!(matches!(
                result,
                Err(FenceReconciliationSettlementError::Qualification(_))
            ));
        } else {
            assert!(matches!(
                result,
                Err(FenceReconciliationSettlementError::ReceiptMismatch)
            ));
        }
        assert_eq!(evidence(&stage, &fixture), before);
    }
}

fn satisfy_prerequisite(fixture: &Fixture, stage: &ExecutionStageGuard<'_>) {
    let authority = fixture.plan.attempt_authority(43).unwrap();
    let mut journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    let attempt = journal
        .reserve_planned_mutation(&fixture.plan.digest())
        .unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: authority.binding().request().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: challenge().hash().into(),
        })
        .unwrap();
}

#[test]
fn complete_original_coverage_is_rechecked_before_and_after_qualification() {
    for during in [false, true] {
        let fixture = Fixture::new(false, true);
        let stage = fixture.stage();
        satisfy_prerequisite(&fixture, &stage);
        let observation = observe(&fixture, &stage);
        let before = evidence(&stage, &fixture);
        let path = stage.layout().unwrap().root().join("attempt-43.json");
        let retained = path.with_extension("retained-original");
        if !during {
            fs::rename(&path, &retained).unwrap();
        }
        let mut calls = 0;
        let result = ready(settle_fence_acquisition_observation(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            challenge(),
            &observation,
            async |_, observation| {
                calls += 1;
                fs::rename(&path, &retained).unwrap();
                Ok::<_, Infallible>(receipt(observation, ObservationOutcomeRecord::NotApplied))
            },
        ));
        assert!(matches!(
            result,
            Err(FenceReconciliationSettlementError::Progress(_))
        ));
        assert_eq!(calls, usize::from(during));
        assert_eq!(evidence(&stage, &fixture), before);
        assert!(retained.is_file());
        assert!(!path.exists());
    }
}

#[test]
fn wrong_original_challenge_and_changed_source_cannot_record_an_observation() {
    for source_change in [false, true] {
        let fixture = Fixture::new(true, false);
        let stage = fixture.stage();
        let observation = observe(&fixture, &stage);
        let before = evidence(&stage, &fixture);
        let mut calls = 0;
        let result = ready(settle_fence_acquisition_observation(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            if source_change {
                challenge()
            } else {
                ArtifactChecksumRecord::from_bytes(b"another challenge")
            },
            &observation,
            async |_, observation| {
                calls += 1;
                fs::write(
                    fixture.source_layout.root().join("operation-plan.json"),
                    b"{}",
                )
                .unwrap();
                Ok::<_, Infallible>(receipt(observation, ObservationOutcomeRecord::NotApplied))
            },
        ));
        assert!(result.is_err());
        assert_eq!(calls, usize::from(source_change));
        assert_eq!(evidence(&stage, &fixture), before);
    }
}

#[test]
fn cancelled_non_send_qualification_keeps_exclusion_spending_and_references() {
    use std::rc::Rc;
    let fixture = Fixture::new(true, false);
    let stage = fixture.stage();
    let observation = observe(&fixture, &stage);
    let before = evidence(&stage, &fixture);
    let references = fixture.source_layout.restore_references().unwrap();
    let local = Rc::new(());
    let mut future = Box::pin(settle_fence_acquisition_observation(
        &stage,
        fixture.requirement(),
        &fixture.obligation,
        &fixture.payload,
        challenge(),
        &observation,
        async |_, _| {
            std::future::pending::<()>().await;
            drop(local);
            Ok::<_, Infallible>(receipt(&observation, ObservationOutcomeRecord::NotApplied))
        },
    ));
    crate::test_support::ready::assert_pending(future.as_mut());
    assert!(
        AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &fixture.plan.attempt_authority(42).unwrap()
        )
        .is_err()
    );
    drop(future);
    assert_eq!(evidence(&stage, &fixture), before);
    assert_eq!(
        fixture.source_layout.restore_references().unwrap(),
        references
    );
}

#[test]
fn failed_publication_preserves_original_journal_and_borrowed_observation() {
    let fixture = Fixture::new(false, false);
    let stage = fixture.stage();
    let observation = observe(&fixture, &stage);
    let before = evidence(&stage, &fixture);
    let path = stage.layout().unwrap().root().join("attempt-42.json");
    let retained = path.with_extension("retained-original");
    let result = ready(settle_fence_acquisition_observation(
        &stage,
        fixture.requirement(),
        &fixture.obligation,
        &fixture.payload,
        challenge(),
        &observation,
        async |_, observation| {
            fs::rename(&path, &retained).unwrap();
            fs::create_dir(&path).unwrap();
            Ok::<_, Infallible>(receipt(observation, ObservationOutcomeRecord::NotApplied))
        },
    ));
    assert!(matches!(
        result,
        Err(FenceReconciliationSettlementError::Progress(_))
    ));
    assert_eq!(fs::read(retained).unwrap(), before);
    assert!(path.is_dir());
    assert_eq!(
        observation.input().evidence,
        ArtifactChecksumRecord::from_bytes(b"original native observation reply")
    );
}
