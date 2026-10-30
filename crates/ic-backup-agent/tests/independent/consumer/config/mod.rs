//! Decode the parent-owned journey configuration once before building any plans.
use ic_backup::model::operation_plan::PlanContextRecord;
use ic_backup::model::{
    effect_graph::EffectGraphRecord,
    inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
    operation_plan::{
        OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlannedOperationRecord,
    },
};
use serde::Deserialize;
use std::{io, path::Path};

#[derive(Deserialize)]
pub(super) struct Config {
    pub context: PlanContextRecord,
    pub target: String,
    pub endpoint: String,
    pub key: Vec<u8>,
}

impl Config {
    pub(super) fn read(root: &Path) -> io::Result<Self> {
        Ok(serde_json::from_slice(&super::read(
            &root.join("config.json"),
        )?)?)
    }

    pub(super) fn plan(
        &self,
        graph: EffectGraphRecord,
        operations: Vec<PlannedOperationRecord>,
        budget: PlanBudgetRecord,
    ) -> OperationPlanRecord {
        OperationPlanRecord::new(OperationPlanRequest {
            context: self.context.clone(),
            inventory: InventoryRecord::new(vec![
                InventoryTargetRecord::new(&InventoryTargetRequest {
                    canister_id: self.target.clone(),
                    parent_canister_id: None,
                    role: None,
                    module_hash: None,
                })
                .unwrap(),
            ])
            .unwrap(),
            selected_targets: vec![self.target.clone()],
            graph,
            operations,
            budget,
        })
        .unwrap()
    }
}
