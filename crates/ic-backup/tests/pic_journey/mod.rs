//! Focused real recovery cases and shared fixture oracles; product release stays separate.

use ic_platform::candid::contract;
mod backend;
pub(super) mod planned_download;
mod planned_restore;
pub(super) mod planned_runner;
pub(super) mod planned_upload;
mod recovery;

use backend::Backend;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_lifecycle_reply::{IcLifecycleReply, IcLifecycleReplyKind},
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        ic_snapshot_data::{IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
        ic_snapshot_transfer_read::IcSnapshotTransferReadPayload,
        ic_snapshot_upload::{IcSnapshotUploadReply, IcSnapshotUploadReplyKind},
    },
    ops::persistence::{BackupLayoutGuard, DownloadJournalGuard},
};
use ic_platform::management_canister_types::{CanisterStatusType, SnapshotDataKind};
use std::fs;

fn management(backend: &Backend, method: Method, id: Option<Vec<u8>>) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: backend.target.to_text(),
        snapshot_id: id,
    })
    .unwrap()
}

fn lifecycle(
    backend: &mut Backend,
    method: Method,
    id: Option<Vec<u8>>,
    fault: Fault,
    verify: impl FnOnce(&mut Backend) -> Option<ArtifactChecksumRecord>,
) -> Option<u64> {
    let expected = match method {
        Method::StopCanister => CanisterStatusType::Running,
        Method::LoadCanisterSnapshot | Method::StartCanister => CanisterStatusType::Stopped,
        _ => panic!("fixture mutation lane"),
    };
    status(backend, expected);
    let request = management(backend, method, id);
    let discard = matches!(
        (method, fault),
        (Method::StopCanister, Fault::Stop)
            | (Method::StartCanister, Fault::Start)
            | (
                Method::LoadCanisterSnapshot,
                Fault::Load | Fault::LoadObservation
            )
    );
    let (index, raw) = backend.call(
        request.method().name(),
        request.arguments(),
        &request.digest(),
        |raw| IcLifecycleReply::decode(&request, raw).unwrap().digest(),
        discard,
    );
    if raw.is_none() {
        if !recovery::lifecycle(
            backend,
            index,
            &request,
            fault == Fault::LoadObservation,
            verify,
        ) {
            return Some(index);
        }
    } else {
        assert_eq!(
            verify(backend).is_some(),
            method == Method::LoadCanisterSnapshot
        );
    }
    None
}

