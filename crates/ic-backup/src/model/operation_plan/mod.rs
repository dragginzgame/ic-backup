//! Immutable local binding of declarations to graph operations and original attempt authority.

mod budget;
mod context;
mod operation;
pub use budget::{AllocatedAttemptsView, MAX_PLAN_ATTEMPTS, PlanBudgetRecord};
pub use context::{PlanContextRecord, PlanContextRequest};
pub use operation::{PlannedOperationRecord, PlannedOperationRequest};

use crate::model::{
    artifacts::{ArtifactChecksumRecord, ChecksumError},
    attempt_journal::{
        AttemptAuthorityRecord, AttemptJournalRecordError, OperationBindingRecord,
        OperationBindingRequest,
    },
    effect_graph::{EffectGraphRecord, MAX_EFFECT_OPERATIONS},
    inventory::{InventoryRecord, InventoryRecordError, MAX_INVENTORY_TARGETS},
};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{collections::BTreeSet, fmt};
use thiserror::Error;

/// Maximum encoded input and canonical output bytes admitted by plan persistence.
pub const MAX_OPERATION_PLAN_BYTES: u64 = 1024 * 1024;

/// Passive immutable-plan declaration assembled from validated owner records.
#[derive(Clone, Debug)]
pub struct OperationPlanRequest {
    /// Exact declared network/caller/release shared by operations.
    pub context: PlanContextRecord,
    /// Full original declared physical inventory, including unselected parents.
    pub inventory: InventoryRecord,
    /// Nonempty explicit physical selection, without role or privileged-root semantics.
    pub selected_targets: Vec<String>,
    /// Exact declared dependencies over the operation table.
    pub graph: EffectGraphRecord,
    /// One exact target/request/original-budget binding per graph operation.
    pub operations: Vec<PlannedOperationRecord>,
    /// Original declared aggregate attempt ceilings.
    pub budget: PlanBudgetRecord,
}

