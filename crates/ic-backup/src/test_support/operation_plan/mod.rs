//! Shared original-plan fixture builders; assertions remain beside their owners.

use crate::model::operation_plan::*;
use crate::model::{
    attempt_journal::AttemptBudgetRecord,
    effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
};

pub(crate) const ROOT: &str = "aaaaa-aa";
pub(crate) const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";
pub(crate) fn request() -> OperationPlanRequest {
    let inventory = InventoryRecord::new(
        [(ROOT, None), (APP, Some(ROOT))]
            .into_iter()
            .map(|(id, parent)| {
                InventoryTargetRecord::new(&InventoryTargetRequest {
                    canister_id: id.into(),
                    parent_canister_id: parent.map(str::to_owned),
                    role: None,
                    module_hash: None,
                })
                .expect("row")
            })
            .collect(),
    )
    .expect("inventory");
    let graph = EffectGraphRecord::new(
        [(0, vec![]), (7, vec![0])]
            .into_iter()
            .map(|(id, depends_on)| {
                EffectNodeRecord::new(EffectNodeRequest {
                    operation_sequence: id,
                    depends_on,
                })
                .expect("node")
            })
            .collect(),
    )
    .expect("graph");
    OperationPlanRequest {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: "ab".repeat(32),
            caller: "2vxsx-fae".into(),
            release: "cd".repeat(32),
        })
        .expect("context"),
        inventory,
        selected_targets: vec![APP.into()],
        graph,
        operations: vec![operation(0, APP, "ef", 1, 1), operation(7, APP, "01", 2, 1)],
        budget: PlanBudgetRecord::new(3, 2).expect("aggregate ceilings"),
    }
}
pub(crate) fn operation(
    id: u64,
    target: &str,
    hash: &str,
    mutations: u32,
    observations: u32,
) -> PlannedOperationRecord {
    PlannedOperationRecord::new(PlannedOperationRequest {
        operation_sequence: id,
        target: target.into(),
        request: hash.repeat(32),
        budget: AttemptBudgetRecord::new(mutations, observations).expect("finite operation"),
    })
    .expect("operation")
}
