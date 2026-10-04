//! Fresh explicit dependency regressions adapted from Canic phase and restore ordering.

use super::*;

fn node(sequence: u64, deps: &[u64]) -> EffectNodeRecord {
    EffectNodeRecord::new(EffectNodeRequest {
        operation_sequence: sequence,
        depends_on: deps.to_vec(),
    })
    .expect("node")
}
fn diamond() -> EffectGraphRecord {
    EffectGraphRecord::new(vec![
        node(0, &[]),
        node(2, &[0]),
        node(9, &[0]),
        node(u64::MAX, &[9, 2]),
    ])
    .expect("diamond")
}

#[test]
fn canonical_binary_hash_and_order_ignore_input_order_but_bind_exact_identities_and_edges() {
    let graph = diamond();
    let mut reverse = graph.nodes().to_vec();
    reverse.reverse();
    assert_eq!(
        EffectGraphRecord::new(reverse).expect("same declarations"),
        graph
    );
    assert_eq!(
        graph.digest().hash(),
        "e4c817eec02d0579d3aae92526818ba26863ae8ee406963659f0a51c4e99f188"
    );
    assert_eq!(
        graph.node(u64::MAX).expect("exact max ID").depends_on(),
        &[2, 9]
    );
    assert_eq!(
        graph
            .ordered_nodes()
            .map(EffectNodeRecord::operation_sequence)
            .collect::<Vec<_>>(),
        vec![0, 2, 9, u64::MAX]
    );
    let changed = EffectGraphRecord::new(vec![
        node(0, &[]),
        node(2, &[0]),
        node(9, &[]),
        node(u64::MAX, &[2, 9]),
    ])
    .expect("changed edge");
    assert_ne!(changed.digest(), graph.digest());
    let changed = EffectGraphRecord::new(vec![
        node(0, &[]),
        node(3, &[0]),
        node(9, &[0]),
        node(u64::MAX, &[3, 9]),
    ])
    .expect("changed identity");
    assert_ne!(changed.digest(), graph.digest());
    let mut value = serde_json::to_value(&graph).expect("json");
    value["nodes"][3]["depends_on"] = serde_json::json!([9, 2]);
    assert_eq!(
        serde_json::from_value::<EffectGraphRecord>(value).expect("normalize edge order"),
        graph
    );
    assert_eq!(
        serde_json::from_slice::<EffectGraphRecord>(&serde_json::to_vec(&graph).expect("encode"))
            .expect("roundtrip"),
        graph
    );
}

#[test]
fn explicit_edges_qualify_parent_first_child_first_and_smallest_ready_ties() {
    let stop = EffectGraphRecord::new(vec![node(5, &[10]), node(10, &[])])
        .expect("explicit parent before child");
    assert_eq!(
        stop.ordered_nodes()
            .map(EffectNodeRecord::operation_sequence)
            .collect::<Vec<_>>(),
        vec![10, 5]
    );
    let start = EffectGraphRecord::new(vec![node(101, &[102]), node(102, &[])])
        .expect("explicit child before parent");
    assert_eq!(
        start
            .ordered_nodes()
            .map(EffectNodeRecord::operation_sequence)
            .collect::<Vec<_>>(),
        vec![102, 101]
    );
    let tie = EffectGraphRecord::new(vec![node(1, &[4]), node(4, &[]), node(8, &[])])
        .expect("ready ties");
    assert_eq!(
        tie.ordered_nodes()
            .map(EffectNodeRecord::operation_sequence)
            .collect::<Vec<_>>(),
        vec![4, 1, 8]
    );
}

#[test]
fn duplicate_missing_and_cyclic_dependencies_reject_without_truncating_walks() {
    assert!(matches!(
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 2,
            depends_on: vec![1, 1]
        }),
        Err(EffectGraphError::DuplicateDependency {
            operation_sequence: 2,
            dependency: 1
        })
    ));
    assert!(matches!(
        EffectGraphRecord::new(vec![node(1, &[]), node(1, &[])]),
        Err(EffectGraphError::DuplicateOperation(1))
    ));
    assert!(matches!(
        EffectGraphRecord::new(vec![node(2, &[1])]),
        Err(EffectGraphError::MissingDependency {
            operation_sequence: 2,
            dependency: 1
        })
    ));
    assert!(matches!(
        EffectGraphRecord::new(vec![node(1, &[1])]),
        Err(EffectGraphError::Cycle)
    ));
    assert!(matches!(
        EffectGraphRecord::new(vec![
            node(1, &[2]),
            node(2, &[3]),
            node(3, &[1]),
            node(4, &[])
        ]),
        Err(EffectGraphError::Cycle)
    ));
    assert!(matches!(
        diamond().node(1),
        Err(EffectGraphError::UnknownOperation(1))
    ));
}

