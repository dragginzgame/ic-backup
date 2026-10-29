//! Original stage allocations and exact learned-plan bindings; no second spending ledger.

use crate::model::evidence_digest::EvidenceDigest;
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    bounded_sequence,
    effect_graph::MAX_EFFECT_DEPENDENCIES,
    operation_plan::{OperationPlanError, OperationPlanRecord, PlannedOperationRecord},
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Maximum raw and canonical workflow declaration bytes.
pub const MAX_EXECUTION_WORKFLOW_BYTES: u64 = 1024 * 1024;
/// Maximum raw and canonical learned-stage binding bytes.
pub const MAX_EXECUTION_STAGE_BYTES: u64 = 512 * 1024;

/// Strict v1 original stage catalog, admitted by the existing plan structural owner.
///
/// Each allocation operation denotes one stage: its request hash commits the
/// integration-owned input/purpose contract, its target is the exact stage target,
/// and its budget is the original ceiling for that stage's later exact child plan.
/// The allocation is never an executable operation plan or an attempt-journal
/// authority. Only bound child plans own attempt journals. Unassigned aggregate
/// headroom cannot be transferred to a stage. This distinct record does not change
/// the meaning or schema of ordinary v1 operation plans.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "WorkflowFields")]
pub struct ExecutionWorkflowRecord {
    version: u16,
    allocation: OperationPlanRecord,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowFields {
    version: u16,
    allocation: OperationPlanRecord,
}
impl TryFrom<WorkflowFields> for ExecutionWorkflowRecord {
    type Error = ExecutionWorkflowError;
    fn try_from(fields: WorkflowFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(ExecutionWorkflowError::UnsupportedVersion(fields.version));
        }
        Ok(Self::new(fields.allocation))
    }
}
impl ExecutionWorkflowRecord {
    pub(crate) const fn allocation(&self) -> &OperationPlanRecord {
        &self.allocation
    }
    /// Retain a canonical allocation without creating journals or dispatch authority.
    #[must_use]
    pub const fn new(allocation: OperationPlanRecord) -> Self {
        Self {
            version: 1,
            allocation,
        }
    }
    /// Read one original stage's purpose commitment, target and attempt ceilings.
    /// # Errors
    /// Rejects a stage not present in the original catalog.
    pub fn stage(&self, sequence: u64) -> Result<&PlannedOperationRecord, OperationPlanError> {
        self.allocation.operation(sequence)
    }
    /// Read the canonical original stage catalog; rows are declarations, not journals.
    #[must_use]
    pub fn stages(&self) -> &[PlannedOperationRecord] {
        self.allocation.operations()
    }
    /// Bind the full original allocation under a distinct NUL-terminated v1 domain.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = EvidenceDigest::new(b"ic-backup/execution-workflow/v1\0");
        bytes.checksum(&self.allocation.digest());
        bytes.finish()
    }
}

/// Exact preceding stage binding, chronological settlement and learned-input commitment.
///
/// The evidence digest is integration-owned. It proves neither authenticity nor
/// that learned snapshot IDs/dimensions were derived from those settled attempts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionStagePredecessorRecord {
    stage_sequence: u64,
    binding: ArtifactChecksumRecord,
    settlement: ArtifactChecksumRecord,
    learned_evidence: ArtifactChecksumRecord,
}
impl ExecutionStagePredecessorRecord {
    /// Declare exact predecessor identities without inferring an effect outcome.
    #[must_use]
    pub const fn new(
        stage_sequence: u64,
        binding: ArtifactChecksumRecord,
        settlement: ArtifactChecksumRecord,
        learned_evidence: ArtifactChecksumRecord,
    ) -> Self {
        Self {
            stage_sequence,
            binding,
            settlement,
            learned_evidence,
        }
    }
    /// Read the original predecessor stage identity.
    #[must_use]
    pub const fn stage_sequence(&self) -> u64 {
        self.stage_sequence
    }
    /// Read the exact immutable predecessor binding identity.
    #[must_use]
    pub const fn binding(&self) -> &ArtifactChecksumRecord {
        &self.binding
    }
    /// Read the exact retained chronological settlement identity.
    #[must_use]
    pub const fn settlement(&self) -> &ArtifactChecksumRecord {
        &self.settlement
    }
    /// Read the opaque learned-input commitment, without qualifying its meaning.
    #[must_use]
    pub const fn learned_evidence(&self) -> &ArtifactChecksumRecord {
        &self.learned_evidence
    }
}

