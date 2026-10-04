//! Fresh canonical plan, projection and finite assigned-authority regressions.

use super::*;
use crate::model::{
    attempt_journal::AttemptBudgetRecord,
    effect_graph::{EffectNodeRecord, EffectNodeRequest},
    inventory::{InventoryTargetRecord, InventoryTargetRequest},
};

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";
pub(crate) fn request() -> OperationPlanRequest {
    let inventory = InventoryRecord::new(
        [(ROOT, None), (APP, Some(ROOT))]
            .into_iter()
            .map(|(id, parent)| {
                InventoryTargetRecord::new(&InventoryTargetRequest {
                    canister_id: id.into(),
                    parent_canister_id: parent.map(str::to_owned),
                    role: None,
                    module_hash: None,
                })
                .expect("row")
            })
            .collect(),
    )
    .expect("inventory");
    let graph = EffectGraphRecord::new(
        [(0, vec![]), (7, vec![0])]
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
    OperationPlanRequest {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: "ab".repeat(32),
            caller: "2vxsx-fae".into(),
            release: "cd".repeat(32),
        })
        .expect("context"),
        inventory,
        selected_targets: vec![APP.into()],
        graph,
        operations: vec![operation(0, APP, "ef", 1, 1), operation(7, APP, "01", 2, 1)],
        budget: PlanBudgetRecord::new(3, 2).expect("aggregate ceilings"),
    }
}
fn operation(
    id: u64,
    target: &str,
    hash: &str,
    mutations: u32,
    observations: u32,
) -> PlannedOperationRecord {
    PlannedOperationRecord::new(PlannedOperationRequest {
        operation_sequence: id,
        target: target.into(),
        request: hash.repeat(32),
        budget: AttemptBudgetRecord::new(mutations, observations).expect("finite operation"),
    })
    .expect("operation")
}

#[test]
fn canonical_digest_and_authority_bind_every_owner_without_recursive_intent() {
    let plan = OperationPlanRecord::new(request()).expect("plan");
    assert_eq!(
        plan.digest().hash(),
        "5385eead7d20b830b36459b1e34edfef024f319b76477d0217a370a27481017a"
    );
    let mut reversed = request();
    reversed.operations.reverse();
    assert_eq!(
        OperationPlanRecord::new(reversed).expect("canonical order"),
        plan
    );
    let mut value = serde_json::to_value(&plan).expect("json");
    for pointer in [
        "/context/network",
        "/context/caller",
        "/context/release",
        "/selected_targets/0",
        "/operations/0/target",
        "/operations/0/request",
    ] {
        let text = value
            .pointer(pointer)
            .expect("field")
            .as_str()
            .expect("text")
            .to_uppercase();
        *value.pointer_mut(pointer).expect("field") = serde_json::json!(text);
    }
    assert_eq!(
        serde_json::from_value::<OperationPlanRecord>(value).expect("canonical aliases"),
        plan
    );
    let authority = plan
        .attempt_authority(7)
        .expect("exact derived declaration");
    assert_eq!(authority.binding().intent(), plan.digest().hash());
    assert_eq!(authority.binding().operation_sequence(), 7);
    assert_eq!(authority.binding().network(), plan.context().network());
    assert_eq!(authority.binding().caller(), plan.context().caller());
    assert_eq!(authority.binding().target(), APP);
    assert_eq!(authority.binding().release(), plan.context().release());
    assert_eq!(
        authority.binding().request(),
        plan.operation(7).expect("binding").request()
    );
    assert_eq!(
        authority.budget(),
        plan.operation(7).expect("budget").budget()
    );
    assert_eq!(
        plan.attempt_authority(7).expect("repeat does not consume"),
        authority
    );
    assert_eq!(
        plan.allocated_attempts(),
        AllocatedAttemptsView {
            mutations: 3,
            observations: 2
        }
    );
    assert_eq!(plan.inventory().targets().len(), 2);
    assert_eq!(plan.selected_targets(), &[APP.to_owned()]);
    assert_eq!(plan.graph().nodes().len(), 2);
    assert_eq!(plan.operations().len(), 2);
    assert_eq!(plan.budget().mutations(), 3);
    assert_eq!(plan.budget().observations(), 2);
    assert!(matches!(
        plan.attempt_authority(99),
        Err(OperationPlanError::UnknownOperation(99))
    ));
    assert_eq!(
        serde_json::from_slice::<OperationPlanRecord>(&serde_json::to_vec(&plan).expect("encode"))
            .expect("decode"),
        plan
    );
}

