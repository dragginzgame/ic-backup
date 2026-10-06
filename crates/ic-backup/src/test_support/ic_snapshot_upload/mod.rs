//! Declared originals and independent wire vectors; no simulated IC effects.

use crate::model::{
    ic_snapshot_upload::IcSnapshotUploadRequest, operation_plan::OperationPlanRecord,
};
use serde_json::json;

pub const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
pub const SOURCE_ID: &[u8] = &[0, 255, 17];
pub const DESTINATION_ID: &[u8] = &[21, 0, 255];

pub fn unhex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

pub fn raw(name: &str) -> Vec<u8> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    unhex(
        cases.iter().find(|case| case["name"] == name).unwrap()["reply_hex"]
            .as_str()
            .unwrap(),
    )
}

pub fn plan(request: &str) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1, "context":{"network":"ab".repeat(32), "caller":"2vxsx-fae", "release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":TARGET,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[TARGET], "graph":{"version":1,"nodes":[{"operation_sequence":7,"depends_on":[]}]},
        "operations":[{"operation_sequence":7,"target":TARGET,"request":request,"budget":{"mutations":1,"observations":1}}],
        "budget":{"mutations":1,"observations":1}
    })).unwrap()
}

pub fn source_plan() -> OperationPlanRecord {
    plan(&"ef".repeat(32))
}
pub fn upload_plan(payload: &IcSnapshotUploadRequest<'_>) -> OperationPlanRecord {
    plan(payload.binding_digest().hash())
}
