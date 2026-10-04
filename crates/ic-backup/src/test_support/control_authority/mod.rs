//! Typed wire/original-plan and passive provider fixtures; no IC backend.

use crate::model::{
    control_authority::{ControlObservationInput, ControlObservationRequest, ControllerSet},
    ic_request::{
        IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord, IcRequestEffect,
    },
    operation_plan::{OperationPlanRecord, PlanContextRecord, PlanContextRequest},
};
use crate::test_support::membership::{APP, hash};

pub fn wires() -> Vec<IcManagementRequestRecord> {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../model/ic_request/tests/golden.json")).unwrap();
    cases
        .into_iter()
        .filter_map(|case| {
            let method: IcManagementMethodRecord =
                serde_json::from_value(case["method"].clone()).unwrap();
            (method.effect() == IcRequestEffect::Mutation).then(|| {
                IcManagementRequestRecord::new(IcManagementRequest {
                    method,
                    target: APP.into(),
                    snapshot_id: serde_json::from_value(case["snapshot_id"].clone()).unwrap(),
                })
                .unwrap()
            })
        })
        .collect()
}
pub fn stop() -> IcManagementRequestRecord {
    wires()
        .into_iter()
        .find(|wire| wire.method() == IcManagementMethodRecord::StopCanister)
        .unwrap()
}
pub fn plan(wire: &IcManagementRequestRecord) -> OperationPlanRecord {
    let original = crate::test_support::membership::plan();
    let mut value = serde_json::to_value(original).unwrap();
    value["operations"][1]["request"] = serde_json::json!(wire.digest().hash());
    serde_json::from_value(value).unwrap()
}
pub fn input(request: &ControlObservationRequest<'_>) -> ControlObservationInput {
    ControlObservationInput {
        request: request.digest(),
        context: PlanContextRecord::new(&PlanContextRequest {
            network: request.binding().network().into(),
            caller: request.binding().caller().into(),
            release: request.binding().release().into(),
        })
        .unwrap(),
        target: request.binding().target().into(),
        controllers: ControllerSet::new(vec![request.binding().caller().into(), "aaaaa-aa".into()])
            .unwrap(),
        evidence: hash("34"),
        remote_observations: 1,
    }
}