fn extents(metadata: &IcSnapshotMetadataReply<'_>) -> Vec<SnapshotDataKind> {
    let values = metadata.metadata();
    let mut result = Vec::new();
    for (region, total) in [
        values.wasm_module_size,
        values.wasm_memory_size,
        values.stable_memory_size,
    ]
    .into_iter()
    .enumerate()
    {
        let mut offset = 0;
        while offset < total {
            let size = (total - offset).min(32 * 1024);
            result.push(match region {
                0 => SnapshotDataKind::WasmModule { offset, size },
                1 => SnapshotDataKind::WasmMemory { offset, size },
                _ => SnapshotDataKind::StableMemory { offset, size },
            });
            offset += size;
        }
    }
    result.extend(
        values
            .wasm_chunk_store
            .iter()
            .map(|chunk| SnapshotDataKind::WasmChunk {
                hash: chunk.hash.clone(),
            }),
    );
    result
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Fault {
    None,
    Capture,
    Metadata,
    Data,
    Stop,
    Load,
    Start,
    LoadObservation,
}

pub(super) fn run(fault: Fault) {
    if matches!(fault, Fault::Stop | Fault::Start | Fault::Capture) {
        let mut backend = Backend::new();
        assert!(lifecycle(&mut backend, Method::StopCanister, None, fault, |_| None).is_none());
        match fault {
            Fault::Start => {
                assert!(
                    lifecycle(&mut backend, Method::StartCanister, None, fault, |_| None).is_none()
                );
                backend.assert_state(42);
                backend.assert_calls("start_canister", 1);
            }
            Fault::Capture => {
                capture(&mut backend);
                backend.assert_calls("take_canister_snapshot", 1);
                backend.assert_calls("read_canister_snapshot_metadata", 0);
            }
            Fault::Stop => backend.assert_calls("stop_canister", 1),
            _ => unreachable!(),
        }
        backend.assert_local_replay(None);
        backend.complete(match fault {
            Fault::Start => "start-reply-discarded",
            Fault::Capture => "capture-reply-discarded",
            _ => "stop-reply-discarded",
        });
        return;
    }

    // Reuse the production capture/metadata/download coordinators for successful setup.
    // Each case stops at its own recovery boundary; the connected runner owns round trips.
    let CapturedSource {
        mut backend,
        plan,
        snapshot,
        raw_metadata,
        source,
    } = planned_runner::capture_source();
    let request = IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw_metadata).unwrap();
    let layout = BackupLayoutGuard::acquire(&source.root).unwrap();
    let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    let original_journal = fs::read(journal.path()).unwrap();
    let original_references = layout.restore_references().unwrap();
    let mut pending = None;
    match fault {
        Fault::Metadata | Fault::Data => {
            recover_upload(&mut backend, &plan, &journal, &metadata, fault);
        }
        Fault::Load | Fault::LoadObservation => {
            planned_runner::change_live_state(&mut backend);
            // Original native snapshot suffices for lifecycle recovery; uploaded destinations
            // and full transfer/load/start composition are checked by the connected runner.
            pending = lifecycle(
                &mut backend,
                Method::LoadCanisterSnapshot,
                Some(snapshot.id().to_vec()),
                fault,
                |backend| {
                    Some(verify_loaded(
                        backend,
                        &metadata,
                        snapshot.id(),
                        &source.chunks,
                    ))
                },
            );
            backend.assert_calls("load_canister_snapshot", 1);
            backend.assert_calls("start_canister", 1); // Only the deliberate pre-load mutation.
            status(&mut backend, CanisterStatusType::Stopped);
        }
        _ => panic!("focused recovery requires a fault"),
    }
    assert_eq!(
        journal
            .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
            .unwrap(),
        source.checksum
    );
    assert_eq!(fs::read(journal.path()).unwrap(), original_journal);
    assert_eq!(
        journal
            .read_download_manifest(&plan, &source.manifest)
            .unwrap(),
        *journal.record().unwrap()
    );
    assert_eq!(layout.restore_references().unwrap(), original_references);
    if let Some(sequence) = pending {
        backend.assert_pending_lifecycle_replay(sequence);
    }
    backend.assert_local_replay(pending);
    backend.complete(match fault {
        Fault::Metadata => "metadata-reply-discarded",
        Fault::Data => "data-reply-discarded",
        Fault::Load => "load-reply-discarded",
        Fault::LoadObservation => "load-and-status-replies-discarded",
        _ => unreachable!(),
    });
}

fn recover_upload(
    backend: &mut Backend,
    plan: &ic_backup::model::operation_plan::OperationPlanRecord,
    journal: &DownloadJournalGuard<'_>,
    metadata: &IcSnapshotMetadataReply<'_>,
    fault: Fault,
) {
    let upload = journal
        .prepare_ic_snapshot_upload_metadata(plan, TOKEN, metadata)
        .unwrap();
    let destination = allocate(backend, &upload, fault);
    backend.assert_calls("upload_canister_snapshot_metadata", 1);
    if fault == Fault::Data {
        let request =
            IcSnapshotMetadataRequest::new(&backend.target.to_text(), &destination).unwrap();
        let (_, raw) = backend.read(IcSnapshotTransferReadPayload::Metadata(&request));
        let destination_metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let kind = extents(metadata).into_iter().next().unwrap();
        let payload = journal
            .prepare_ic_snapshot_upload_data(plan, TOKEN, &upload, &destination, kind.clone())
            .unwrap();
        let (sequence, raw) = backend.call(
            payload.method(),
            payload.arguments(),
            &payload.binding_digest(),
            |_| panic!("reply is deliberately lost"),
            true,
        );
        assert!(raw.is_none());
        let read = IcSnapshotDataRequest::new(&destination_metadata, kind).unwrap();
        recovery::data(backend, sequence, &payload, &read);
        backend.assert_calls("upload_canister_snapshot_data", 1);
    } else {
        backend.assert_calls("upload_canister_snapshot_data", 0);
    }
    backend.assert_calls("load_canister_snapshot", 0);
    backend.assert_calls("start_canister", 0);
}

