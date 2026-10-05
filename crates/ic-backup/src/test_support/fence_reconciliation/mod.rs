//! Passive exact pending journal and application evidence fixtures; no backend.

use crate::{
    model::{
        attempt_journal::AttemptJournalRecord, consistency::*, fence_obligation::*,
        fence_reconciliation::*, operation_plan::OperationPlanRecord,
        restore_safety::RestoreSafetyLaneRecord,
    },
    test_support::{
        consistency,
        membership::{hash, plan},
        restore_safety,
    },
};
pub struct Original {
    pub plan: OperationPlanRecord,
    pub obligation: FenceObligationRecord,
    pub journal: AttemptJournalRecord,
}
pub fn original(restore: bool) -> Original {
    let plan = plan();
    let obligation = if restore {
        let source = restore_safety::source();
        let requirement =
            restore_safety::requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
        FenceObligationRecord::for_restore(&plan, &source, &requirement, 0).unwrap()
    } else {
        FenceObligationRecord::for_capture(
            &plan,
            &ConsistencyRequirementRecord::new(
                &plan,
                ConsistencyGuaranteeRecord::ApplicationCoordinated,
            ),
            0,
            &ApplicationFenceBinding {
                identity: hash("56"),
                membership_revision: hash("78"),
            },
        )
        .unwrap()
    };
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(0).unwrap());
    journal.reserve_mutation().unwrap();
    Original {
        plan,
        obligation,
        journal,
    }
}
pub fn reserve(original: &mut Original, challenge: &str) {
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash(challenge),
    )
    .unwrap();
    original
        .journal
        .reserve_observation(intent.mutation_attempt(), intent.digest().hash())
        .unwrap();
}
pub fn input(request: &FenceReconciliationRequest<'_, '_>) -> FenceReconciliationObservationInput {
    FenceReconciliationObservationInput {
        request: request.digest(),
        mutation_attempt: request.intent().mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        context: request.intent().plan().context().clone(),
        inventory: request.intent().plan().inventory().clone(),
        selected_targets: request.intent().plan().selected_targets().into(),
        settlement: match request.intent().obligation().scope() {
            FenceObligationScopeRecord::Capture { .. } => {
                FenceReconciliationEvidence::AcquiredCapture {
                    fence: Box::new(consistency::fence()),
                    attribution: hash("34"),
                }
            }
            FenceObligationScopeRecord::Restore { .. } => {
                FenceReconciliationEvidence::AcquiredRestore {
                    fence: Box::new(restore_safety::fence()),
                    attribution: hash("34"),
                }
            }
        },
        evidence: hash("ab"),
        remote_observations: 1,
    }
}