#[test]
fn changed_context_inventory_graph_request_and_limits_change_original_intent_and_authority() {
    let base = OperationPlanRecord::new(request()).expect("plan");
    for field in [
        "network",
        "caller",
        "release",
        "inventory",
        "selection",
        "graph",
        "request",
        "operation_budget",
        "plan_budget",
    ] {
        let mut changed = request();
        match field {
            "network" => {
                changed.context = PlanContextRecord::new(&PlanContextRequest {
                    network: "23".repeat(32),
                    caller: changed.context.caller().into(),
                    release: changed.context.release().into(),
                })
                .expect("network");
            }
            "caller" => {
                changed.context = PlanContextRecord::new(&PlanContextRequest {
                    network: changed.context.network().into(),
                    caller: ROOT.into(),
                    release: changed.context.release().into(),
                })
                .expect("caller");
            }
            "release" => {
                changed.context = PlanContextRecord::new(&PlanContextRequest {
                    network: changed.context.network().into(),
                    caller: changed.context.caller().into(),
                    release: "23".repeat(32),
                })
                .expect("release");
            }
            "inventory" => {
                let mut value = serde_json::to_value(&changed.inventory).expect("json");
                value["targets"][0]["role"] = serde_json::json!("changed unselected parent");
                changed.inventory = serde_json::from_value(value).expect("inventory");
            }
            "selection" => {
                changed.selected_targets = vec![ROOT.into()];
                changed.operations = vec![
                    operation(0, ROOT, "ef", 1, 1),
                    operation(7, ROOT, "01", 2, 1),
                ];
            }
            "graph" => {
                changed.graph = EffectGraphRecord::new(vec![
                    EffectNodeRecord::new(EffectNodeRequest {
                        operation_sequence: 0,
                        depends_on: vec![],
                    })
                    .expect("node"),
                    EffectNodeRecord::new(EffectNodeRequest {
                        operation_sequence: 7,
                        depends_on: vec![],
                    })
                    .expect("node"),
                ])
                .expect("graph");
            }
            "request" => changed.operations[0] = operation(0, APP, "23", 1, 1),
            "operation_budget" => changed.operations[0] = operation(0, APP, "ef", 0, 1),
            "plan_budget" => {
                changed.budget = PlanBudgetRecord::new(4, 2).expect("different original ceiling");
            }
            _ => unreachable!(),
        }
        let changed = OperationPlanRecord::new(changed).expect("different valid plan");
        assert_ne!(changed.digest(), base.digest(), "{field}");
        assert_ne!(
            changed.attempt_authority(7).expect("derived"),
            base.attempt_authority(7).expect("base"),
            "{field}"
        );
    }
}

#[test]
fn graph_table_physical_selection_and_original_budget_mismatches_reject() {
    let mut input = request();
    input.selected_targets.clear();
    assert!(matches!(
        OperationPlanRecord::new(input),
        Err(OperationPlanError::EmptySelection)
    ));
    let mut input = request();
    input.selected_targets.push(APP.to_uppercase());
    assert!(
        matches!(OperationPlanRecord::new(input),Err(OperationPlanError::DuplicateSelected(id)) if id==APP)
    );
    let mut input = request();
    input.selected_targets = vec!["2vxsx-fae".into()];
    assert!(matches!(
        OperationPlanRecord::new(input),
        Err(OperationPlanError::Inventory(
            InventoryRecordError::UnknownTarget(_)
        ))
    ));
    let mut input = request();
    input.operations.pop();
    assert!(matches!(
        OperationPlanRecord::new(input),
        Err(OperationPlanError::OperationCountMismatch)
    ));
    let mut input = request();
    input.operations[1] = input.operations[0].clone();
    assert!(matches!(
        OperationPlanRecord::new(input),
        Err(OperationPlanError::DuplicateOperation(0))
    ));
    let mut input = request();
    input.operations[1] = operation(9, APP, "01", 2, 1);
    assert!(matches!(
        OperationPlanRecord::new(input),
        Err(OperationPlanError::OperationGraphMismatch {
            expected: 7,
            actual: 9
        })
    ));
    let mut input = request();
    input.operations[1] = operation(7, ROOT, "01", 2, 1);
    assert!(
        matches!(OperationPlanRecord::new(input),Err(OperationPlanError::UnselectedTarget(id)) if id==ROOT)
    );
    let mut input = request();
    input.selected_targets.push(ROOT.into());
    assert!(
        matches!(OperationPlanRecord::new(input),Err(OperationPlanError::UnusedTarget(id)) if id==ROOT)
    );
    for budget in [
        PlanBudgetRecord::new(2, 2).expect("mutation limit"),
        PlanBudgetRecord::new(3, 1).expect("observation limit"),
    ] {
        let mut input = request();
        input.budget = budget;
        assert!(matches!(
            OperationPlanRecord::new(input),
            Err(OperationPlanError::AssignedBudgetExceeded)
        ));
    }
}

