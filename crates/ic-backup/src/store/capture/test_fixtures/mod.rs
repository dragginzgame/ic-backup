//! Shared owned cryptographic/native file fixtures. No IC-effect qualification.
use super::advance::*;
use super::{
    BackupApplicationGuard, BackupConnection, BackupSigner, CaptureOperation, CapturePlan,
    CapturePolicy, CaptureRequest, CaptureTiming, OperationAction, OperationOptions,
    OperationSigningRequest, PreparedStatus, StatusAdmissionRecord,
};
use crate::store::BackupDigest;
use ic_platform::{candid_codec, ic_protocol::EnvelopeContent};
use std::borrow::Cow;
pub(in crate::store) mod certificates;
pub(in crate::store) mod completion;
pub(in crate::store) mod data;
pub(in crate::store) mod http;
pub(in crate::store) mod lookup;
pub(in crate::store) mod signatures;
pub(in crate::store) mod source;

fn plan() -> CapturePolicy {
    super::status::tests::plan()
}
fn call_content() -> EnvelopeContent {
    let p = plan();
    EnvelopeContent::Call {
        nonce: Some(vec![0, 255, 7]),
        ingress_expiry: 21,
        sender: p.request().sender.as_slice().try_into().unwrap(),
        canister_id: [].as_slice().try_into().unwrap(),
        method_name: "canister_status".into(),
        arg: super::advance::arguments(&p).unwrap(),
        sender_info: None,
    }
}

pub(in crate::store) fn reply(
    plan: &CapturePlan,
    step: super::OperationStep,
    mixed: bool,
) -> Vec<u8> {
    use ic_platform::{candid_codec::Reserved, management_canister_types::*};
    let snapshot = || Snapshot {
        id: vec![8],
        taken_at_timestamp: 100,
        total_size: u64::MAX,
    };
    let metadata = ReadCanisterSnapshotMetadataResult {
        source: Some(SnapshotSource::TakenFromCanister(Reserved)),
        taken_at_timestamp: 100,
        canister_version: 41,
        wasm_module_size: 10,
        wasm_memory_size: if mixed { 6 } else { 0 },
        stable_memory_size: if mixed { 3 } else { 0 },
        globals: vec![],
        certified_data: vec![],
        wasm_chunk_store: if mixed {
            [b"".as_slice(), b"chunk"]
                .into_iter()
                .map(|bytes| ChunkHash {
                    hash: BackupDigest::compute(bytes).as_bytes().to_vec(),
                })
                .collect()
        } else {
            vec![]
        },
        global_timer: Some(CanisterTimer::Inactive),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::ConditionNotSatisfied),
    };
    match step {
        super::OperationStep::InitialStatus | super::OperationStep::FinalStatus => {
            candid_codec::encode_one(&super::status::tests::status(plan.plan()), 4096).unwrap()
        }
        super::OperationStep::ListSnapshots => {
            let mut old = snapshot();
            old.id = vec![9];
            candid_codec::encode_one(&vec![old], 4096).unwrap()
        }
        super::OperationStep::TakeSnapshot => candid_codec::encode_one(&snapshot(), 4096).unwrap(),
        super::OperationStep::Metadata => candid_codec::encode_one(&metadata, 4096).unwrap(),
        super::OperationStep::Data(n) => {
            let transfer = plan.plan().plan_transfer(&[8], &metadata).unwrap();
            let bytes = match transfer.requests().nth(n as usize).unwrap().kind {
                SnapshotDataKind::WasmModule { offset: 0, .. } => b"hello".as_slice(),
                SnapshotDataKind::WasmModule { .. } => b"world",
                SnapshotDataKind::WasmMemory { offset: 0, .. } => b"abcde",
                SnapshotDataKind::WasmMemory { .. } => b"f",
                SnapshotDataKind::StableMemory { .. } => b"xyz",
                SnapshotDataKind::WasmChunk { hash } => {
                    if hash == BackupDigest::compute(b"").as_bytes() {
                        b"".as_slice()
                    } else {
                        b"chunk".as_slice()
                    }
                }
            };
            candid_codec::encode_one(
                &ReadCanisterSnapshotDataResult {
                    chunk: bytes.to_vec(),
                },
                4096,
            )
            .unwrap()
        }
        _ => panic!("capture fixture received restore step"),
    }
}
