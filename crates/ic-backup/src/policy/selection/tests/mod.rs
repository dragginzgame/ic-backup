//! Fresh pure selection regressions adapted from Canic direct-child/subtree traversal.

use super::*;
use crate::model::inventory::InventoryTargetRequest;

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";
const WORKER: &str = "rno2w-sqaaa-aaaaa-aaacq-cai";
fn inventory() -> InventoryRecord {
    InventoryRecord::new(
        [
            (ROOT, None),
            (APP, Some(ROOT)),
            (WORKER, Some(APP)),
            ("2vxsx-fae", None),
        ]
        .into_iter()
        .map(|(id, parent)| {
            InventoryTargetRecord::new(&InventoryTargetRequest {
                canister_id: id.into(),
                parent_canister_id: parent.map(str::to_owned),
                role: Some("app".into()),
                module_hash: None,
            })
            .expect("row")
        })
        .collect(),
    )
    .expect("forest")
}
fn request(ids: &[&str], expansion: SelectionExpansion) -> SelectionRequest {
    SelectionRequest {
        canister_ids: ids.iter().map(|id| (*id).into()).collect(),
        expansion,
    }
}
fn ids(view: &SelectionView<'_>) -> BTreeSet<String> {
    view.targets
        .iter()
        .map(|target| target.canister_id().to_owned())
        .collect()
}

#[test]
fn exact_direct_children_and_descendants_expand_only_selected_declared_edges() {
    let inventory = inventory();
    for (expansion, expected) in [
        (SelectionExpansion::Exact, vec![ROOT]),
        (SelectionExpansion::DirectChildren, vec![ROOT, APP]),
        (SelectionExpansion::Descendants, vec![ROOT, APP, WORKER]),
    ] {
        let selected = select(&inventory, &request(&[ROOT], expansion)).expect("selection");
        assert_eq!(
            ids(&selected),
            expected.into_iter().map(str::to_owned).collect()
        );
        assert_eq!(selected.inventory, inventory.digest());
        assert!(
            selected
                .targets
                .windows(2)
                .all(|pair| pair[0].canister_id() < pair[1].canister_id())
        );
    }
    let single = select(&inventory, &request(&[WORKER], SelectionExpansion::Exact)).expect("leaf");
    assert_eq!(single.targets[0].parent_canister_id(), Some(APP));
    assert_eq!(single.targets[0].role(), Some("app"));
}

#[test]
fn overlapping_expansions_union_targets_but_explicit_duplicate_aliases_reject() {
    let inventory = inventory();
    let original = inventory.clone();
    let a = select(
        &inventory,
        &request(&[ROOT, APP], SelectionExpansion::Descendants),
    )
    .expect("overlap");
    let b = select(
        &inventory,
        &request(&[APP, ROOT], SelectionExpansion::Descendants),
    )
    .expect("reverse");
    assert_eq!(ids(&a), ids(&b));
    assert_eq!(a.targets.len(), 3);
    let direct = select(
        &inventory,
        &request(&[ROOT, APP], SelectionExpansion::DirectChildren),
    )
    .expect("explicit parent also expanded");
    assert_eq!(ids(&direct), ids(&a));
    assert!(
        matches!(select(&inventory,&request(&[ROOT,&ROOT.to_uppercase()],SelectionExpansion::Exact)),Err(SelectionError::DuplicateSelector(id)) if id==ROOT)
    );
    assert_eq!(inventory, original);
}

#[test]
fn empty_excessive_role_and_unknown_selectors_fail_with_typed_errors() {
    let inventory = inventory();
    assert!(matches!(
        select(&inventory, &request(&[], SelectionExpansion::Exact)),
        Err(SelectionError::EmptySelection)
    ));
    let excessive = SelectionRequest {
        canister_ids: vec![ROOT.into(); MAX_INVENTORY_TARGETS + 1],
        expansion: SelectionExpansion::Descendants,
    };
    assert!(matches!(
        select(&inventory, &excessive),
        Err(SelectionError::TooManySelectors)
    ));
    assert!(matches!(
        select(&inventory, &request(&["app"], SelectionExpansion::Exact)),
        Err(SelectionError::Inventory(
            InventoryRecordError::InvalidPrincipal("canister_id")
        ))
    ));
    let absent = ic_principal::Principal::from_slice(&[99]).to_text();
    assert!(
        matches!(select(&inventory,&request(&[&absent],SelectionExpansion::Descendants)),Err(SelectionError::Inventory(InventoryRecordError::UnknownTarget(id))) if id==absent)
    );
}
