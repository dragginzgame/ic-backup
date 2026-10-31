//! Fixed journey declarations; IDs remain opaque and differ from execution order.

use super::backend::Backend;
use ic_backup::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::AttemptBudgetRecord,
    effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    execution_workflow::ExecutionWorkflowRecord,
    operation_plan::{
        OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlannedOperationRecord,
        PlannedOperationRequest,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Stage {
    Capture = 0,
    Metadata = 7,
    Download = 9,
    Allocation = 11,
    Upload = 13,
    Load = 17,
    Start = 3,
}
impl Stage {
    pub const CAPTURE: [Self; 3] = [Self::Capture, Self::Metadata, Self::Download];
    pub const RESTORE: [Self; 4] = [Self::Allocation, Self::Upload, Self::Load, Self::Start];
    pub const fn sequence(self) -> u64 {
        self as u64
    }
    pub fn from_sequence(sequence: u64) -> Self {
        Self::CAPTURE
            .into_iter()
            .chain(Self::RESTORE)
            .find(|stage| stage.sequence() == sequence)
            .expect("fixed original journey stage")
    }
    pub const fn predecessor(self) -> Option<Self> {
        match self {
            Self::Capture | Self::Allocation => None,
            Self::Metadata => Some(Self::Capture),
            Self::Download => Some(Self::Metadata),
            Self::Upload => Some(Self::Allocation),
            Self::Load => Some(Self::Upload),
            Self::Start => Some(Self::Load),
        }
    }
    fn budget(self) -> AttemptBudgetRecord {
        let mutations = if matches!(self, Self::Download | Self::Upload) {
            64
        } else {
            1
        };
        let observations = u32::from(matches!(self, Self::Capture | Self::Metadata));
        AttemptBudgetRecord::new(mutations, observations).unwrap()
    }
    fn request(self, first: &ArtifactChecksumRecord) -> ArtifactChecksumRecord {
        match self {
            Self::Capture | Self::Allocation => first.clone(),
            Self::Metadata => ArtifactChecksumRecord::from_bytes(b"metadata contract"),
            Self::Download => {
                ArtifactChecksumRecord::from_bytes(b"complete original metadata data reads")
            }
            _ => ArtifactChecksumRecord::from_bytes(
                format!("original round-trip stage {}", self.sequence()).as_bytes(),
            ),
        }
    }
}

pub(super) fn catalog(
    backend: &Backend,
    stages: &[Stage],
    first: &ArtifactChecksumRecord,
) -> ExecutionWorkflowRecord {
    let original = backend.plan(first, stages[0].sequence());
    let nodes = stages
        .iter()
        .map(|stage| {
            EffectNodeRecord::new(EffectNodeRequest {
                operation_sequence: stage.sequence(),
                depends_on: stage
                    .predecessor()
                    .map(Stage::sequence)
                    .into_iter()
                    .collect(),
            })
            .unwrap()
        })
        .collect();
    let operations = stages
        .iter()
        .map(|stage| {
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: stage.sequence(),
                target: backend.target.to_text(),
                request: stage.request(first).hash().into(),
                budget: stage.budget(),
            })
            .unwrap()
        })
        .collect();
    let mutations = stages.iter().map(|stage| stage.budget().mutations()).sum();
    let observations = stages
        .iter()
        .map(|stage| stage.budget().observations())
        .sum();
    ExecutionWorkflowRecord::new(
        OperationPlanRecord::new(OperationPlanRequest {
            context: original.context().clone(),
            inventory: original.inventory().clone(),
            selected_targets: original.selected_targets().to_vec(),
            graph: EffectGraphRecord::new(nodes).unwrap(),
            operations,
            budget: PlanBudgetRecord::new(mutations, observations).unwrap(),
        })
        .unwrap(),
    )
}
