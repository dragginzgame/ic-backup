//! First real management-service journey; product terminal/release remains separate.

mod backend;
pub(super) mod planned_download;
pub(super) mod planned_upload;
mod recovery;

use backend::Backend;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        download_journal::DownloadArtifactRequest,
        ic_lifecycle_reply::{IcLifecycleReply, IcLifecycleReplyKind},
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        ic_snapshot_data::{IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_reply::IcSnapshotReply,
        ic_snapshot_transfer_read::IcSnapshotTransferReadPayload,
        ic_snapshot_upload::{IcSnapshotUploadReply, IcSnapshotUploadReplyKind},
    },
    ops::persistence::{BackupLayoutGuard, DownloadJournalGuard, create_operation_plan},
};
use ic_management_canister_types::{CanisterStatusType, SnapshotDataKind};
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
    let mut backend = Backend::new();
    assert!(lifecycle(&mut backend, Method::StopCanister, None, fault, |_| None).is_none());
    status(&mut backend, CanisterStatusType::Stopped);
    let (plan, snapshot_id, timestamp, total_size) = capture(&mut backend, fault);
    let request = IcSnapshotMetadataRequest::new(&backend.target.to_text(), &snapshot_id).unwrap();
    let (_, raw) = backend.read(IcSnapshotTransferReadPayload::Metadata(&request), false);
    let raw = raw.unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    assert_eq!(metadata.metadata().stable_memory_size, 65_536);
    assert!(metadata.metadata().wasm_memory_size >= 131_072);
    assert_eq!(metadata.metadata().certified_data, 42_u64.to_le_bytes());
    assert_eq!(metadata.metadata().wasm_chunk_store.len(), 1);
    let downloaded = download(
        &mut backend,
        &plan,
        &metadata,
        &raw,
        timestamp,
        total_size,
        false,
    )
    .unwrap();
    let root = &downloaded.root;
    let checksum = &downloaded.checksum;
    let manifest = &downloaded.manifest;
    let original_chunks = &downloaded.chunks;
    let token = "retained-original-snapshot";
    let before_replay = backend.calls;
    let layout = BackupLayoutGuard::acquire(root).unwrap();
    let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    assert_eq!(
        journal.read_download_manifest(&plan, manifest).unwrap(),
        *journal.record().unwrap()
    );
    assert_eq!(backend.calls, before_replay);
    let retained_journal = fs::read(journal.path()).unwrap();
    let retained_references = fs::read(root.join("restore-references.json")).unwrap();
    let metadata_upload = journal
        .prepare_ic_snapshot_upload_metadata(&plan, token, &metadata)
        .unwrap();
    let destination = allocate(&mut backend, &metadata_upload, fault);
    upload_data(
        &mut backend,
        &plan,
        &journal,
        &metadata_upload,
        &destination,
        fault,
    );
    verify_destination(&mut backend, &metadata, &destination, original_chunks);
    let pending = restore(
        &mut backend,
        fault,
        &metadata,
        &destination,
        original_chunks,
    );
    assert_eq!(
        journal
            .verify_ic_snapshot_artifact(&plan, token, &metadata)
            .unwrap(),
        *checksum
    );
    assert_eq!(fs::read(journal.path()).unwrap(), retained_journal);
    assert_eq!(
        journal.read_download_manifest(&plan, manifest).unwrap(),
        *journal.record().unwrap()
    );
    assert_eq!(
        fs::read(root.join("restore-references.json")).unwrap(),
        retained_references
    );
    if let Some(sequence) = pending {
        backend.assert_pending_lifecycle_replay(sequence);
    }
    backend.assert_local_replay(pending);
    let case = match fault {
        Fault::None => "normal",
        Fault::Capture => "capture-reply-discarded",
        Fault::Metadata => "metadata-reply-discarded",
        Fault::Data => "data-reply-discarded",
        Fault::Stop => "stop-reply-discarded",
        Fault::Load => "load-reply-discarded",
        Fault::Start => "start-reply-discarded",
        Fault::LoadObservation => "load-and-status-replies-discarded",
    };
    backend.complete(case, &snapshot_id, &destination, checksum, manifest);
    eprintln!(
        "PocketIC journey retained at {} ({} management calls)",
        backend.root.display(),
        backend.calls
    );
}

struct Downloaded {
    root: std::path::PathBuf,
    checksum: ic_backup::model::artifacts::ArtifactChecksumRecord,
    manifest: ic_backup::model::artifacts::ArtifactChecksumRecord,
    chunks: Vec<Vec<u8>>,
}

