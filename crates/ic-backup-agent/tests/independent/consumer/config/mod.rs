//! Decode the parent-owned journey configuration once before building any plans.
use ic_backup::model::operation_plan::PlanContextRecord;
use ic_backup::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::AttemptBudgetRecord,
    effect_graph::{EffectNodeRecord, EffectNodeRequest},
    operation_plan::PlannedOperationRequest,
};
use ic_backup::model::{
    effect_graph::EffectGraphRecord,
    inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
    operation_plan::{
        OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlannedOperationRecord,
    },
};
use ic_backup_agent::AgentTransport;
use serde::{Deserialize, Serialize};
use std::{io, path::Path, sync::Arc, time::Duration};

#[derive(Deserialize, Serialize)]
pub(in super::super) struct Config {
    pub context: PlanContextRecord,
    pub target: String,
    pub endpoint: String,
    pub key: Vec<u8>,
}

impl Config {
    pub(in super::super) fn read(root: &Path) -> io::Result<Self> {
        Ok(serde_json::from_slice(&super::read(
            &root.join("config.json"),
        )?)?)
    }

    pub(super) fn singleton(
        &self,
        sequence: u64,
        request: &ArtifactChecksumRecord,
        budget: AttemptBudgetRecord,
    ) -> OperationPlanRecord {
        let (graph, operations, budget) = singleton_parts(sequence, &self.target, request, budget);
        self.plan(graph, operations, budget)
    }

    pub(super) fn transport(&self, plan: &OperationPlanRecord) -> AgentTransport {
        AgentTransport::new(
            plan.context().clone(),
            &self.endpoint,
            Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(&[7; 32])),
            self.key.clone(),
            Duration::from_secs(30),
        )
        .unwrap()
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

#[derive(Deserialize, Serialize)]
pub(in super::super) struct ApplicationConfig {
    #[serde(flatten)]
    pub connection: Config,
    pub reply_limit: usize,
}

// Only graph/operation construction is shared; the caller supplies the original base and exact budgets.
pub(super) fn singleton(
    original: &OperationPlanRecord,
    sequence: u64,
    target: &str,
    request: &ArtifactChecksumRecord,
    budget: AttemptBudgetRecord,
) -> OperationPlanRecord {
    let (graph, operations, budget) = singleton_parts(sequence, target, request, budget);
    OperationPlanRecord::new(OperationPlanRequest {
        context: original.context().clone(),
        inventory: original.inventory().clone(),
        selected_targets: original.selected_targets().to_vec(),
        graph,
        operations,
        budget,
    })
    .unwrap()
}

fn singleton_parts(
    sequence: u64,
    target: &str,
    request: &ArtifactChecksumRecord,
    budget: AttemptBudgetRecord,
) -> (
    EffectGraphRecord,
    Vec<PlannedOperationRecord>,
    PlanBudgetRecord,
) {
    let aggregate = PlanBudgetRecord::new(budget.mutations(), budget.observations()).unwrap();
    let graph = EffectGraphRecord::new(vec![
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: sequence,
            depends_on: vec![],
        })
        .unwrap(),
    ])
    .unwrap();
    let operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: sequence,
            target: target.into(),
            request: request.hash().into(),
            budget,
        })
        .unwrap(),
    ];
    (graph, operations, aggregate)
}
