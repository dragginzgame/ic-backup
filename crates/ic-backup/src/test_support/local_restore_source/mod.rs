//! Passive original local source declarations; no authenticated capture/transfer evidence.

use crate::{
    model::{
        download_journal::{DownloadArtifactRequest, DownloadJournalRecord},
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest,
        },
    },
    test_support::{membership, restore_safety},
};
use serde_json::json;

pub fn source() -> OperationPlanRecord {
    let mut value = serde_json::to_value(membership::plan()).unwrap();
    value["selected_targets"] = json!(["aaaaa-aa", membership::APP]);
    value["operations"][0]["target"] = json!("aaaaa-aa");
    value["context"]["caller"] = json!("aaaaa-aa");
    serde_json::from_value(value).unwrap()
}
pub fn requests(source: &OperationPlanRecord) -> Vec<DownloadArtifactRequest> {
    source
        .selected_targets()
        .iter()
        .map(|target| DownloadArtifactRequest {
            canister_id: target.clone(),
            snapshot_id: format!("Original-{target}"),
            snapshot_taken_at_timestamp: u64::MAX,
            snapshot_total_size_bytes: u64::MAX,
        })
        .collect()
}
pub fn manifest(source: &OperationPlanRecord) -> DownloadJournalRecord {
    let original = DownloadJournalRecord::new(source.digest().hash(), requests(source)).unwrap();
    let mut value = serde_json::to_value(original).unwrap();
    for row in value["artifacts"].as_array_mut().unwrap() {
        row["state"] = json!("Durable");
        row["checksum"] = serde_json::to_value(membership::hash("12")).unwrap();
    }
    serde_json::from_value(value).unwrap()
}
pub fn requirement(
    restore: &OperationPlanRecord,
    source: &OperationPlanRecord,
    manifest: &DownloadJournalRecord,
) -> RestoreSafetyRequirementRecord {
    RestoreSafetyRequirementRecord::new(
        restore,
        source,
        RestoreSafetyRequirementRequest {
            source_artifacts: manifest.digest(),
            safety: RestoreSafetyLaneRecord::ApplicationFenced,
            expected_fence: Some(restore_safety::binding()),
        },
    )
    .unwrap()
}