fn download(
    backend: &mut Backend,
    plan: &ic_backup::model::operation_plan::OperationPlanRecord,
    metadata: &IcSnapshotMetadataReply<'_>,
    raw: &[u8],
    timestamp: u64,
    total_size: u64,
    lose_first_reply: bool,
) -> Result<Downloaded, u64> {
    let kinds = extents(metadata);
    let mut original_chunks = Vec::new();
    let root = backend.root.join("backup");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("artifacts")).unwrap();
    let token = "retained-original-snapshot"; // Integration-owned raw-ID/token association.
    let (checksum, manifest) = {
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        create_operation_plan(&layout, plan).unwrap();
        layout
            .retain_restore(
                &backend.root.join("unfinished-restore.json"),
                plan.digest().hash(),
            )
            .unwrap();
        let mut journal = DownloadJournalGuard::create(
            &layout,
            plan.digest().hash(),
            vec![DownloadArtifactRequest {
                canister_id: backend.target.to_text(),
                snapshot_id: token.into(),
                snapshot_taken_at_timestamp: timestamp,
                snapshot_total_size_bytes: total_size,
            }],
        )
        .unwrap();
        let mut writer = journal
            .stage_ic_snapshot_artifact(token, metadata, raw)
            .unwrap();
        for kind in &kinds {
            let request = IcSnapshotDataRequest::new(metadata, kind.clone()).unwrap();
            let (sequence, raw) = backend.read(
                IcSnapshotTransferReadPayload::Data(&request),
                lose_first_reply && original_chunks.is_empty(),
            );
            let Some(raw) = raw else {
                return Err(sequence);
            };
            let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
            original_chunks.push(reply.chunk().to_vec());
            writer = writer.append(&reply).unwrap();
        }
        let checksum = writer.finish().unwrap();
        assert_eq!(
            journal
                .verify_ic_snapshot_artifact(plan, token, metadata)
                .unwrap(),
            checksum
        );
        let manifest = journal.publish_download_manifest(plan).unwrap();
        (checksum, manifest)
    };
    Ok(Downloaded {
        root,
        checksum,
        manifest,
        chunks: original_chunks,
    })
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

fn upload_data(
    backend: &mut Backend,
    plan: &ic_backup::model::operation_plan::OperationPlanRecord,
    journal: &DownloadJournalGuard<'_>,
    metadata_upload: &ic_backup::model::ic_snapshot_upload::IcSnapshotUploadRequest<'_>,
    destination: &[u8],
    fault: Fault,
) {
    let token = "retained-original-snapshot";
    let initial_destination_request =
        IcSnapshotMetadataRequest::new(&backend.target.to_text(), destination).unwrap();
    let (_, initial_destination_raw) = backend.call(
        initial_destination_request.method(),
        initial_destination_request.arguments(),
        &initial_destination_request.digest(),
        |raw| {
            IcSnapshotMetadataReply::decode(&initial_destination_request, raw)
                .unwrap()
                .digest()
        },
        false,
    );
    let initial_destination_raw = initial_destination_raw.unwrap();
    let initial_destination_metadata =
        IcSnapshotMetadataReply::decode(&initial_destination_request, &initial_destination_raw)
            .unwrap();
    for (extent, kind) in extents(metadata_upload.source()).iter().enumerate() {
        let upload = journal
            .prepare_ic_snapshot_upload_data(
                plan,
                token,
                metadata_upload,
                destination,
                kind.clone(),
            )
            .unwrap();
        let discard_reply = fault == Fault::Data && extent == 0;
        let (write_index, _) = backend.call(
            upload.method(),
            upload.arguments(),
            &upload.binding_digest(),
            |raw| {
                IcSnapshotUploadReply::decode(&upload, raw)
                    .unwrap()
                    .digest()
            },
            discard_reply,
        );
        if discard_reply {
            let read =
                IcSnapshotDataRequest::new(&initial_destination_metadata, kind.clone()).unwrap();
            recovery::data(backend, write_index, &upload, &read);
        }
    }
}

fn verify_destination(
    backend: &mut Backend,
    metadata: &IcSnapshotMetadataReply<'_>,
    destination: &[u8],
    original_chunks: &[Vec<u8>],
) -> ArtifactChecksumRecord {
    let destination_request =
        IcSnapshotMetadataRequest::new(&backend.target.to_text(), destination).unwrap();
    let (_, destination_raw) = backend.read(
        IcSnapshotTransferReadPayload::Metadata(&destination_request),
        false,
    );
    let destination_raw = destination_raw.unwrap();
    let destination_metadata =
        IcSnapshotMetadataReply::decode(&destination_request, &destination_raw).unwrap();
    assert_eq!(
        destination_metadata.metadata().certified_data,
        metadata.metadata().certified_data
    );
    assert_eq!(
        candid::encode_one(&destination_metadata.metadata().globals).unwrap(),
        candid::encode_one(&metadata.metadata().globals).unwrap()
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
        candid::encode_one(&actual.wasm_chunk_store).unwrap(),
        candid::encode_one(&source.wasm_chunk_store).unwrap()
    );
    assert_eq!(
        candid::encode_one(&actual.global_timer).unwrap(),
        candid::encode_one(&source.global_timer).unwrap()
    );
    assert_eq!(
        candid::encode_one(&actual.on_low_wasm_memory_hook_status).unwrap(),
        candid::encode_one(&source.on_low_wasm_memory_hook_status).unwrap()
    );
    let mut evidence = vec![metadata.digest(), destination_metadata.digest()];
    for (kind, expected) in extents(metadata).iter().zip(original_chunks) {
        let request = IcSnapshotDataRequest::new(&destination_metadata, kind.clone()).unwrap();
        let (_, raw) = backend.read(IcSnapshotTransferReadPayload::Data(&request), false);
        let raw = raw.unwrap();
        let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
        assert_eq!(reply.chunk(), expected);
        evidence.push(reply.digest());
    }
    ArtifactChecksumRecord::from_bytes(&serde_json::to_vec(&evidence).unwrap())
}

