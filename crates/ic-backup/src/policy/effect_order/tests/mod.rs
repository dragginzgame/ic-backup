//! Fresh pure causal progress and readiness qualification; declared progress is not a receipt.

use super::*;
use crate::model::effect_graph::{EffectNodeRecord, EffectNodeRequest};
fn graph() -> EffectGraphRecord {
    EffectGraphRecord::new(
        [
            (10, vec![]),
            (20, vec![10]),
            (30, vec![10]),
            (40, vec![20, 30]),
        ]
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
    .expect("graph")
}
fn progress(completed: &[u64]) -> EffectProgressRequest {
    EffectProgressRequest {
        completed_operations: completed.to_vec(),
    }
}
fn ids(nodes: &[&EffectNodeRecord]) -> Vec<u64> {
    nodes.iter().map(|node| node.operation_sequence()).collect()
}

#[test]
fn causal_progress_projects_exact_ready_blocked_and_complete_declared_sets() {
    let graph = graph();
    let original = graph.clone();
    for (completed, ready, blocked) in [
        (vec![], vec![10], vec![20, 30, 40]),
        (vec![10], vec![20, 30], vec![40]),
        (vec![20, 10], vec![30], vec![40]),
        (vec![30, 10, 20], vec![40], vec![]),
        (vec![40, 20, 30, 10], vec![], vec![]),
    ] {
        let view = readiness(&graph, &progress(&completed)).expect("causal declared progress");
        assert_eq!(view.graph, graph.digest());
        assert_eq!(view.completed_operations, completed.len());
        assert_eq!(ids(&view.ready), ready);
        assert_eq!(ids(&view.blocked), blocked);
    }
    assert_eq!(graph, original);
}

#[test]
fn duplicate_unknown_excessive_and_inconsistent_completion_reject() {
    let graph = graph();
    assert!(matches!(
        readiness(&graph, &progress(&[10, 10])),
        Err(EffectOrderError::DuplicateCompleted(10))
    ));
    assert!(matches!(
        readiness(&graph, &progress(&[99])),
        Err(EffectOrderError::Graph(EffectGraphError::UnknownOperation(
            99
        )))
    ));
    assert!(matches!(
        readiness(&graph, &progress(&[20])),
        Err(EffectOrderError::UnmetDependency {
            operation_sequence: 20,
            dependency: 10
        })
    ));
    assert!(matches!(
        readiness(&graph, &progress(&[10, 20, 40])),
        Err(EffectOrderError::UnmetDependency {
            operation_sequence: 40,
            dependency: 30
        })
    ));
    assert!(matches!(
        readiness(&graph, &progress(&vec![10; MAX_EFFECT_OPERATIONS + 1])),
        Err(EffectOrderError::TooManyCompleted)
    ));
}