#[test]
fn decoding_requires_closed_fields_valid_dependencies_and_recomputed_order() {
    let value = serde_json::to_value(diamond()).expect("json");
    for (path, fields) in [
        ("", vec!["version", "nodes"]),
        ("/nodes/0", vec!["operation_sequence", "depends_on"]),
    ] {
        for field in fields {
            let mut bad = value.clone();
            bad.pointer_mut(path)
                .expect("object")
                .as_object_mut()
                .expect("fields")
                .remove(field);
            assert!(
                serde_json::from_value::<EffectGraphRecord>(bad).is_err(),
                "missing {field}"
            );
        }
        let mut bad = value.clone();
        bad.pointer_mut(path)
            .expect("object")
            .as_object_mut()
            .expect("fields")
            .insert("extra".into(), serde_json::json!(true));
        assert!(serde_json::from_value::<EffectGraphRecord>(bad).is_err());
    }
    for (path, entry) in [
        ("/version", serde_json::json!(2)),
        ("/nodes/0/depends_on", serde_json::json!([u64::MAX])),
        ("/nodes/0/operation_sequence", serde_json::json!(-1)),
        ("/nodes/1/depends_on", serde_json::json!([0, 0])),
        ("/nodes/1/depends_on", serde_json::json!([7])),
        ("/nodes/0/depends_on", serde_json::Value::Null),
    ] {
        let mut bad = value.clone();
        *bad.pointer_mut(path).expect("field") = entry;
        assert!(serde_json::from_value::<EffectGraphRecord>(bad).is_err());
    }
    let mut bad = value.clone();
    bad["order"] = serde_json::json!([3, 2, 1, 0]);
    assert!(serde_json::from_value::<EffectGraphRecord>(bad).is_err());
    let text = serde_json::to_string(&value).expect("json").replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(serde_json::from_str::<EffectGraphRecord>(&text).is_err());
}

#[test]
fn exact_operation_depth_and_edge_bounds_admit_without_recursive_walks() {
    assert!(matches!(
        EffectGraphRecord::new(vec![]),
        Err(EffectGraphError::EmptyGraph)
    ));
    let nodes = (0..MAX_EFFECT_OPERATIONS)
        .map(|index| {
            let id = u64::try_from(index).expect("index");
            if id == 0 {
                node(id, &[])
            } else {
                node(id, &[id - 1])
            }
        })
        .collect();
    let graph = EffectGraphRecord::new(nodes).expect("maximum chain");
    assert_eq!(graph.ordered_nodes().count(), MAX_EFFECT_OPERATIONS);
    assert_eq!(
        serde_json::from_slice::<EffectGraphRecord>(&serde_json::to_vec(&graph).expect("encode"))
            .expect("maximum decode"),
        graph
    );
    let mut excessive = graph.nodes().to_vec();
    excessive.push(node(u64::MAX, &[]));
    assert!(matches!(
        EffectGraphRecord::new(excessive),
        Err(EffectGraphError::TooManyOperations)
    ));
    let mut bad = serde_json::to_value(&graph).expect("json");
    bad["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(serde_json::json!({}));
    assert!(
        serde_json::from_value::<EffectGraphRecord>(bad)
            .expect_err("count before next node decode")
            .to_string()
            .contains("exceeds")
    );
    let dependencies: Vec<_> =
        (0..u64::try_from(MAX_EFFECT_DEPENDENCIES).expect("bound")).collect();
    let mut dense: Vec<_> = dependencies.iter().map(|id| node(*id, &[])).collect();
    for id in 1024..1088 {
        dense.push(node(id, &dependencies));
    }
    let record = EffectGraphRecord::new(dense.clone()).expect("exact 65536 edges");
    assert_eq!(
        record
            .nodes()
            .iter()
            .map(|node| node.depends_on().len())
            .sum::<usize>(),
        MAX_EFFECT_EDGES
    );
    assert_eq!(
        serde_json::from_slice::<EffectGraphRecord>(&serde_json::to_vec(&record).expect("encode"))
            .expect("edge-bound decode"),
        record
    );
    dense.push(node(1088, &[0]));
    assert!(matches!(
        EffectGraphRecord::new(dense),
        Err(EffectGraphError::TooManyEdges)
    ));
    let mut bad = serde_json::to_value(&record).expect("json");
    bad["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(serde_json::json!({"operation_sequence":1088,"depends_on":[0]}));
    assert!(
        serde_json::from_value::<EffectGraphRecord>(bad)
            .expect_err("total bound during decoding")
            .to_string()
            .contains("edges")
    );
    let mut too_many = dependencies;
    too_many.push(1024);
    assert!(matches!(
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 2000,
            depends_on: too_many
        }),
        Err(EffectGraphError::TooManyDependencies)
    ));
    let mut bad = serde_json::to_value(&record).expect("json");
    bad["nodes"][1024]["depends_on"] = serde_json::json!((0..1025).collect::<Vec<_>>());
    assert!(
        serde_json::from_value::<EffectGraphRecord>(bad)
            .expect_err("direct edge bound")
            .to_string()
            .contains("dependencies")
    );
}