fn capture(
    backend: &mut Backend,
    fault: Fault,
) -> (
    ic_backup::model::operation_plan::OperationPlanRecord,
    Vec<u8>,
    u64,
    u64,
) {
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
        |raw| IcSnapshotReply::decode(&capture, raw).unwrap().digest(),
        fault == Fault::Capture,
    );
    let (snapshot_id, timestamp, total_size) = if let Some(raw) = raw {
        let reply = IcSnapshotReply::decode(&capture, &raw).unwrap();
        let snapshot = &reply.snapshots()[0];
        (
            snapshot.id().to_vec(),
            snapshot.taken_at_timestamp(),
            snapshot.total_size(),
        )
    } else {
        recovery::capture(backend, index, &capture, &list, &baseline)
    };
    let plan = backend.retained_plan(index);
    (plan, snapshot_id, timestamp, total_size)
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
        [candid::Principal::anonymous().to_text()]
    );
}

fn restore(
    backend: &mut Backend,
    fault: Fault,
    metadata: &IcSnapshotMetadataReply<'_>,
    destination: &[u8],
    original_chunks: &[Vec<u8>],
) -> Option<u64> {
    // Deliberate application work separates current state from the retained snapshot.
    assert!(lifecycle(backend, Method::StartCanister, None, Fault::None, |_| None).is_none());
    backend.write_state(99);
    backend.assert_state(99);
    backend.clear_fixture_chunk_store();
    assert!(lifecycle(backend, Method::StopCanister, None, Fault::None, |_| None).is_none());
    // The closed fixture's no-external-effects admission never applies generically.
    let pending = lifecycle(
        backend,
        Method::LoadCanisterSnapshot,
        Some(destination.to_vec()),
        fault,
        |backend| {
            Some(verify_loaded(
                backend,
                metadata,
                destination,
                original_chunks,
            ))
        },
    );
    if pending.is_some() {
        return pending;
    }
    assert!(lifecycle(backend, Method::StartCanister, None, fault, |_| None).is_none());
    backend.assert_state(42);
    None
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

/// Actual successful ingress with a discarded transfer-read reply stops the journey.
pub(super) fn lost_transfer_read(data: bool) {
    let mut backend = Backend::new();
    assert!(
        lifecycle(
            &mut backend,
            Method::StopCanister,
            None,
            Fault::None,
            |_| None
        )
        .is_none()
    );
    status(&mut backend, CanisterStatusType::Stopped);
    let (plan, id, timestamp, total_size) = capture(&mut backend, Fault::None);
    let request = IcSnapshotMetadataRequest::new(&backend.target.to_text(), &id).unwrap();
    let (sequence, raw) = backend.read(IcSnapshotTransferReadPayload::Metadata(&request), !data);
    if data {
        let raw = raw.unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let Err(sequence) = download(
            &mut backend,
            &plan,
            &metadata,
            &raw,
            timestamp,
            total_size,
            true,
        ) else {
            panic!("discarded data reply must stop before artifact publication");
        };
        let root = backend.root.join("backup");
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
        assert_eq!(
            journal.record().unwrap().artifacts()[0].state(),
            ic_backup::model::download_journal::ArtifactStateRecord::Created
        );
        assert!(
            !root
                .join("artifacts")
                .join(backend.target.to_text())
                .exists()
        );
        assert!(root.join("restore-references.json").is_file());
        let bytes = fs::read(journal.path()).unwrap();
        let references = fs::read(root.join("restore-references.json")).unwrap();
        drop(journal);
        drop(layout);
        backend.assert_pending_read_replay(sequence);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
        assert_eq!(fs::read(journal.path()).unwrap(), bytes);
        assert_eq!(
            fs::read(root.join("restore-references.json")).unwrap(),
            references
        );
    } else {
        assert!(raw.is_none());
        assert!(!backend.root.join("backup").exists());
        backend.assert_pending_read_replay(sequence);
    }
}