const TOKEN: &str = "runner-original-snapshot";

struct CapturedSource {
    backend: Backend,
    plan: ic_backup::model::operation_plan::OperationPlanRecord,
    snapshot: IcSnapshotInfo,
    raw_metadata: Vec<u8>,
    source: Downloaded,
}

struct Downloaded {
    root: std::path::PathBuf,
    checksum: ic_backup::model::artifacts::ArtifactChecksumRecord,
    manifest: ic_backup::model::artifacts::ArtifactChecksumRecord,
    chunks: Vec<Vec<u8>>,
}

fn allocate(
    backend: &mut Backend,
    metadata_upload: &ic_backup::model::ic_snapshot_upload::IcSnapshotUploadRequest<'_>,
    fault: Fault,
) -> Vec<u8> {
    let list = management(backend, Method::ListCanisterSnapshots, None);
    let (_, baseline_raw) = backend.call(
        list.method().name(),
        list.arguments(),
        &list.digest(),
        |raw| IcSnapshotReply::decode(&list, raw).unwrap().digest(),
        false,
    );
    let baseline_raw = baseline_raw.unwrap();
    let upload_baseline = IcSnapshotReply::decode(&list, &baseline_raw).unwrap();
    let (allocation_index, raw) = backend.call(
        metadata_upload.method(),
        metadata_upload.arguments(),
        &metadata_upload.binding_digest(),
        |raw| {
            IcSnapshotUploadReply::decode(metadata_upload, raw)
                .unwrap()
                .digest()
        },
        fault == Fault::Metadata,
    );
    let destination = if let Some(raw) = raw {
        let upload_reply = IcSnapshotUploadReply::decode(metadata_upload, &raw).unwrap();
        let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = upload_reply.kind() else {
            panic!("metadata allocation reply")
        };
        snapshot_id.clone()
    } else {
        recovery::allocation(
            backend,
            allocation_index,
            metadata_upload,
            &list,
            &upload_baseline,
        )
    };
    assert_ne!(
        destination,
        metadata_upload.source().request().snapshot_id()
    );
    destination
}

fn verify_destination(
    backend: &mut Backend,
    metadata: &IcSnapshotMetadataReply<'_>,
    destination: &[u8],
    original_chunks: &[Vec<u8>],
) -> ArtifactChecksumRecord {
    let kinds = extents(metadata);
    assert_eq!(
        kinds.len(),
        original_chunks.len(),
        "readback oracle must cover every declared extent",
    );
    let destination_request =
        IcSnapshotMetadataRequest::new(&backend.target.to_text(), destination).unwrap();
    let (_, destination_raw) = backend.read(IcSnapshotTransferReadPayload::Metadata(
        &destination_request,
    ));
    let destination_metadata =
        IcSnapshotMetadataReply::decode(&destination_request, &destination_raw).unwrap();
    assert_eq!(
        destination_metadata.metadata().certified_data,
        metadata.metadata().certified_data
    );
    assert_eq!(
        contract::encode_one(&destination_metadata.metadata().globals).unwrap(),
        contract::encode_one(&metadata.metadata().globals).unwrap()
    );
    let source = metadata.metadata();
    let actual = destination_metadata.metadata();
    assert_eq!(
        [
            actual.wasm_module_size,
            actual.wasm_memory_size,
            actual.stable_memory_size
        ],
        [
            source.wasm_module_size,
            source.wasm_memory_size,
            source.stable_memory_size
        ]
    );
    assert_eq!(
        contract::encode_one(&actual.wasm_chunk_store).unwrap(),
        contract::encode_one(&source.wasm_chunk_store).unwrap()
    );
    assert_eq!(
        contract::encode_one(&actual.global_timer).unwrap(),
        contract::encode_one(&source.global_timer).unwrap()
    );
    assert_eq!(
        contract::encode_one(&actual.on_low_wasm_memory_hook_status).unwrap(),
        contract::encode_one(&source.on_low_wasm_memory_hook_status).unwrap()
    );
    let mut evidence = vec![metadata.digest(), destination_metadata.digest()];
    for (kind, expected) in kinds.iter().zip(original_chunks) {
        let request = IcSnapshotDataRequest::new(&destination_metadata, kind.clone()).unwrap();
        let (_, raw) = backend.read(IcSnapshotTransferReadPayload::Data(&request));
        let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
        assert_eq!(reply.chunk(), expected);
        evidence.push(reply.digest());
    }
    ArtifactChecksumRecord::from_bytes(&serde_json::to_vec(&evidence).unwrap())
}

