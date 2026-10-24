//! External application journey over explicit declarations, with no live discovery or IC effects.

mod support;

use ic_backup::{
    model::inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
    model::operation_plan::OperationPlanRecord,
    ops::persistence::{BackupLayoutGuard, create_operation_plan, read_operation_plan},
    policy::selection::{SelectionExpansion, SelectionRequest, select},
};
use std::fs;

#[test]
fn explicit_inventory_reopens_and_selects_exact_physical_targets_without_frameworks() {
    let root = support::temp_root("ic-backup-public-inventory");
    let record = InventoryRecord::new(
        [
            ("aaaaa-aa", None),
            ("renrk-eyaaa-aaaaa-aaada-cai", Some("aaaaa-aa")),
            (
                "rno2w-sqaaa-aaaaa-aaacq-cai",
                Some("renrk-eyaaa-aaaaa-aaada-cai"),
            ),
        ]
        .into_iter()
        .map(|(id, parent)| {
            InventoryTargetRecord::new(&InventoryTargetRequest {
                canister_id: id.into(),
                parent_canister_id: parent.map(str::to_owned),
                role: None,
                module_hash: None,
            })
            .expect("explicit row")
        })
        .collect(),
    )
    .expect("declared forest");
    let original_digest = record.digest();
    let plan: OperationPlanRecord = serde_json::from_value(serde_json::json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": record,
        "selected_targets": ["renrk-eyaaa-aaaaa-aaada-cai"],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": "renrk-eyaaa-aaaaa-aaada-cai", "request": "ef".repeat(32), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).expect("complete original plan");
    let layout = BackupLayoutGuard::acquire(&root).expect("exclusion");
    create_operation_plan(&layout, &plan).expect("persist complete original plan");
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).expect("reopen");
    let retained = read_operation_plan(&layout, &plan.digest()).expect("admit original exact plan");
    let selected = select(
        retained.inventory(),
        &SelectionRequest {
            canister_ids: vec!["RENRK-EYAAA-AAAAA-AAADA-CAI".into()],
            expansion: SelectionExpansion::Descendants,
        },
    )
    .expect("canonical exact selector");
    assert_eq!(selected.inventory, original_digest);
    assert_eq!(
        selected
            .targets
            .iter()
            .map(|target| target.canister_id())
            .collect::<Vec<_>>(),
        vec!["renrk-eyaaa-aaaaa-aaada-cai", "rno2w-sqaaa-aaaaa-aaacq-cai"]
    );
    assert_eq!(selected.targets[0].parent_canister_id(), Some("aaaaa-aa"));
    assert_eq!(retained.inventory(), &record);
    assert_eq!(retained, plan);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}
