//! External application journey over explicit declarations, with no live discovery or IC effects.

mod support;

use ic_backup::{
    model::inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
    ops::persistence::{BackupLayoutGuard, InventoryError, create_inventory, read_inventory},
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
    let layout = BackupLayoutGuard::acquire(&root).expect("exclusion");
    create_inventory(&layout, &record).expect("persist before later planning");
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).expect("reopen");
    let retained =
        read_inventory(&layout, &original_digest).expect("admit original exact declaration");
    let selected = select(
        &retained,
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
    assert!(matches!(
        create_inventory(&layout, &record),
        Err(InventoryError::Persistence(_))
    ));
    assert_eq!(
        read_inventory(&layout, &original_digest).expect("evidence preserved"),
        record
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}