/// Strict v1 immutable association of one allocated stage with one exact ordinary plan.
///
/// The canonical child plan remains the sole request/authority owner. This record
/// copies no counters, receipts, completion flags or payloads. Local persistence
/// additionally requires every declared predecessor's complete Applied journal
/// settlement; that still grants no authentic capture, transfer or dispatch permit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "StageFields")]
pub struct ExecutionStageBindingRecord {
    version: u16,
    workflow: ArtifactChecksumRecord,
    stage_sequence: u64,
    plan: ArtifactChecksumRecord,
    predecessors: Vec<ExecutionStagePredecessorRecord>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StageFields {
    version: u16,
    workflow: ArtifactChecksumRecord,
    stage_sequence: u64,
    plan: ArtifactChecksumRecord,
    #[serde(deserialize_with = "bounded_predecessors")]
    predecessors: Vec<ExecutionStagePredecessorRecord>,
}
impl TryFrom<StageFields> for ExecutionStageBindingRecord {
    type Error = ExecutionWorkflowError;
    fn try_from(mut fields: StageFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(ExecutionWorkflowError::UnsupportedVersion(fields.version));
        }
        canonical_predecessors(&mut fields.predecessors)?;
        Ok(Self {
            version: 1,
            workflow: fields.workflow,
            stage_sequence: fields.stage_sequence,
            plan: fields.plan,
            predecessors: fields.predecessors,
        })
    }
}
impl ExecutionStageBindingRecord {
    /// Bind exact learned requests within one original target and allocation.
    ///
    /// Child aggregate ceilings must equal the stage's original ceilings; existing
    /// plan admission bounds assigned operations below those ceilings. Zero-budget
    /// child operations reject, so the original 65,536 workflow attempt ceiling
    /// also bounds total bound child operations. No allowance is borrowed/reset.
    /// # Errors
    /// Rejects changed context/inventory/target/limits, unknown stages and incomplete dependencies.
    pub fn new(
        workflow: &ExecutionWorkflowRecord,
        stage_sequence: u64,
        plan: &OperationPlanRecord,
        mut predecessors: Vec<ExecutionStagePredecessorRecord>,
    ) -> Result<Self, ExecutionWorkflowError> {
        validate_child(workflow, stage_sequence, plan)?;
        canonical_predecessors(&mut predecessors)?;
        validate_predecessor_set(workflow, stage_sequence, &predecessors)?;
        Ok(Self {
            version: 1,
            workflow: workflow.digest(),
            stage_sequence,
            plan: plan.digest(),
            predecessors,
        })
    }
    /// Re-admit decoded bindings against full original workflow and child plan.
    /// # Errors
    /// Rejects original identity, plan or allocation/dependency changes.
    pub fn validate(
        &self,
        workflow: &ExecutionWorkflowRecord,
        plan: &OperationPlanRecord,
    ) -> Result<(), ExecutionWorkflowError> {
        if self.workflow != workflow.digest() || self.plan != plan.digest() {
            return Err(ExecutionWorkflowError::BindingMismatch);
        }
        // Construction and deserialization already admit canonical predecessors.
        // Recheck their relation to the original allocation without rebuilding a record.
        validate_child(workflow, self.stage_sequence, plan)?;
        validate_predecessor_set(workflow, self.stage_sequence, &self.predecessors)?;
        Ok(())
    }

