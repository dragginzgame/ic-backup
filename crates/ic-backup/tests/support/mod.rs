//! Exclusive integration fixture roots; failed journeys retain their evidence.

use std::path::PathBuf;

pub(crate) mod observation_replay;

/// Decode golden bytes strictly; this support module is compiled outside the library.
pub(crate) fn hex_bytes(hex: &str) -> Vec<u8> {
    let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
    assert!(
        remainder.is_empty(),
        "fixture hex must contain complete bytes"
    );
    pairs
        .iter()
        .map(|pair| {
            u8::from_str_radix(
                std::str::from_utf8(pair).expect("fixture hex must be ASCII"),
                16,
            )
            .expect("invalid fixture hex byte")
        })
        .collect()
}

/// Reserve a fresh root before a public integration journey writes any children.
pub(crate) fn temp_root(prefix: &str) -> PathBuf {
    let parent = std::env::var_os("TMPDIR").map_or_else(
        || {
            let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .unwrap()
                .join("target/tmp");
            std::fs::create_dir_all(&parent).expect("create workspace fixture parent");
            parent
        },
        PathBuf::from,
    );
    let selected_temp = parent
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
pub(crate) fn read_json<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
    max_bytes: u64,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = ic_host_fs::read::read_file_no_follow(path, usize::try_from(max_bytes)?)?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// Broad finite admission for trusted fixture oracles, independent of product limits.
pub(crate) const WIRE_ORACLE_LIMITS: ic_platform::candid::DecodeLimits =
    ic_platform::candid::DecodeLimits {
        reply_bytes: 64 * 1024 * 1024,
        decoding_quota: 512 * 1024 * 1024,
        skipping_quota: 64 * 1024 * 1024,
        type_entries: 4096,
        header_bytes: 64 * 1024,
    };

/// One valid single-target declaration; corruption tests retain their explicit JSON inputs.
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

/// Complete a native byte fixture through the actual IC artifact writer (no IC calls).
#[cfg(unix)]
pub(crate) fn finish_ic_artifact(
    journal: &mut ic_backup::ops::persistence::DownloadJournalGuard<'_>,
    target: &str,
    token: &str,
    bytes: &[u8],
) {
    use ic_backup::model::{
        ic_snapshot_data::{IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
    };
    use ic_platform::{
        candid::contract,
        management_canister_types::{
            ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult, SnapshotDataKind,
        },
    };
    let sources: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../src/model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let source = sources
        .iter()
        .find(|source| source["name"] == "data-source")
        .unwrap();
    let mut values: ReadCanisterSnapshotMetadataResult = ic_platform::candid::decode_one(
        &hex_bytes(source["reply_hex"].as_str().unwrap()),
        WIRE_ORACLE_LIMITS,
    )
    .unwrap();
    values.taken_at_timestamp = journal
        .record()
        .unwrap()
        .artifacts()
        .iter()
        .find(|entry| entry.canister_id() == target && entry.snapshot_id() == token)
        .unwrap()
        .snapshot_taken_at_timestamp();
    values.wasm_module_size = u64::try_from(bytes.len()).unwrap();
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let raw = contract::encode_one(values).unwrap();
    let request = IcSnapshotMetadataRequest::new(target, &[0, 255, 17]).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let data = IcSnapshotDataRequest::new(
        &metadata,
        SnapshotDataKind::WasmModule {
            offset: 0,
            size: u64::try_from(bytes.len()).unwrap(),
        },
    )
    .unwrap();
    let raw_reply = contract::encode_one(ReadCanisterSnapshotDataResult {
        chunk: bytes.to_vec(),
    })
    .unwrap();
    let reply = IcSnapshotDataReply::decode(&data, &raw_reply).unwrap();
    journal
        .stage_ic_snapshot_artifact(token, &metadata, &raw)
        .unwrap()
        .append(&reply)
        .unwrap()
        .finish()
        .unwrap();
}
