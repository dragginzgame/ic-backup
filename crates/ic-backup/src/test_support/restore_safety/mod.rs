//! Passive original-source and application-evidence fixtures; not an IC/fence backend.

use crate::{
    model::{
        attempt_journal::AttemptBudgetRecord,
        consistency::ApplicationFenceState,
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
        operation_plan::{
            OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlanContextRecord,
            PlanContextRequest, PlannedOperationRecord, PlannedOperationRequest,
        },
        restore_safety::*,
    },
    test_support::{
        control_authority::wires,
        membership::{APP, hash},
    },
};
use ic_platform::management_canister_types::CanisterStatusType;

pub fn wire(method: IcManagementMethodRecord) -> IcManagementRequestRecord {
    wires()
        .into_iter()
        .find(|wire| wire.method() == method)
        .unwrap()
}
pub fn plan() -> OperationPlanRecord {
    fixture_plan(
        "2vxsx-fae",
        &[
            (7, IcManagementMethodRecord::LoadCanisterSnapshot),
            (8, IcManagementMethodRecord::StartCanister),
        ],
    )
}
pub fn source() -> OperationPlanRecord {
    fixture_plan(
        "aaaaa-aa",
        &[(7, IcManagementMethodRecord::TakeCanisterSnapshot)],
    )
}
fn fixture_plan(caller: &str, methods: &[(u64, IcManagementMethodRecord)]) -> OperationPlanRecord {
    let original = super::membership::plan();
    let nodes = methods
        .iter()
        .enumerate()
        .map(|(index, (sequence, _))| {
            EffectNodeRecord::new(EffectNodeRequest {
                operation_sequence: *sequence,
                depends_on: index
                    .checked_sub(1)
                    .map(|prior| methods[prior].0)
                    .into_iter()
                    .collect(),
            })
            .unwrap()
        })
        .collect();
    let operations = methods
        .iter()
        .map(|(sequence, method)| {
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: *sequence,
                target: APP.into(),
                request: wire(*method).digest().hash().into(),
                budget: AttemptBudgetRecord::new(1, 1).unwrap(),
            })
            .unwrap()
        })
        .collect();
    let count = u32::try_from(methods.len()).unwrap();
    OperationPlanRecord::new(OperationPlanRequest {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: original.context().network().into(),
            caller: caller.into(),
            release: original.context().release().into(),
        })
        .unwrap(),
        inventory: original.inventory().clone(),
        selected_targets: original.selected_targets().to_vec(),
        graph: EffectGraphRecord::new(nodes).unwrap(),
        operations,
        budget: PlanBudgetRecord::new(count, count).unwrap(),
    })
    .unwrap()
}
pub fn binding() -> RestoreFenceBindingRecord {
    RestoreFenceBindingRecord {
        identity: hash("56"),
        membership_revision: hash("78"),
        external_obligations_revision: hash("90"),
    }
}
pub fn parameters(method: IcManagementMethodRecord) -> RestoreSafetyRequestInput {
    RestoreSafetyRequestInput {
        operation_sequence: if method == IcManagementMethodRecord::StartCanister {
            8
        } else {
            7
        },
        challenge: hash("12"),
        max_remote_observations: 1,
    }
}
pub fn requirement(
    plan: &OperationPlanRecord,
    source: &OperationPlanRecord,
    safety: RestoreSafetyLaneRecord,
) -> RestoreSafetyRequirementRecord {
    RestoreSafetyRequirementRecord::new(
        plan,
        source,
        RestoreSafetyRequirementRequest {
            source_artifacts: hash("ab"),
            safety,
            expected_fence: (safety == RestoreSafetyLaneRecord::ApplicationFenced).then(binding),
        },
    )
    .unwrap()
}
pub fn fence() -> RestoreFenceEvidence {
    RestoreFenceEvidence {
        state: ApplicationFenceState::Active,
        binding: binding(),
        whole_selection: hash("bc"),
        rewind_independent_custody: hash("cd"),
        external_obligations_and_replay: hash("de"),
        controlled_execution: Some(hash("ef")),
    }
}
pub fn input(request: &RestoreSafetyRequest<'_>) -> RestoreSafetyObservationInput {
    RestoreSafetyObservationInput {
        request: request.digest(),
        context: PlanContextRecord::new(&PlanContextRequest {
            network: request.binding().network().into(),
            caller: request.binding().caller().into(),
            release: request.binding().release().into(),
        })
        .unwrap(),
        inventory: request.inventory().clone(),
        source_plan_intent: ArtifactChecksumRecord::from_hash(
            request.requirement().source_plan_intent(),
        )
        .unwrap(),
        source_artifacts: request.requirement().source_artifacts().clone(),
        targets: request
            .selected_targets()
            .iter()
            .map(|target| TargetRestoreEvidence {
                target: target.clone(),
                state: CanisterStatusType::Stopped,
                lifecycle_evidence: hash("23"),
                restored_acceptance: Some(hash("45")),
            })
            .collect(),
        safety: match request.requirement().safety() {
            RestoreSafetyLaneRecord::NoIrreversibleEffects => {
                RestoreSafetyEvidence::NoIrreversibleEffects(hash("67"))
            }
            RestoreSafetyLaneRecord::ApplicationFenced => {
                RestoreSafetyEvidence::ApplicationFenced(Box::new(fence()))
            }
        },
        evidence: hash("89"),
        remote_observations: 1,
    }
}
use crate::model::artifacts::ArtifactChecksumRecord;
