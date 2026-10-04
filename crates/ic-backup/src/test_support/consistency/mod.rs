//! Passive whole-selection consistency fixtures; no management or application backend.

use crate::model::consistency::*;
use crate::test_support::membership::hash;

pub fn parameters(guarantee: ConsistencyGuaranteeRecord) -> ConsistencyRequestInput {
    ConsistencyRequestInput {
        operation_sequence: 7,
        challenge: hash("12"),
        boundary: ConsistencyBoundary::BeforeCapture,
        expected_fence: match guarantee {
            ConsistencyGuaranteeRecord::PerCanister => None,
            ConsistencyGuaranteeRecord::ApplicationCoordinated => Some(ApplicationFenceBinding {
                identity: hash("56"),
                membership_revision: hash("78"),
            }),
        },
        max_remote_observations: 1,
    }
}
pub fn fence() -> ApplicationFenceEvidence {
    ApplicationFenceEvidence {
        state: ApplicationFenceState::Active,
        identity: hash("56"),
        membership_revision: hash("78"),
        writes: hash("90"),
        membership: hash("ab"),
        timers: hash("bc"),
        external_work: hash("cd"),
        drained_work: hash("de"),
    }
}
pub fn input(request: &ConsistencyRequest<'_>) -> ConsistencyObservationInput {
    let context = crate::model::operation_plan::PlanContextRecord::new(
        &crate::model::operation_plan::PlanContextRequest {
            network: request.binding().network().into(),
            caller: request.binding().caller().into(),
            release: request.binding().release().into(),
        },
    )
    .unwrap();
    ConsistencyObservationInput {
        request: request.digest(),
        context,
        inventory: request.inventory().clone(),
        targets: request
            .selected_targets()
            .iter()
            .map(|target| TargetCaptureEvidence {
                target: target.clone(),
                state: CaptureState::Stopped,
                stopped_and_drained: hash("ef"),
            })
            .collect(),
        membership_revision: Some(hash("78")),
        consistency: match request.requirement().guarantee() {
            ConsistencyGuaranteeRecord::PerCanister => ConsistencyEvidence::PerCanister,
            ConsistencyGuaranteeRecord::ApplicationCoordinated => {
                ConsistencyEvidence::ApplicationCoordinated(Box::new(fence()))
            }
        },
        evidence: hash("34"),
        remote_observations: 1,
    }
}
