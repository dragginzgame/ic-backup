//! Explicit operator identity, trust, target and original declarations.
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
use std::{io, sync::Arc, time::Duration};

/// Explicit transport inputs; no identity or network trust is discovered implicitly.
pub struct Connection {
    pub(super) context: PlanContextRecord,
    pub(super) target: String,
    pub(super) transport: AgentTransport,
}
impl Connection {
    /// Configure a real signer, out-of-band trust root, fixed endpoint and finite deadline.
    /// # Errors
    /// Rejects invalid target, signer/context, endpoint, trust and timeout inputs.
    pub fn new(
        context: PlanContextRecord,
        target: &str,
        endpoint: &str,
        identity: Arc<dyn ic_platform::agent::Identity>,
        trusted_root: Vec<u8>,
        timeout: Duration,
    ) -> io::Result<Self> {
        let target = ic_platform::Principal::from_text(target)
            .map_err(super::error)?
            .to_text();
        let transport =
            AgentTransport::new(context.clone(), endpoint, identity, trusted_root, timeout)
                .map_err(super::error)?;
        Ok(Self {
            context,
            target,
            transport,
        })
    }
    /// Build one bounded original operation for an adjacent application adapter.
    #[must_use]
    pub fn singleton(
        &self,
        sequence: u64,
        request: &ArtifactChecksumRecord,
        budget: AttemptBudgetRecord,
    ) -> OperationPlanRecord {
        let (graph, operations, budget) = singleton_parts(sequence, &self.target, request, budget);
        self.plan(graph, operations, budget)
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
