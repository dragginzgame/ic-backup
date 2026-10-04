//! Pure readiness over declared completion identities; no completion proof or dispatch.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    effect_graph::{EffectGraphError, EffectGraphRecord, EffectNodeRecord, MAX_EFFECT_OPERATIONS},
};
use std::collections::BTreeSet;
use thiserror::Error;

/// Passive declared progress supplied by the qualified receipt-owning caller.
#[derive(Clone, Debug)]
pub struct EffectProgressRequest {
    /// Exact graph operations whose prerequisites and completion evidence the caller owns.
    pub completed_operations: Vec<u64>,
}

/// Read-only dependency projection; readiness grants no effect authority or paid allowance.
#[derive(Clone, Debug)]
pub struct EffectReadinessView<'a> {
    /// Exact original full dependency graph digest.
    pub graph: ArtifactChecksumRecord,
    /// Number of admitted declared completed operations, not authenticated receipts.
    pub completed_operations: usize,
    /// Incomplete nodes with every declared prerequisite completed, in planning order.
    pub ready: Vec<&'a EffectNodeRecord>,
    /// Incomplete nodes with at least one uncompleted prerequisite, in planning order.
    pub blocked: Vec<&'a EffectNodeRecord>,
}

/// Validate causal declared progress and project ready/blocked nodes without IO.
///
/// The caller owns qualified actual receipts. Declared completion never establishes
/// current authority, command custody, settlement or full terminal completion.
///
/// # Errors
/// Rejects excessive/duplicate/unknown completed identities and unmet prerequisites.
pub fn readiness<'a>(
    graph: &'a EffectGraphRecord,
    request: &EffectProgressRequest,
) -> Result<EffectReadinessView<'a>, EffectOrderError> {
    if request.completed_operations.len() > MAX_EFFECT_OPERATIONS {
        return Err(EffectOrderError::TooManyCompleted);
    }
    let mut completed = BTreeSet::new();
    for sequence in &request.completed_operations {
        graph.node(*sequence)?;
        if !completed.insert(*sequence) {
            return Err(EffectOrderError::DuplicateCompleted(*sequence));
        }
    }
    for sequence in &completed {
        let node = graph.node(*sequence)?;
        if let Some(dependency) = node
            .depends_on()
            .iter()
            .find(|dependency| !completed.contains(dependency))
        {
            return Err(EffectOrderError::UnmetDependency {
                operation_sequence: *sequence,
                dependency: *dependency,
            });
        }
    }
    let mut ready = Vec::new();
    let mut blocked = Vec::new();
    for node in graph
        .ordered_nodes()
        .filter(|node| !completed.contains(&node.operation_sequence()))
    {
        if node
            .depends_on()
            .iter()
            .all(|dependency| completed.contains(dependency))
        {
            ready.push(node);
        } else {
            blocked.push(node);
        }
    }
    Ok(EffectReadinessView {
        graph: graph.digest(),
        completed_operations: completed.len(),
        ready,
        blocked,
    })
}

/// Typed declared progress inconsistency, before scheduling or effects.
#[derive(Debug, Error)]
pub enum EffectOrderError {
    /// Declared completed list exceeds the maintained graph bound.
    #[error("completed effects exceed {MAX_EFFECT_OPERATIONS} operations")]
    TooManyCompleted,
    /// One exact operation is declared completed more than once.
    #[error("duplicate completed effect {0}")]
    DuplicateCompleted(u64),
    /// A declared completed operation lacks a declared completed prerequisite.
    #[error("completed effect {operation_sequence} has unmet dependency {dependency}")]
    UnmetDependency {
        /// Exact declared completed operation.
        operation_sequence: u64,
        /// Missing declared completed prerequisite.
        dependency: u64,
    },
    /// Exact operation identity is absent from its owning graph.
    #[error(transparent)]
    Graph(#[from] EffectGraphError),
}

#[cfg(test)]
mod tests;