fn capture(backend: &mut Backend) {
    let list = management(backend, Method::ListCanisterSnapshots, None);
    let (_, baseline_raw) = backend.call(
        list.method().name(),
        list.arguments(),
        &list.digest(),
        |raw| IcSnapshotReply::decode(&list, raw).unwrap().digest(),
        false,
    );
    let baseline_raw = baseline_raw.unwrap();
    let baseline = IcSnapshotReply::decode(&list, &baseline_raw).unwrap();
    assert_eq!(baseline.snapshots().len(), 0, "fresh fixture baseline");
    let capture = management(backend, Method::TakeCanisterSnapshot, None);
    let (index, raw) = backend.call(
        capture.method().name(),
        capture.arguments(),
        &capture.digest(),
        |_| panic!("discarded capture reply never enters decoding"),
        true,
    );
    assert!(raw.is_none());
    recovery::capture(backend, index, &capture, &list, &baseline);
}

fn status(backend: &mut Backend, expected: CanisterStatusType) {
    let request = management(backend, Method::CanisterStatus, None);
    let (_, raw) = backend.call(
        request.method().name(),
        request.arguments(),
        &request.digest(),
        |raw| IcLifecycleReply::decode(&request, raw).unwrap().digest(),
        false,
    );
    let raw = raw.unwrap();
    let reply = IcLifecycleReply::decode(&request, &raw).unwrap();
    let IcLifecycleReplyKind::Status(info) = reply.kind() else {
        panic!("status projection")
    };
    assert_eq!(info.status(), expected);
    assert_eq!(
        info.controllers().principals(),
        [ic_platform::Principal::anonymous().to_text()]
    );
}

fn verify_loaded(
    backend: &mut Backend,
    metadata: &IcSnapshotMetadataReply<'_>,
    destination: &[u8],
    original_chunks: &[Vec<u8>],
) -> ArtifactChecksumRecord {
    status(backend, CanisterStatusType::Stopped);
    // Explicit fresh verification allocates another retained snapshot under a
    // distinct original plan/budget. It neither retries load nor reuses capture allowance.
    let request = management(backend, Method::TakeCanisterSnapshot, None);
    let (sequence, raw) = backend.call(
        request.method().name(),
        request.arguments(),
        &request.digest(),
        |raw| IcSnapshotReply::decode(&request, raw).unwrap().digest(),
        false,
    );
    let raw = raw.unwrap();
    let snapshot = IcSnapshotReply::decode(&request, &raw).unwrap();
    let actual = snapshot.snapshots()[0].id();
    assert_ne!(actual, destination);
    assert_ne!(actual, metadata.request().snapshot_id());
    let evidence = verify_destination(backend, metadata, actual, original_chunks);
    fs::write(backend.root.join("post-load-verification.json"), serde_json::to_vec_pretty(&serde_json::json!({"original_source_metadata":metadata.digest(),"loaded_destination":destination,"verification_operation_sequence":sequence,"verification_snapshot_id":actual,"complete_state_evidence":evidence})).unwrap()).unwrap();
    evidence
}

#[test]
fn readback_requires_complete_expected_chunks_before_management_calls() {
    let CapturedSource {
        mut backend,
        snapshot,
        raw_metadata,
        source,
        ..
    } = planned_runner::capture_source();
    let request = IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw_metadata).unwrap();
    let (_, incomplete) = source
        .chunks
        .split_last()
        .expect("fixture retains at least one expected chunk");
    let calls = backend.calls;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        verify_destination(&mut backend, &metadata, snapshot.id(), incomplete)
    }));
    assert!(result.is_err(), "incomplete byte oracle must reject");
    assert_eq!(
        backend.calls, calls,
        "refuse before another management call"
    );
}
