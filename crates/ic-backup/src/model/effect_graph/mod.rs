//! Immutable explicit operation dependencies; no universal application ordering or authority.

mod node;
pub use node::{EffectNodeRecord, EffectNodeRequest, MAX_EFFECT_DEPENDENCIES};

use crate::model::artifacts::ArtifactChecksumRecord;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use thiserror::Error;

/// Maximum operation identities in a declared dependency graph.
pub const MAX_EFFECT_OPERATIONS: usize = 8192;
/// Maximum total explicit dependencies across one graph.
pub const MAX_EFFECT_EDGES: usize = 65536;
/// Maximum input/canonical-output bytes admitted by graph persistence.
pub const MAX_EFFECT_GRAPH_BYTES: u64 = 1024 * 1024;

/// Canonical v1 dependency graph over opaque exact operation sequence identities.
///
/// Nodes refer to operations owned by a future complete plan. This record contains
/// no effects, targets, request codecs, current authority or completion receipts.
/// Parent-before-child and lifecycle order must be explicitly qualified by an integration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "GraphFields")]
pub struct EffectGraphRecord {
    version: u16,
    nodes: Vec<EffectNodeRecord>,
    #[serde(skip)]
    order: Vec<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphFields {
    version: u16,
    #[serde(deserialize_with = "bounded_nodes")]
    nodes: Vec<EffectNodeRecord>,
}
impl TryFrom<GraphFields> for EffectGraphRecord {
    type Error = EffectGraphError;
    fn try_from(fields: GraphFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(EffectGraphError::UnsupportedVersion(fields.version));
        }
        Self::new(fields.nodes)
    }
}

impl EffectGraphRecord {
    /// Validate bounded, unique, closed acyclic dependencies without IO.
    ///
    /// # Errors
    /// Rejects empty/excessive graphs, repeated identities, missing edges and cycles.
    pub fn new(mut nodes: Vec<EffectNodeRecord>) -> Result<Self, EffectGraphError> {
        if nodes.is_empty() {
            return Err(EffectGraphError::EmptyGraph);
        }
        if nodes.len() > MAX_EFFECT_OPERATIONS {
            return Err(EffectGraphError::TooManyOperations);
        }
        nodes.sort_by_key(EffectNodeRecord::operation_sequence);
        for pair in nodes.windows(2) {
            if pair[0].operation_sequence() == pair[1].operation_sequence() {
                return Err(EffectGraphError::DuplicateOperation(
                    pair[0].operation_sequence(),
                ));
            }
        }
        let mut edges = 0;
        for node in &nodes {
            edges = add_edges(edges, node.depends_on().len())?;
        }
        let order = topological_order(&nodes)?;
        Ok(Self {
            version: 1,
            nodes,
            order,
        })
    }
    /// Read immutable nodes in ascending operation sequence order.
    #[must_use]
    pub fn nodes(&self) -> &[EffectNodeRecord] {
        &self.nodes
    }
    /// Resolve an exact operation identity without treating its numeric value as an order.
    ///
    /// # Errors
    /// Rejects operation identities absent from the original graph.
    pub fn node(&self, sequence: u64) -> Result<&EffectNodeRecord, EffectGraphError> {
        self.nodes
            .binary_search_by_key(&sequence, EffectNodeRecord::operation_sequence)
            .map(|index| &self.nodes[index])
            .map_err(|_| EffectGraphError::UnknownOperation(sequence))
    }
    /// Project deterministic topological order; smallest currently ready sequence wins ties.
    ///
    /// This order grants no effect authorization, target identity or fresh safety evidence.
    pub fn ordered_nodes(
        &self,
    ) -> impl ExactSizeIterator<Item = &EffectNodeRecord> + DoubleEndedIterator + '_ {
        self.order.iter().map(|index| &self.nodes[*index])
    }
    /// Hash canonical explicit identities/edges using the documented v1 binary encoding.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/effect-graph/v1\0".to_vec();
        append_count(&mut bytes, self.nodes.len());
        for node in &self.nodes {
            bytes.extend_from_slice(&node.operation_sequence().to_be_bytes());
            append_count(&mut bytes, node.depends_on().len());
            for dependency in node.depends_on() {
                bytes.extend_from_slice(&dependency.to_be_bytes());
            }
        }
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "validated operation/dependency counts are at most 8192"
)]
fn append_count(bytes: &mut Vec<u8>, count: usize) {
    bytes.extend_from_slice(&(count as u32).to_be_bytes());
}

