//! Exclusive integration fixture roots; failed journeys retain their evidence.

use std::path::PathBuf;

/// Reserve a fresh root before a public integration journey writes any children.
pub(crate) fn temp_root(prefix: &str) -> PathBuf {
    let selected_temp = std::env::temp_dir()
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve integration fixture parent: {error}"));
    let root = ic_host_fs::testing::create_fixture_directory(&selected_temp, prefix)
        .unwrap_or_else(|error| panic!("reserve integration fixture {prefix}: {error}"));
    eprintln!(
        "Integration fixture retained until success: {}",
        root.display()
    );
    root
}

/// Publish a finite private JSON fixture; this is test setup, not product authority.
#[allow(
    dead_code,
    reason = "this shared module also compiles in integration binaries without JSON fixtures"
)]
pub(crate) fn create_json_durable(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ic_host_artifacts::artifact::BoundedWriter::new(Vec::new(), 16 * 1024 * 1024);
    serde_json::to_writer_pretty(&mut writer, value)?;
    let bytes = writer.into_inner();
    ic_host_fs::durable::write_bytes(
        path,
        ic_host_fs::durable::WriteOptions {
            mode: ic_host_fs::durable::PublicationMode::CreateNew,
            permissions: 0o600,
        },
        &bytes,
    )?;
    Ok(())
}

/// Decode fixture evidence with Host's bounded, regular no-follow reader.
#[allow(
    dead_code,
    reason = "shared support also compiles in binaries without JSON fixtures"
)]
pub(crate) fn read_json<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
    max_bytes: u64,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = ic_host_fs::read::read_file_no_follow(path, usize::try_from(max_bytes)?)?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// Broad finite admission for trusted fixture oracles, independent of product limits.
#[allow(
    dead_code,
    reason = "shared support also compiles without Candid fixture oracles"
)]
pub(crate) const WIRE_ORACLE_LIMITS: ic_platform::candid::DecodeLimits =
    ic_platform::candid::DecodeLimits {
        reply_bytes: 64 * 1024 * 1024,
        decoding_quota: 512 * 1024 * 1024,
        skipping_quota: 64 * 1024 * 1024,
        type_entries: 4096,
        header_bytes: 64 * 1024,
    };

/// One valid single-target declaration; corruption tests retain their explicit JSON inputs.
#[allow(
    dead_code,
    reason = "support also compiles in unrelated integration binaries"
)]
pub(crate) fn single_operation_plan(
    target: &str,
    sequence: u64,
    request: &str,
) -> ic_backup::model::operation_plan::OperationPlanRecord {
    use ic_backup::model::{
        attempt_journal::AttemptBudgetRecord,
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
        operation_plan::{
            OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlanContextRecord,
            PlanContextRequest, PlannedOperationRecord, PlannedOperationRequest,
        },
    };
    OperationPlanRecord::new(OperationPlanRequest {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: "ab".repeat(32),
            caller: "2vxsx-fae".into(),
            release: "cd".repeat(32),
        })
        .unwrap(),
        inventory: InventoryRecord::new(vec![
            InventoryTargetRecord::new(&InventoryTargetRequest {
                canister_id: target.into(),
                parent_canister_id: None,
                role: None,
                module_hash: None,
            })
            .unwrap(),
        ])
        .unwrap(),
        selected_targets: vec![target.into()],
        graph: EffectGraphRecord::new(vec![
            EffectNodeRecord::new(EffectNodeRequest {
                operation_sequence: sequence,
                depends_on: vec![],
            })
            .unwrap(),
        ])
        .unwrap(),
        operations: vec![
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: sequence,
                target: target.into(),
                request: request.into(),
                budget: AttemptBudgetRecord::new(1, 1).unwrap(),
            })
            .unwrap(),
        ],
        budget: PlanBudgetRecord::new(1, 1).unwrap(),
    })
    .unwrap()
}