/// Canonical v1 operation plan binding declarations to exact finite attempt authority.
///
/// This is not an authenticated backup/restore plan, fresh preflight, qualified
/// request payload, consistency/lifecycle contract or executable dispatch permit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "PlanFields")]
pub struct OperationPlanRecord {
    version: u16,
    context: PlanContextRecord,
    inventory: InventoryRecord,
    selected_targets: Vec<String>,
    graph: EffectGraphRecord,
    operations: Vec<PlannedOperationRecord>,
    budget: PlanBudgetRecord,
    #[serde(skip)]
    allocated: AllocatedAttemptsView,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanFields {
    version: u16,
    context: PlanContextRecord,
    inventory: InventoryRecord,
    #[serde(deserialize_with = "bounded_selected")]
    selected_targets: Vec<String>,
    graph: EffectGraphRecord,
    #[serde(deserialize_with = "bounded_operations")]
    operations: Vec<PlannedOperationRecord>,
    budget: PlanBudgetRecord,
}
impl TryFrom<PlanFields> for OperationPlanRecord {
    type Error = OperationPlanError;
    fn try_from(fields: PlanFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(OperationPlanError::UnsupportedVersion(fields.version));
        }
        Self::new(OperationPlanRequest {
            context: fields.context,
            inventory: fields.inventory,
            selected_targets: fields.selected_targets,
            graph: fields.graph,
            operations: fields.operations,
            budget: fields.budget,
        })
    }
}
impl OperationPlanRecord {
    /// Admit canonical exact graph/table/physical selection and aggregate attempt binding.
    ///
    /// # Errors
    /// Rejects invalid selection, operation identity mismatch, unused targets and excessive allowances.
    pub fn new(mut request: OperationPlanRequest) -> Result<Self, OperationPlanError> {
        if request.selected_targets.is_empty() {
            return Err(OperationPlanError::EmptySelection);
        }
        if request.selected_targets.len() > MAX_INVENTORY_TARGETS {
            return Err(OperationPlanError::TooManySelected);
        }
        for id in &mut request.selected_targets {
            *id = request.inventory.target(id)?.canister_id().into();
        }
        request.selected_targets.sort();
        for pair in request.selected_targets.windows(2) {
            if pair[0] == pair[1] {
                return Err(OperationPlanError::DuplicateSelected(pair[0].clone()));
            }
        }
        if request.operations.len() > MAX_EFFECT_OPERATIONS {
            return Err(OperationPlanError::TooManyOperations);
        }
        request
            .operations
            .sort_by_key(PlannedOperationRecord::operation_sequence);
        for pair in request.operations.windows(2) {
            if pair[0].operation_sequence() == pair[1].operation_sequence() {
                return Err(OperationPlanError::DuplicateOperation(
                    pair[0].operation_sequence(),
                ));
            }
        }
        if request.operations.len() != request.graph.nodes().len() {
            return Err(OperationPlanError::OperationCountMismatch);
        }
        let selected: BTreeSet<_> = request
            .selected_targets
            .iter()
            .map(String::as_str)
            .collect();
        let mut used = BTreeSet::new();
        let mut allocated = AllocatedAttemptsView {
            mutations: 0,
            observations: 0,
        };
        for (node, operation) in request.graph.nodes().iter().zip(&request.operations) {
            if node.operation_sequence() != operation.operation_sequence() {
                return Err(OperationPlanError::OperationGraphMismatch {
                    expected: node.operation_sequence(),
                    actual: operation.operation_sequence(),
                });
            }
            if !selected.contains(operation.target()) {
                return Err(OperationPlanError::UnselectedTarget(
                    operation.target().into(),
                ));
            }
            used.insert(operation.target());
            allocated.mutations = allocated
                .mutations
                .checked_add(operation.budget().mutations())
                .ok_or(OperationPlanError::BudgetTooLarge)?;
            allocated.observations = allocated
                .observations
                .checked_add(operation.budget().observations())
                .ok_or(OperationPlanError::BudgetTooLarge)?;
        }
        if !request.budget.admits(allocated) {
            return Err(OperationPlanError::AssignedBudgetExceeded);
        }
        for id in &request.selected_targets {
            if !used.contains(id.as_str()) {
                return Err(OperationPlanError::UnusedTarget(id.clone()));
            }
        }
        Ok(Self {
            version: 1,
            context: request.context,
            inventory: request.inventory,
            selected_targets: request.selected_targets,
            graph: request.graph,
            operations: request.operations,
            budget: request.budget,
            allocated,
        })
    }
    /// Read immutable declared context.
    #[must_use]
    pub const fn context(&self) -> &PlanContextRecord {
        &self.context
    }
    /// Read the full original declared inventory.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        &self.inventory
    }
    /// Read canonical exact physical selection, without effect-order semantics.
    #[must_use]
    pub fn selected_targets(&self) -> &[String] {
        &self.selected_targets
    }
    /// Read the bound original explicit dependency graph.
    #[must_use]
    pub const fn graph(&self) -> &EffectGraphRecord {
        &self.graph
    }
    /// Read operation bindings in canonical sequence order.
    #[must_use]
    pub fn operations(&self) -> &[PlannedOperationRecord] {
        &self.operations
    }
    /// Read original aggregate attempt ceilings.
    #[must_use]
    pub const fn budget(&self) -> &PlanBudgetRecord {
        &self.budget
    }
    /// Project assigned allowances only; this performs no observations or consumption.
    #[must_use]
    pub const fn allocated_attempts(&self) -> AllocatedAttemptsView {
        self.allocated
    }
    /// Resolve an exact declared operation binding.
    ///
    /// # Errors
    /// Rejects an operation absent from the exact original plan.
    pub fn operation(&self, sequence: u64) -> Result<&PlannedOperationRecord, OperationPlanError> {
        self.operations
            .binary_search_by_key(&sequence, PlannedOperationRecord::operation_sequence)
            .map(|index| &self.operations[index])
            .map_err(|_| OperationPlanError::UnknownOperation(sequence))
    }
    /// Derive exact existing attempt authority under the full original canonical plan digest.
    ///
    /// Repeated derivation returns the same declaration and never creates/resets a journal.
    /// Fresh authority, actual request bytes and dispatch admission stay caller-owned.
    ///
    /// # Errors
    /// Rejects unknown operations or invalid derived identity admission.
    pub fn attempt_authority(
        &self,
        sequence: u64,
    ) -> Result<AttemptAuthorityRecord, OperationPlanError> {
        let operation = self.operation(sequence)?;
        let binding = OperationBindingRecord::new(&OperationBindingRequest {
            intent: self.digest().hash().into(),
            operation_sequence: sequence,
            network: self.context.network().into(),
            caller: self.context.caller().into(),
            target: operation.target().into(),
            release: self.context.release().into(),
            request: operation.request().into(),
        })?;
        Ok(AttemptAuthorityRecord::new(
            binding,
            operation.budget().clone(),
        ))
    }
    /// Hash the entire canonical declaration using the maintained nonrecursive v1 encoding.
    ///
    /// Derived intent/authority, allocated views and local progress are excluded.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/operation-plan/v1\0".to_vec();
        bytes.extend_from_slice(self.context.network().as_bytes());
        append_principal(&mut bytes, self.context.caller());
        bytes.extend_from_slice(self.context.release().as_bytes());
        bytes.extend_from_slice(self.inventory.digest().hash().as_bytes());
        bytes.extend_from_slice(self.graph.digest().hash().as_bytes());
        append_count(&mut bytes, self.selected_targets.len());
        for target in &self.selected_targets {
            append_principal(&mut bytes, target);
        }
        bytes.extend_from_slice(&self.budget.mutations().to_be_bytes());
        bytes.extend_from_slice(&self.budget.observations().to_be_bytes());
        append_count(&mut bytes, self.operations.len());
        for operation in &self.operations {
            bytes.extend_from_slice(&operation.operation_sequence().to_be_bytes());
            append_principal(&mut bytes, operation.target());
            bytes.extend_from_slice(operation.request().as_bytes());
            bytes.extend_from_slice(&operation.budget().mutations().to_be_bytes());
            bytes.extend_from_slice(&operation.budget().observations().to_be_bytes());
        }
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}
fn append_principal(bytes: &mut Vec<u8>, text: &str) {
    // Owning boundaries admit at most 63 canonical ASCII bytes, so the low byte is exact.
    bytes.push(text.len().to_le_bytes()[0]);
    bytes.extend_from_slice(text.as_bytes());
}
#[expect(
    clippy::cast_possible_truncation,
    reason = "validated selection/operation counts are at most 8192"
)]
fn append_count(bytes: &mut Vec<u8>, count: usize) {
    bytes.extend_from_slice(&(count as u32).to_be_bytes());
}
fn canonical_hash(text: &str) -> Result<String, OperationPlanError> {
    Ok(ArtifactChecksumRecord::from_hash(text)?.hash().into())
}

