//! Public local graph/progress journey; no IC effects, receipt qualification or dispatch.

use ic_backup::{
    model::effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    ops::persistence::{BackupLayoutGuard, create_effect_graph, read_effect_graph},
    policy::effect_order::{EffectOrderError, EffectProgressRequest, readiness},
};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn retained_graph_projects_causal_progress_without_rewriting_original_dependencies() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("parent")
        .join(format!(
            "ic-backup-public-effect-{}-{nonce}",
            std::process::id()
        ));
    fs::create_dir(&root).expect("layout");
    let record = EffectGraphRecord::new(
        [(10, vec![]), (20, vec![10]), (30, vec![20])]
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
    let digest = record.digest();
    let layout = BackupLayoutGuard::acquire(&root).expect("exclusion");
    create_effect_graph(&layout, &record).expect("retain original graph");
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).expect("reopen");
    let graph = read_effect_graph(&layout, &digest).expect("original digest");
    let retained = fs::read(root.join("effect-graph.json")).expect("original bytes");
    let view = readiness(
        &graph,
        &EffectProgressRequest {
            completed_operations: vec![10],
        },
    )
    .expect("caller-declared causal progress");
    assert_eq!(view.graph, digest);
    assert_eq!(
        view.ready
            .iter()
            .map(|node| node.operation_sequence())
            .collect::<Vec<_>>(),
        vec![20]
    );
    assert_eq!(
        view.blocked
            .iter()
            .map(|node| node.operation_sequence())
            .collect::<Vec<_>>(),
        vec![30]
    );
    assert!(matches!(
        readiness(
            &graph,
            &EffectProgressRequest {
                completed_operations: vec![30]
            }
        ),
        Err(EffectOrderError::UnmetDependency {
            operation_sequence: 30,
            dependency: 20
        })
    ));
    assert_eq!(
        fs::read(root.join("effect-graph.json")).expect("unchanged graph"),
        retained
    );
    assert_eq!(graph, record);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}
