//! Passive original-source and application-evidence fixtures; not an IC/fence backend.

use crate::{
    model::{
        consistency::ApplicationFenceState,
        ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
        operation_plan::{OperationPlanRecord, PlanContextRecord, PlanContextRequest},
        restore_safety::*,
    },
    test_support::{
        control_authority::wires,
        membership::{APP, hash},
    },
};
use ic_management_canister_types::CanisterStatusType;
use serde_json::json;

pub fn wire(method: IcManagementMethodRecord) -> IcManagementRequestRecord {
    wires()
        .into_iter()
        .find(|wire| wire.method() == method)
        .unwrap()
}
pub fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[
            {"canister_id":"aaaaa-aa","parent_canister_id":null,"role":null,"module_hash":null},
            {"canister_id":APP,"parent_canister_id":"aaaaa-aa","role":null,"module_hash":null}
        ]},
        "selected_targets":[APP],"graph":{"version":1,"nodes":[{"operation_sequence":7,"depends_on":[]},{"operation_sequence":8,"depends_on":[7]}]},
        "operations":[
            {"operation_sequence":7,"target":APP,"request":wire(IcManagementMethodRecord::LoadCanisterSnapshot).digest().hash(),"budget":{"mutations":1,"observations":1}},
            {"operation_sequence":8,"target":APP,"request":wire(IcManagementMethodRecord::StartCanister).digest().hash(),"budget":{"mutations":1,"observations":1}}
        ],"budget":{"mutations":2,"observations":2}
    })).unwrap()
}
pub fn source() -> OperationPlanRecord {
    let mut value = serde_json::to_value(plan()).unwrap();
    value["context"]["caller"] = json!("aaaaa-aa");
    value["graph"]["nodes"].as_array_mut().unwrap().truncate(1);
    value["operations"].as_array_mut().unwrap().truncate(1);
    value["operations"][0]["request"] = json!(
        wire(IcManagementMethodRecord::TakeCanisterSnapshot)
            .digest()
            .hash()
    );
    value["budget"] = json!({"mutations":1,"observations":1});
    serde_json::from_value(value).unwrap()
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