fn bounded_selected<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    bounded::<D, String, MAX_INVENTORY_TARGETS>(deserializer)
}
fn bounded_operations<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<PlannedOperationRecord>, D::Error> {
    bounded::<D, PlannedOperationRecord, MAX_EFFECT_OPERATIONS>(deserializer)
}
fn bounded<'de, D, T, const LIMIT: usize>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Visitor<T, const LIMIT: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const LIMIT: usize> de::Visitor<'de> for Visitor<T, LIMIT> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "at most {LIMIT} plan entries")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = Vec::new();
            while entries.len() < LIMIT {
                match sequence.next_element()? {
                    Some(entry) => entries.push(entry),
                    None => return Ok(entries),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(format!(
                    "plan list exceeds {LIMIT} entries"
                )));
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(Visitor::<T, LIMIT>(std::marker::PhantomData))
}

/// Typed original plan identity, structural binding or finite allowance rejection.
#[derive(Debug, Error)]
pub enum OperationPlanError {
    /// Only protocol generation v1 is maintained.
    #[error("unsupported operation plan version {0}")]
    UnsupportedVersion(u16),
    /// At least one exact physical selection is required.
    #[error("operation plan has no selected targets")]
    EmptySelection,
    /// Exact selection exceeds the maintained inventory bound.
    #[error("operation plan exceeds {MAX_INVENTORY_TARGETS} selected targets")]
    TooManySelected,
    /// Operation bindings exceed the maintained graph count.
    #[error("operation plan exceeds {MAX_EFFECT_OPERATIONS} operations")]
    TooManyOperations,
    /// Principal text fails canonical admission at this named field.
    #[error("invalid operation plan principal in {0}")]
    InvalidPrincipal(&'static str),
    /// Equivalent physical target was explicitly selected twice.
    #[error("duplicate selected plan target {0}")]
    DuplicateSelected(String),
    /// Operation identity was bound more than once.
    #[error("duplicate planned operation {0}")]
    DuplicateOperation(u64),
    /// Operation table does not have one binding per graph node.
    #[error("operation table count differs from dependency graph")]
    OperationCountMismatch,
    /// Exact table identity differs from its corresponding canonical graph node.
    #[error("operation table differs from graph: expected {expected}, actual {actual}")]
    OperationGraphMismatch {
        /// Exact graph operation identity.
        expected: u64,
        /// Rejected table identity.
        actual: u64,
    },
    /// Exact operation target lies outside explicit selection.
    #[error("operation targets unselected physical identity {0}")]
    UnselectedTarget(String),
    /// A selected physical target has no bound operation.
    #[error("selected plan target has no operation {0}")]
    UnusedTarget(String),
    /// Requested exact operation is absent.
    #[error("unknown planned operation {0}")]
    UnknownOperation(u64),
    /// Aggregate ceilings overflow or exceed the maintained maximum.
    #[error("operation plan attempt ceiling exceeds {MAX_PLAN_ATTEMPTS}")]
    BudgetTooLarge,
    /// Assigned original allowances exceed a declared mutation/observation ceiling.
    #[error("assigned operation attempts exceed original plan ceilings")]
    AssignedBudgetExceeded,
    /// Digest text fails canonical admission.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// Owning inventory rejects selected identity.
    #[error(transparent)]
    Inventory(#[from] InventoryRecordError),
    /// Existing attempt authority boundary rejected derivation.
    #[error(transparent)]
    Attempt(#[from] AttemptJournalRecordError),
}

#[cfg(test)]
pub(crate) mod tests;