    // Both transfer plans bind one exact learned metadata/allocation predecessor.
    // Callers first validate this binding against its full workflow and child plan.
    pub(crate) fn matches_learned_predecessor(
        &self,
        plan: &OperationPlanRecord,
        request: &str,
        learned: &ArtifactChecksumRecord,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> bool {
        matches!(plan.operations(), [operation] if operation.request() == request)
            && predecessors.iter().any(|row| {
                row.stage_sequence() == self.stage_sequence
                    && row.binding() == &self.digest()
                    && row.learned_evidence() == learned
            })
    }

    pub(crate) fn has_learned_evidence(
        &self,
        sequence: u64,
        learned: &ArtifactChecksumRecord,
    ) -> bool {
        self.stage_sequence == sequence
            && self
                .predecessors
                .iter()
                .any(|row| row.learned_evidence() == learned)
    }
    /// Read full original workflow identity.
    #[must_use]
    pub const fn workflow(&self) -> &ArtifactChecksumRecord {
        &self.workflow
    }
    /// Read the fixed original stage identity.
    #[must_use]
    pub const fn stage_sequence(&self) -> u64 {
        self.stage_sequence
    }
    /// Read the exact canonical child plan identity.
    #[must_use]
    pub const fn plan(&self) -> &ArtifactChecksumRecord {
        &self.plan
    }
    /// Read canonical exact predecessors and learned-input commitments.
    #[must_use]
    pub fn predecessors(&self) -> &[ExecutionStagePredecessorRecord] {
        &self.predecessors
    }
    /// Hash the v1 NUL domain, workflow/plan ASCII hashes, big-endian u64 stage/count,
    /// then canonical predecessor u64 identities and binding/settlement/evidence hashes.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = EvidenceDigest::new(b"ic-backup/execution-stage/v1\0");
        bytes.checksum(&self.workflow);
        bytes.u64_be(self.stage_sequence);
        bytes.checksum(&self.plan);
        bytes.u64_be(self.predecessors.len() as u64);
        for row in &self.predecessors {
            bytes.u64_be(row.stage_sequence);
            for digest in [&row.binding, &row.settlement, &row.learned_evidence] {
                bytes.checksum(digest);
            }
        }
        bytes.finish()
    }
}
fn validate_child(
    workflow: &ExecutionWorkflowRecord,
    stage_sequence: u64,
    plan: &OperationPlanRecord,
) -> Result<(), ExecutionWorkflowError> {
    let stage = workflow
        .stage(stage_sequence)
        .map_err(|_| ExecutionWorkflowError::UnknownStage(stage_sequence))?;
    if plan.context() != workflow.allocation.context()
        || plan.inventory() != workflow.allocation.inventory()
        || plan.selected_targets() != [stage.target()]
    {
        return Err(ExecutionWorkflowError::ChildIdentityMismatch);
    }
    if plan.budget().mutations() != stage.budget().mutations()
        || plan.budget().observations() != stage.budget().observations()
    {
        return Err(ExecutionWorkflowError::ChildBudgetMismatch);
    }
    if plan.operations().iter().any(|operation| {
        operation.budget().mutations() == 0 && operation.budget().observations() == 0
    }) {
        return Err(ExecutionWorkflowError::UnallocatedOperation);
    }
    Ok(())
}

fn validate_predecessor_set(
    workflow: &ExecutionWorkflowRecord,
    stage_sequence: u64,
    predecessors: &[ExecutionStagePredecessorRecord],
) -> Result<(), ExecutionWorkflowError> {
    let dependencies = workflow
        .allocation
        .graph()
        .node(stage_sequence)
        .map_err(|_| ExecutionWorkflowError::PredecessorMismatch)?
        .depends_on();
    if !predecessors
        .iter()
        .map(ExecutionStagePredecessorRecord::stage_sequence)
        .eq(dependencies.iter().copied())
    {
        return Err(ExecutionWorkflowError::PredecessorMismatch);
    }
    Ok(())
}

fn canonical_predecessors(
    rows: &mut [ExecutionStagePredecessorRecord],
) -> Result<(), ExecutionWorkflowError> {
    if rows.len() > MAX_EFFECT_DEPENDENCIES {
        return Err(ExecutionWorkflowError::PredecessorMismatch);
    }
    rows.sort_by_key(ExecutionStagePredecessorRecord::stage_sequence);
    if rows
        .windows(2)
        .any(|pair| pair[0].stage_sequence == pair[1].stage_sequence)
    {
        return Err(ExecutionWorkflowError::PredecessorMismatch);
    }
    Ok(())
}
fn bounded_predecessors<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ExecutionStagePredecessorRecord>, D::Error> {
    bounded_sequence::deserialize::<_, _, MAX_EFFECT_DEPENDENCIES>(
        deserializer,
        "at most 1,024 exact stage predecessors",
        ExecutionWorkflowError::PredecessorMismatch,
    )
}
/// Typed rejection without changing original records or spending.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExecutionWorkflowError {
    /// Only the distinct maintained v1 generation is admitted.
    #[error("unsupported execution workflow/stage version {0}")]
    UnsupportedVersion(u16),
    /// Full context, inventory or exact single allocated target changed.
    #[error("child plan differs from original stage identity")]
    ChildIdentityMismatch,
    /// Child aggregate ceilings differ from original assigned stage limits.
    #[error("child plan differs from original stage budget")]
    ChildBudgetMismatch,
    /// A child operation has no assigned allowance.
    #[error("child plan contains an unallocated operation")]
    UnallocatedOperation,
    /// Predecessors are excessive, duplicated, missing or outside the original graph.
    #[error("stage predecessors differ from original dependencies")]
    PredecessorMismatch,
    /// Retained full workflow or child plan digest differs.
    #[error("execution stage binding mismatch")]
    BindingMismatch,
    /// The original catalog has no stage with this identity.
    #[error("unknown execution stage {0}")]
    UnknownStage(u64),
}

#[cfg(test)]
mod tests;
