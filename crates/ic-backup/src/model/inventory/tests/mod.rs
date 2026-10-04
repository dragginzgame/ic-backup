//! Fresh declaration regressions adapted from Canic topology and graph inputs.

use super::*;
use ic_principal::Principal;

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";
const WORKER: &str = "rno2w-sqaaa-aaaaa-aaacq-cai";

fn target(
    id: &str,
    parent: Option<&str>,
    role: Option<&str>,
    hash: Option<&str>,
) -> InventoryTargetRecord {
    InventoryTargetRecord::new(&InventoryTargetRequest {
        canister_id: id.into(),
        parent_canister_id: parent.map(str::to_owned),
        role: role.map(str::to_owned),
        module_hash: hash.map(str::to_owned),
    })
    .expect("valid row")
}
fn forest() -> InventoryRecord {
    InventoryRecord::new(vec![
        target(ROOT, None, Some("root"), None),
        target(APP, Some(ROOT), Some("app"), Some(&"ab".repeat(32))),
        target(WORKER, Some(APP), None, None),
    ])
    .expect("forest")
}

#[test]
fn canonical_order_case_and_golden_hash_bind_the_exact_declaration() {
    let inventory = forest();
    assert_eq!(
        inventory.digest().hash(),
        "e8d4ca96f6c9e13b79cafb3e8bc76a985f5d8dc43958aad77cfcbc713fa027cd"
    );
    let mut rows = inventory.targets().to_vec();
    rows.reverse();
    assert_eq!(InventoryRecord::new(rows).expect("reordered"), inventory);
    assert_eq!(
        inventory
            .target(&APP.to_uppercase())
            .expect("normalize")
            .canister_id(),
        APP
    );
    let mut json = serde_json::to_value(&inventory).expect("json");
    for row in json["targets"].as_array_mut().expect("rows") {
        for field in ["canister_id", "parent_canister_id", "module_hash"] {
            if let Some(text) = row[field].as_str() {
                row[field] = serde_json::json!(text.to_uppercase());
            }
        }
    }
    assert_eq!(
        serde_json::from_value::<InventoryRecord>(json).expect("canonical"),
        inventory
    );
    let one =
        InventoryRecord::new(vec![target(ROOT, None, None, None)]).expect("explicit standalone");
    assert_eq!(
        one.digest().hash(),
        "637c4054fa40a834193b0616a3153e4152bfa9b87ffc50406e8f2801b2f3f819"
    );
    assert_eq!(
        serde_json::from_slice::<InventoryRecord>(&serde_json::to_vec(&inventory).expect("encode"))
            .expect("decode"),
        inventory
    );
}

#[test]
fn hashes_bind_each_field_and_keep_null_empty_and_literal_null_distinct() {
    let baseline = forest();
    for replacement in [
        target(APP, Some(ROOT), Some("different"), Some(&"ab".repeat(32))),
        target(APP, None, Some("app"), Some(&"ab".repeat(32))),
        target(APP, Some(ROOT), Some("app"), Some(&"cd".repeat(32))),
    ] {
        let changed = InventoryRecord::new(vec![
            baseline.target(ROOT).expect("root").clone(),
            replacement,
            baseline.target(WORKER).expect("worker").clone(),
        ])
        .expect("different valid forest");
        assert_ne!(changed.digest(), baseline.digest());
    }
    let digest = |role| {
        InventoryRecord::new(vec![target(ROOT, None, role, None)])
            .expect("row")
            .digest()
    };
    assert_ne!(digest(None), digest(Some("")));
    assert_ne!(digest(None), digest(Some("null")));
    assert_ne!(digest(Some("")), digest(Some("null")));
    assert_ne!(digest(Some("a|module_hash=null\nb")), digest(Some("a")));
    let extra = InventoryRecord::new(vec![
        target(ROOT, None, None, None),
        target(APP, None, None, None),
    ])
    .expect("two roots");
    assert_ne!(extra.digest(), digest(None));
    assert_eq!(extra.targets().len(), 2);
}

