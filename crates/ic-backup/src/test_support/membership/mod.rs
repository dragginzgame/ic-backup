//! Original plan fixture shared by current-membership model/policy qualification.

use crate::model::{artifacts::ArtifactChecksumRecord, operation_plan::OperationPlanRecord};
use serde_json::json;

pub const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";

pub fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [
            {"canister_id": "aaaaa-aa", "parent_canister_id": null, "role": null, "module_hash": null},
            {"canister_id": APP, "parent_canister_id": "aaaaa-aa", "role": null, "module_hash": null}
        ]},
        "selected_targets": [APP],
        "graph": {"version": 1, "nodes": [
            {"operation_sequence": 0, "depends_on": []},
            {"operation_sequence": 7, "depends_on": [0]}
        ]},
        "operations": [
            {"operation_sequence": 0, "target": APP, "request": "ef".repeat(32), "budget": {"mutations": 1, "observations": 1}},
            {"operation_sequence": 7, "target": APP, "request": "01".repeat(32), "budget": {"mutations": 2, "observations": 1}}
        ],
        "budget": {"mutations": 3, "observations": 2}
    }))
    .expect("original qualified plan fixture")
}

pub fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).expect("opaque exact SHA-256")
}
