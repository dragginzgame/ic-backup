use super::*;
use serde_json::json;

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";

fn plan(targets: &[String]) -> OperationPlanRecord {
    use crate::model::inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest};
    use crate::model::operation_plan::PlanBudgetRecord;
    use crate::tests::support::operation_plan::{graph, operation, request};

    let mut declaration = request();
    declaration.inventory = InventoryRecord::new(
        targets
            .iter()
            .map(|id| {
                InventoryTargetRecord::new(&InventoryTargetRequest {
                    canister_id: id.clone(),
                    parent_canister_id: None,
                    role: None,
                    module_hash: None,
                })
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    declaration.selected_targets = targets.to_vec();
    declaration.graph = graph(
        targets
            .iter()
            .enumerate()
            .map(|(index, _)| (u64::try_from(index).unwrap(), vec![])),
    );
    declaration.operations = targets
        .iter()
        .enumerate()
        .map(|(index, id)| operation(u64::try_from(index).unwrap(), id, "ef", 1, 0))
        .collect();
    declaration.budget = PlanBudgetRecord::new(u32::try_from(targets.len()).unwrap(), 0).unwrap();
    OperationPlanRecord::new(declaration).expect("bounded original plan")
}

fn journal(
    plan: &OperationPlanRecord,
    targets: &[&str],
    state: ArtifactStateRecord,
) -> DownloadJournalRecord {
    let checksum = matches!(
        state,
        ArtifactStateRecord::ChecksumVerified | ArtifactStateRecord::Durable
    )
    .then(|| json!({"algorithm":"sha256","hash":"ef".repeat(32)}));
    serde_json::from_value(json!({
        "version":1,"intent":plan.digest().hash(),
        "artifacts":targets.iter().enumerate().map(|(i,id)|json!({
            "canister_id":id,"snapshot_id":format!("snap-{i}"),
            "snapshot_taken_at_timestamp":u64::MAX,"snapshot_total_size_bytes":u64::MAX,
            "staging_path":format!("artifacts/{id}.tmp"),"artifact_path":format!("artifacts/{id}"),
            "state":state,"checksum":checksum
        })).collect::<Vec<_>>()
    }))
    .expect("validated journal declaration")
}

#[test]
fn exact_durable_set_borrows_original_identity_metadata_and_checksums() {
    let plan = plan(&[ROOT.into()]);
    let journal = journal(&plan, &[ROOT], ArtifactStateRecord::Durable);
    let checksum = validate(&plan, &journal).expect("pure structural admission");
    assert!(std::ptr::eq(
        checksum,
        journal.artifacts()[0].checksum().unwrap()
    ));
    assert_eq!(checksum.hash(), "ef".repeat(32));
}

#[test]
fn rejects_changed_release_and_original_allowances_even_with_equal_targets() {
    let original = plan(&[ROOT.into()]);
    let journal = journal(&original, &[ROOT], ArtifactStateRecord::Durable);
    for change_release in [true, false] {
        let mut value = serde_json::to_value(&original).expect("original declaration");
        if change_release {
            value["context"]["release"] = json!("01".repeat(32));
        } else {
            value["budget"]["mutations"] = json!(2);
        }
        let changed: OperationPlanRecord =
            serde_json::from_value(value).expect("different valid intent");
        assert_eq!(
            validate(&changed, &journal).unwrap_err(),
            DownloadIntegrityPolicyError::IntentMismatch
        );
    }
}

#[test]
fn rejects_missing_extra_and_equal_count_wrong_target_sets() {
    let plan = plan(&[ROOT.into(), APP.into()]);
    for targets in [[ROOT], ["2vxsx-fae"]] {
        let journal = journal(&plan, &targets, ArtifactStateRecord::Durable);
        assert_eq!(
            validate(&plan, &journal).unwrap_err(),
            DownloadIntegrityPolicyError::TargetSetMismatch
        );
    }
}

#[test]
fn rejects_each_non_durable_state() {
    let plan = plan(&[ROOT.into()]);
    let original = journal(&plan, &[ROOT], ArtifactStateRecord::Durable);
    for state in [
        ArtifactStateRecord::Created,
        ArtifactStateRecord::ChecksumVerified,
    ] {
        let mut value = serde_json::to_value(&original).expect("declaration");
        value["artifacts"][0]["state"] = json!(state);
        if state != ArtifactStateRecord::ChecksumVerified {
            value["artifacts"][0]["checksum"] = json!(null);
        }
        let journal = serde_json::from_value(value).expect("valid earlier state");
        assert_eq!(
            validate(&plan, &journal).unwrap_err(),
            DownloadIntegrityPolicyError::NonDurableArtifact {
                canister_id: ROOT.into(),
                state,
            }
        );
    }
}