#[test]
fn duplicate_alias_missing_parent_and_cycles_reject_before_selection() {
    assert!(
        matches!(InventoryRecord::new(vec![target(ROOT,None,None,None),target(&ROOT.to_uppercase(),None,None,None)]),Err(InventoryRecordError::DuplicateTarget(id)) if id==ROOT)
    );
    assert!(
        matches!(InventoryRecord::new(vec![target(APP,Some(ROOT),None,None)]),Err(InventoryRecordError::MissingParent{canister_id,parent}) if canister_id==APP && parent==ROOT)
    );
    assert!(
        matches!(InventoryRecord::new(vec![target(ROOT,Some(ROOT),None,None)]),Err(InventoryRecordError::Cycle(id)) if id==ROOT)
    );
    assert!(matches!(
        InventoryRecord::new(vec![
            target(ROOT, Some(APP), None, None),
            target(APP, Some(WORKER), None, None),
            target(WORKER, Some(ROOT), None, None)
        ]),
        Err(InventoryRecordError::Cycle(_))
    ));
    assert!(matches!(
        forest().target("app"),
        Err(InventoryRecordError::InvalidPrincipal("canister_id"))
    ));
    assert!(
        matches!(forest().target("2vxsx-fae"),Err(InventoryRecordError::UnknownTarget(id)) if id=="2vxsx-fae")
    );
}

#[test]
fn schema_requires_all_fields_and_validates_declared_graph_on_decode() {
    let inventory = forest();
    let value = serde_json::to_value(&inventory).expect("json");
    for (path, fields) in [
        ("", vec!["version", "targets"]),
        (
            "/targets/0",
            vec!["canister_id", "parent_canister_id", "role", "module_hash"],
        ),
    ] {
        for field in fields {
            let mut bad = value.clone();
            bad.pointer_mut(path)
                .expect("object")
                .as_object_mut()
                .expect("fields")
                .remove(field);
            assert!(
                serde_json::from_value::<InventoryRecord>(bad).is_err(),
                "missing {field}"
            );
        }
        let mut bad = value.clone();
        bad.pointer_mut(path)
            .expect("object")
            .as_object_mut()
            .expect("fields")
            .insert("extra".into(), serde_json::json!(true));
        assert!(serde_json::from_value::<InventoryRecord>(bad).is_err());
    }
    for (path, entry) in [
        ("/version", serde_json::json!(2)),
        ("/targets/0/canister_id", serde_json::json!("invalid")),
        ("/targets/0/module_hash", serde_json::json!("short")),
        ("/targets/0/parent_canister_id", serde_json::json!(ROOT)),
    ] {
        let mut bad = value.clone();
        *bad.pointer_mut(path).expect("field") = entry;
        assert!(serde_json::from_value::<InventoryRecord>(bad).is_err());
    }
    let text = serde_json::to_string(&value).expect("json").replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(serde_json::from_str::<InventoryRecord>(&text).is_err());
}

#[test]
fn counts_and_utf8_role_bytes_are_bounded_without_recursive_graph_traversal() {
    assert!(matches!(
        InventoryRecord::new(vec![]),
        Err(InventoryRecordError::EmptyInventory)
    ));
    let mut rows = Vec::new();
    let mut parent = None;
    for index in 0..MAX_INVENTORY_TARGETS {
        let id = Principal::from_slice(&index.to_be_bytes()).to_text();
        rows.push(target(&id, parent.as_deref(), None, None));
        parent = Some(id);
    }
    let record = InventoryRecord::new(rows.clone()).expect("maximum depth/count");
    assert_eq!(record.targets().len(), MAX_INVENTORY_TARGETS);
    let bytes = serde_json::to_vec(&record).expect("maximum");
    assert_eq!(
        serde_json::from_slice::<InventoryRecord>(&bytes).expect("bounded decode"),
        record
    );
    rows.push(target("2vxsx-fae", None, None, None));
    assert!(matches!(
        InventoryRecord::new(rows),
        Err(InventoryRecordError::TooManyTargets)
    ));
    let mut bad = serde_json::to_value(&record).expect("json");
    bad["targets"]
        .as_array_mut()
        .expect("rows")
        .push(serde_json::json!({}));
    assert!(
        serde_json::from_value::<InventoryRecord>(bad)
            .expect_err("bounded before next row decode")
            .to_string()
            .contains("exceeds")
    );
    let mut request = InventoryTargetRequest {
        canister_id: ROOT.into(),
        parent_canister_id: None,
        role: Some("é".repeat(128)),
        module_hash: None,
    };
    assert!(InventoryTargetRecord::new(&request).is_ok());
    request.role = Some("é".repeat(129));
    assert!(matches!(
        InventoryTargetRecord::new(&request),
        Err(InventoryRecordError::RoleTooLarge)
    ));
    request.role = None;
    request.parent_canister_id = Some("x".repeat(64));
    assert!(matches!(
        InventoryTargetRecord::new(&request),
        Err(InventoryRecordError::InvalidPrincipal("parent_canister_id"))
    ));
}
