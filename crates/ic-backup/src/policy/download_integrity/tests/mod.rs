use super::*;
use serde_json::json;

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";

fn plan(targets: &[String]) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1, "context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":targets.iter().map(|id|json!({"canister_id":id,"parent_canister_id":null,"role":null,"module_hash":null})).collect::<Vec<_>>()},
        "selected_targets":targets,
        "graph":{"version":1,"nodes":targets.iter().enumerate().map(|(i,_)|json!({"operation_sequence":i,"depends_on":[]})).collect::<Vec<_>>()},
        "operations":targets.iter().enumerate().map(|(i,id)|json!({"operation_sequence":i,"target":id,"request":"ef".repeat(32),"budget":{"mutations":1,"observations":0}})).collect::<Vec<_>>(),
        "budget":{"mutations":targets.len(),"observations":0}
    })).expect("bounded original plan")
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
    let plan = plan(&[APP.into(), ROOT.into()]);
    let journal = journal(&plan, &[APP, ROOT], ArtifactStateRecord::Durable);
    let view = validate(&plan, &journal).expect("pure structural admission");
    assert_eq!(view.plan(), &plan);
    assert_eq!(view.journal(), &journal);
    assert_eq!(view.artifacts().len(), 2);
    assert_eq!(view.artifacts()[0].artifact().canister_id(), ROOT);
    assert_eq!(view.artifacts()[0].artifact().snapshot_id(), "snap-1");
    assert_eq!(
        view.artifacts()[0].artifact().snapshot_taken_at_timestamp(),
        u64::MAX
    );
    assert_eq!(
        view.artifacts()[0].artifact().snapshot_total_size_bytes(),
        u64::MAX
    );
    assert_eq!(view.artifacts()[0].checksum().hash(), "ef".repeat(32));
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
    for targets in [
        vec![ROOT],
        vec![ROOT, APP, "2vxsx-fae"],
        vec![ROOT, "2vxsx-fae"],
    ] {
        let journal = journal(&plan, &targets, ArtifactStateRecord::Durable);
        assert_eq!(
            validate(&plan, &journal).unwrap_err(),
            DownloadIntegrityPolicyError::TargetSetMismatch
        );
    }
}

#[test]
fn rejects_each_non_durable_state_including_a_later_selected_artifact() {
    let plan = plan(&[ROOT.into(), APP.into()]);
    let original = journal(&plan, &[ROOT, APP], ArtifactStateRecord::Durable);
    for state in [
        ArtifactStateRecord::Created,
        ArtifactStateRecord::Downloaded,
        ArtifactStateRecord::ChecksumVerified,
    ] {
        let mut value = serde_json::to_value(&original).expect("declaration");
        value["artifacts"][1]["state"] = json!(state);
        if state != ArtifactStateRecord::ChecksumVerified {
            value["artifacts"][1]["checksum"] = json!(null);
        }
        let journal = serde_json::from_value(value).expect("valid earlier state");
        assert_eq!(
            validate(&plan, &journal).unwrap_err(),
            DownloadIntegrityPolicyError::NonDurableArtifact {
                canister_id: APP.into(),
                state,
            }
        );
    }
}

#[test]
fn admits_maximum_selected_artifacts_and_snapshot_token_bytes() {
    let targets = (0..crate::model::download_journal::MAX_DOWNLOAD_ARTIFACTS)
        .map(|i| candid::Principal::from_slice(&i.to_be_bytes()).to_text())
        .collect::<Vec<_>>();
    let plan = plan(&targets);
    let references = targets.iter().map(String::as_str).collect::<Vec<_>>();
    let original = journal(&plan, &references, ArtifactStateRecord::Durable);
    let mut value = serde_json::to_value(original).expect("declaration");
    for artifact in value["artifacts"].as_array_mut().expect("entries") {
        artifact["snapshot_id"] =
            json!("s".repeat(crate::model::download_journal::MAX_SNAPSHOT_ID_BYTES));
    }
    let journal = serde_json::from_value(value).expect("bounded identities");
    let view = validate(&plan, &journal).expect("maximum exact selected set");
    assert_eq!(view.artifacts().len(), targets.len());
    assert!(
        view.artifacts()
            .iter()
            .all(|entry| entry.artifact().snapshot_id().len()
                == crate::model::download_journal::MAX_SNAPSHOT_ID_BYTES)
    );
}
