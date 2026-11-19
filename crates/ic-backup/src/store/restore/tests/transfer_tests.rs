//! Exact restore transfer bounds over authenticated sources; no restore effects.

use crate::store::{
    BackupDigest, BackupSummary, RestoreTransferError, RestoreTransferRequest,
    capture::tests::{
        completion::{complete, manifest, mixed, settled},
        source::{extent, maximum},
    },
};
use ic_platform_protocol::management_snapshots::{Extent, ReadKind, SnapshotChunk};
use std::fs;

fn selection(receipt: &BackupSummary) -> RestoreTransferRequest {
    RestoreTransferRequest {
        target: receipt.target,
        release: receipt.release,
        extent_bytes: 3,
        maximum_payload_bytes: 10,
        maximum_management_calls: 15,
    }
}

#[test]
fn exact_region_schedule_uses_new_extent_and_preserves_original_authority() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let path = f.ledger();
    let before = fs::read(&path).unwrap();
    let mut source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    let schedule = source.plan_restore_transfer(&selection(&receipt)).unwrap();
    assert_eq!(schedule.source(), &receipt);
    assert_eq!(
        (schedule.data_calls(), schedule.minimum_management_calls()),
        (4, 15)
    );
    let requests: Vec<_> = schedule.source_requests().collect();
    for (request, (offset, size)) in requests.iter().zip([(0, 3), (3, 3), (6, 3), (9, 1)]) {
        assert_eq!(request.canister_id, receipt.target.canister);
        assert_eq!(request.snapshot_id, extent(&receipt).snapshot_id);
        assert!(
            matches!(&request.kind, ReadKind::WasmModule(Extent { offset: actual_offset, size: actual_size }) if (*actual_offset, *actual_size) == (offset, size))
        );
    }
    let bytes: Vec<_> = requests
        .iter()
        .flat_map(|r| source.read_data(r).unwrap())
        .collect();
    assert_eq!(bytes, b"helloworld");
    assert_eq!(fs::read(path).unwrap(), before);
    assert_eq!(server.count(), 0);
}

#[test]
fn wrong_selection_and_exact_budget_boundaries_refuse_without_source_reads() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    for field in [
        "target",
        "release",
        "extent-zero",
        "extent-large",
        "calls-zero",
        "calls-large",
        "payload-zero",
        "payload-large",
    ] {
        let mut request = selection(&receipt);
        match field {
            "target" => request.target.network = BackupDigest::compute(b"other"),
            "release" => request.release = BackupDigest::compute(b"other"),
            "extent-zero" => request.extent_bytes = 0,
            "extent-large" => request.extent_bytes = 1_048_577,
            "calls-zero" => request.maximum_management_calls = 0,
            "calls-large" => request.maximum_management_calls = 8193,
            "payload-zero" => request.maximum_payload_bytes = 0,
            "payload-large" => request.maximum_payload_bytes = 1_073_741_825,
            _ => unreachable!(),
        }
        let error = source.plan_restore_transfer(&request).unwrap_err();
        assert_eq!(
            error,
            if matches!(field, "target" | "release") {
                RestoreTransferError::Selection
            } else {
                RestoreTransferError::Limits
            }
        );
    }
    let mut request = selection(&receipt);
    request.maximum_payload_bytes = 9;
    assert_eq!(
        source.plan_restore_transfer(&request).unwrap_err(),
        RestoreTransferError::Payload {
            required: 10,
            limit: 9
        }
    );
    request.maximum_payload_bytes = 10;
    request.maximum_management_calls = 14;
    assert_eq!(
        source.plan_restore_transfer(&request).unwrap_err(),
        RestoreTransferError::Calls {
            required: 15,
            limit: 14
        }
    );
    assert_eq!(server.count(), 0);
}

#[test]
fn mixed_regions_and_empty_whole_chunks_use_actual_lengths_and_sorted_order() {
    let (mut f, server, mut p) = mixed();
    let receipt = complete(&mut f, &mut p).unwrap();
    let mut source = f
        .store
        .open_capture_source(&mut f.op, &receipt, 24.try_into().unwrap())
        .unwrap();
    let mut selected = selection(&receipt);
    selected.extent_bytes = 6;
    selected.maximum_payload_bytes = 24;
    selected.maximum_management_calls = 19;
    let schedule = source.plan_restore_transfer(&selected).unwrap();
    assert_eq!(
        (schedule.data_calls(), schedule.minimum_management_calls()),
        (6, 19)
    );
    let requests: Vec<_> = schedule.source_requests().collect();
    let mut hashes = [BackupDigest::compute(b""), BackupDigest::compute(b"chunk")];
    hashes.sort_by_key(|hash| *hash.as_bytes());
    for (request, hash) in requests[4..].iter().zip(hashes) {
        assert!(
            matches!(&request.kind, ReadKind::WasmChunk(SnapshotChunk { hash: actual }) if actual.as_slice() == hash.as_bytes())
        );
    }
    let parts: Vec<_> = requests
        .iter()
        .map(|r| source.read_data(r).unwrap())
        .collect();
    assert_eq!(
        &parts[..4],
        &[
            b"hellow".to_vec(),
            b"orld".to_vec(),
            b"abcdef".to_vec(),
            b"xyz".to_vec()
        ]
    );
    assert_eq!(parts.iter().map(Vec::len).sum::<usize>(), 24);
    assert!(parts[4..].iter().any(Vec::is_empty));
    selected.extent_bytes = 4;
    assert_eq!(
        source.plan_restore_transfer(&selected).unwrap_err(),
        RestoreTransferError::Chunk {
            bytes: 5,
            extent: 4
        }
    );
    assert_eq!(server.count(), 0);
}

#[test]
fn invalidated_payload_source_cannot_plan_fresh_restore_work() {
    let (mut f, _, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let path = manifest(&f).parent().unwrap().join("payload/module.wasm");
    let mut source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    fs::write(path, b"HelloWorld").unwrap();
    assert!(source.read_data(&extent(&receipt)).is_err());
    assert_eq!(
        source
            .plan_restore_transfer(&selection(&receipt))
            .unwrap_err(),
        RestoreTransferError::Source
    );
}