fn topological_order(nodes: &[EffectNodeRecord]) -> Result<Vec<usize>, EffectGraphError> {
    let indices: BTreeMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.operation_sequence(), index))
        .collect();
    let mut dependents = vec![Vec::new(); nodes.len()];
    let mut remaining: Vec<_> = nodes.iter().map(|node| node.depends_on().len()).collect();
    for (index, node) in nodes.iter().enumerate() {
        for dependency in node.depends_on() {
            let parent = indices
                .get(dependency)
                .ok_or(EffectGraphError::MissingDependency {
                    operation_sequence: node.operation_sequence(),
                    dependency: *dependency,
                })?;
            dependents[*parent].push(index);
        }
    }
    let mut ready: BTreeSet<_> = remaining
        .iter()
        .enumerate()
        .filter_map(|(index, count)| (*count == 0).then_some(index))
        .collect();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(index) = ready.pop_first() {
        order.push(index);
        for dependent in &dependents[index] {
            remaining[*dependent] -= 1;
            if remaining[*dependent] == 0 {
                ready.insert(*dependent);
            }
        }
    }
    if order.len() != nodes.len() {
        return Err(EffectGraphError::Cycle);
    }
    Ok(order)
}

fn add_edges(current: usize, additional: usize) -> Result<usize, EffectGraphError> {
    current
        .checked_add(additional)
        .filter(|total| *total <= MAX_EFFECT_EDGES)
        .ok_or(EffectGraphError::TooManyEdges)
}

fn bounded_nodes<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<EffectNodeRecord>, D::Error> {
    struct NodesVisitor;
    impl<'de> de::Visitor<'de> for NodesVisitor {
        type Value = Vec<EffectNodeRecord>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("bounded explicit effect nodes and total edges")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut nodes = Vec::new();
            let mut edges = 0;
            while nodes.len() < MAX_EFFECT_OPERATIONS {
                match sequence.next_element::<EffectNodeRecord>()? {
                    Some(node) => {
                        edges =
                            add_edges(edges, node.depends_on().len()).map_err(de::Error::custom)?;
                        nodes.push(node);
                    }
                    None => return Ok(nodes),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(EffectGraphError::TooManyOperations));
            }
            Ok(nodes)
        }
    }
    deserializer.deserialize_seq(NodesVisitor)
}

/// Typed declared graph identity, dependency or resource-bound rejection.
#[derive(Debug, Error)]
pub enum EffectGraphError {
    /// Only protocol generation v1 is maintained.
    #[error("unsupported effect graph version {0}")]
    UnsupportedVersion(u16),
    /// At least one exact operation must be declared.
    #[error("effect graph contains no operations")]
    EmptyGraph,
    /// Node count exceeds its maintained bound.
    #[error("effect graph exceeds {MAX_EFFECT_OPERATIONS} operations")]
    TooManyOperations,
    /// One operation declares too many direct dependencies.
    #[error("effect node exceeds {MAX_EFFECT_DEPENDENCIES} dependencies")]
    TooManyDependencies,
    /// Total graph dependencies exceed the maintained bound.
    #[error("effect graph exceeds {MAX_EFFECT_EDGES} edges")]
    TooManyEdges,
    /// An exact operation was declared more than once.
    #[error("duplicate effect operation {0}")]
    DuplicateOperation(u64),
    /// A requested exact operation is absent.
    #[error("unknown effect operation {0}")]
    UnknownOperation(u64),
    /// A dependency identity appears more than once in one operation.
    #[error("effect operation {operation_sequence} repeats dependency {dependency}")]
    DuplicateDependency {
        /// Exact owning operation sequence.
        operation_sequence: u64,
        /// Repeated exact prerequisite sequence.
        dependency: u64,
    },
    /// An edge references an absent exact operation.
    #[error("effect operation {operation_sequence} has absent dependency {dependency}")]
    MissingDependency {
        /// Exact owning operation sequence.
        operation_sequence: u64,
        /// Absent prerequisite sequence.
        dependency: u64,
    },
    /// Dependencies include a self-loop or longer cycle.
    #[error("effect graph contains a dependency cycle")]
    Cycle,
}

#[cfg(test)]
mod tests;