#[test]
fn closed_schema_revalidates_all_bindings_and_never_accepts_derived_authority_or_views() {
    let value =
        serde_json::to_value(OperationPlanRecord::new(request()).expect("plan")).expect("json");
    for (path, fields) in [
        (
            "",
            vec![
                "version",
                "context",
                "inventory",
                "selected_targets",
                "graph",
                "operations",
                "budget",
            ],
        ),
        ("/context", vec!["network", "caller", "release"]),
        (
            "/operations/0",
            vec!["operation_sequence", "target", "request", "budget"],
        ),
        ("/budget", vec!["mutations", "observations"]),
    ] {
        for field in fields {
            let mut bad = value.clone();
            bad.pointer_mut(path)
                .expect("object")
                .as_object_mut()
                .expect("fields")
                .remove(field);
            assert!(
                serde_json::from_value::<OperationPlanRecord>(bad).is_err(),
                "missing {field}"
            );
        }
        let mut bad = value.clone();
        bad.pointer_mut(path)
            .expect("object")
            .as_object_mut()
            .expect("fields")
            .insert("extra".into(), serde_json::json!(true));
        assert!(serde_json::from_value::<OperationPlanRecord>(bad).is_err());
    }
    for (path, bad_entry) in [
        ("/version", serde_json::json!(2)),
        ("/context/network", serde_json::json!("label")),
        ("/context/caller", serde_json::json!("invalid")),
        ("/operations/0/target", serde_json::json!("invalid")),
        ("/operations/0/request", serde_json::json!("not a hash")),
        ("/budget/mutations", serde_json::json!(65537)),
        ("/operations/0/budget/mutations", serde_json::json!(1025)),
        ("/operations/1/operation_sequence", serde_json::json!(9)),
        ("/operations/0/target", serde_json::json!(ROOT)),
    ] {
        let mut bad = value.clone();
        *bad.pointer_mut(path).expect("field") = bad_entry;
        assert!(serde_json::from_value::<OperationPlanRecord>(bad).is_err());
    }
    for field in ["intent", "authority", "allocated", "preflight_accepted"] {
        let mut bad = value.clone();
        bad[field] = serde_json::json!(true);
        assert!(serde_json::from_value::<OperationPlanRecord>(bad).is_err());
    }
    let duplicate = serde_json::to_string(&value).expect("json").replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(serde_json::from_str::<OperationPlanRecord>(&duplicate).is_err());
}

#[test]
fn selection_operation_and_assigned_attempt_bounds_reject_without_reallocating() {
    assert!(matches!(
        PlanBudgetRecord::new(u32::MAX, 1),
        Err(OperationPlanError::BudgetTooLarge)
    ));
    assert!(matches!(
        PlanBudgetRecord::new(MAX_PLAN_ATTEMPTS, 1),
        Err(OperationPlanError::BudgetTooLarge)
    ));
    let mut input = request();
    input.selected_targets = vec![APP.into(); MAX_INVENTORY_TARGETS + 1];
    assert!(matches!(
        OperationPlanRecord::new(input),
        Err(OperationPlanError::TooManySelected)
    ));
    let mut input = request();
    input.operations = vec![input.operations[0].clone(); MAX_EFFECT_OPERATIONS + 1];
    assert!(matches!(
        OperationPlanRecord::new(input),
        Err(OperationPlanError::TooManyOperations)
    ));
    let value =
        serde_json::to_value(OperationPlanRecord::new(request()).expect("plan")).expect("json");
    for (field, entries) in [
        (
            "selected_targets",
            serde_json::json!(vec![APP; MAX_INVENTORY_TARGETS + 1]),
        ),
        (
            "operations",
            serde_json::json!(vec![
                value["operations"][0].clone();
                MAX_EFFECT_OPERATIONS + 1
            ]),
        ),
    ] {
        let mut bad = value.clone();
        bad[field] = entries;
        assert!(
            serde_json::from_value::<OperationPlanRecord>(bad)
                .expect_err("decode count bound before admission")
                .to_string()
                .contains("exceeds")
        );
    }
    let mut input = request();
    input.graph = EffectGraphRecord::new(
        (0..64)
            .map(|id| {
                EffectNodeRecord::new(EffectNodeRequest {
                    operation_sequence: id,
                    depends_on: vec![],
                })
                .expect("node")
            })
            .collect(),
    )
    .expect("graph");
    input.operations = (0..64)
        .map(|id| operation(id, APP, "ef", 1024, 0))
        .collect();
    input.budget = PlanBudgetRecord::new(MAX_PLAN_ATTEMPTS, 0).expect("maximum");
    let plan = OperationPlanRecord::new(input).expect("exact assigned bound");
    assert_eq!(plan.allocated_attempts().mutations, MAX_PLAN_ATTEMPTS);
    let mut bad = serde_json::to_value(&plan).expect("json");
    bad["graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(serde_json::json!({"operation_sequence":64,"depends_on":[]}));
    bad["operations"].as_array_mut().expect("ops").push(serde_json::json!({"operation_sequence":64,"target":APP,"request":"ef".repeat(32),"budget":{"mutations":1,"observations":0}}));
    assert!(
        serde_json::from_value::<OperationPlanRecord>(bad)
            .expect_err("assigned total bound")
            .to_string()
            .contains("ceilings")
    );
    let mut input = request();
    input.budget = PlanBudgetRecord::new(0, 0).expect("zero");
    input.operations = vec![operation(0, APP, "ef", 0, 0), operation(7, APP, "01", 0, 0)];
    assert_eq!(
        OperationPlanRecord::new(input)
            .expect("zero allowance declaration")
            .allocated_attempts(),
        AllocatedAttemptsView {
            mutations: 0,
            observations: 0
        }
    );
}
