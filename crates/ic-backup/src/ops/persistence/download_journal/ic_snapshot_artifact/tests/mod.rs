//! Actual private filesystem streaming and durable interruption recovery; no IC backend.

mod custody;
mod upload;
mod verification;

use super::*;
use crate::{
    model::{
        download_journal::DownloadArtifactRequest,
        ic_snapshot_data::{IcSnapshotDataRequest, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES},
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
    },
    ops::persistence::{BackupLayoutGuard, commit_artifact_directory},
    test_support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir},
};
use ic_management_canister_types::{
    ChunkHash, ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult,
};
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    process::Command,
};

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
const TOKEN: &str = "opaque-backend-token";
const INTENT: &str = "abababababababababababababababababababababababababababababababab";

fn values() -> ReadCanisterSnapshotMetadataResult {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let case = cases
        .iter()
        .find(|case| case["name"] == "data-source")
        .unwrap();
    let raw = case["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let mut values: ReadCanisterSnapshotMetadataResult = candid::decode_one(&raw).unwrap();
    values.wasm_module_size = 5;
    values.wasm_memory_size = 3;
    values.stable_memory_size = 1;
    values
}

fn root() -> PathBuf {
    let root = temp_dir("ic-backup-snapshot-artifact");
    fs::create_dir_all(root.join("artifacts")).unwrap();
    root
}

fn journal(layout: &BackupLayoutGuard, timestamp: u64) -> DownloadJournalGuard<'_> {
    DownloadJournalGuard::create(
        layout,
        INTENT,
        vec![DownloadArtifactRequest {
            canister_id: TARGET.into(),
            snapshot_id: TOKEN.into(),
            snapshot_taken_at_timestamp: timestamp,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap()
}

fn append<'a, 'b, 'c>(
    writer: IcSnapshotArtifactWriter<'a, 'b, 'c>,
    kind: SnapshotDataKind,
    chunk: Vec<u8>,
) -> Result<IcSnapshotArtifactWriter<'a, 'b, 'c>, IcSnapshotArtifactError> {
    let metadata = writer.coverage.metadata();
    let request = IcSnapshotDataRequest::new(metadata, kind).unwrap();
    let raw = candid::encode_one(ReadCanisterSnapshotDataResult { chunk }).unwrap();
    let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
    writer.append(&reply)
}

fn complete<'a, 'b, 'c>(
    writer: IcSnapshotArtifactWriter<'a, 'b, 'c>,
) -> IcSnapshotArtifactWriter<'a, 'b, 'c> {
    let hashes = writer
        .coverage
        .metadata()
        .metadata()
        .wasm_chunk_store
        .iter()
        .map(|chunk| chunk.hash.clone())
        .collect::<Vec<_>>();
    let writer = append(
        writer,
        SnapshotDataKind::WasmMemory { offset: 0, size: 3 },
        vec![7; 3],
    )
    .unwrap();
    let writer = append(
        writer,
        SnapshotDataKind::WasmModule { offset: 0, size: 2 },
        vec![8; 2],
    )
    .unwrap();
    let writer = append(
        writer,
        SnapshotDataKind::WasmChunk {
            hash: hashes[1].clone(),
        },
        vec![],
    )
    .unwrap();
    let writer = append(
        writer,
        SnapshotDataKind::StableMemory { offset: 0, size: 1 },
        vec![9],
    )
    .unwrap();
    let writer = append(
        writer,
        SnapshotDataKind::WasmModule { offset: 2, size: 3 },
        vec![10; 3],
    )
    .unwrap();
    append(
        writer,
        SnapshotDataKind::WasmChunk {
            hash: hashes[0].clone(),
        },
        vec![0, 255, 17],
    )
    .unwrap()
}

#[test]
fn interleaved_bytes_metadata_and_empty_chunks_publish_under_the_original_journal() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let mut journal = journal(&layout, metadata.metadata().taken_at_timestamp);
    let writer = journal
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    assert_eq!(
        writer.path().file_name().unwrap(),
        format!("{TARGET}.tmp").as_str()
    );
    assert_eq!(
        fs::metadata(writer.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let writer = complete(writer);
    assert!(writer.coverage().complete().is_some());
    for entry in fs::read_dir(writer.path()).unwrap() {
        assert_eq!(
            entry.unwrap().metadata().unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let expected = checksum_directory(writer.path()).unwrap();
    let checksum = writer.finish().unwrap();
    assert_eq!(checksum, expected);
    let canonical = root.join(format!("artifacts/{TARGET}"));
    assert_eq!(
        fs::read(canonical.join("wasm-module.bin")).unwrap(),
        [8, 8, 10, 10, 10]
    );
    assert_eq!(fs::read(canonical.join("wasm-memory.bin")).unwrap(), [7; 3]);
    assert_eq!(fs::read(canonical.join("stable-memory.bin")).unwrap(), [9]);
    assert_eq!(fs::read(canonical.join("metadata.candid")).unwrap(), raw);
    assert_eq!(
        fs::read(canonical.join("metadata-arguments.candid")).unwrap(),
        request.arguments()
    );
    assert_eq!(
        fs::read(canonical.join("format")).unwrap(),
        b"ic-backup/ic-snapshot-artifact/v1\n"
    );
    for chunk in [vec![0, 255, 17], vec![]] {
        let name = format!("chunk-{:x}.bin", Sha256::digest(&chunk));
        assert_eq!(fs::read(canonical.join(name)).unwrap(), chunk);
    }
    assert_eq!(checksum_directory(&canonical).unwrap(), checksum);
    let entry = &journal.record().unwrap().artifacts()[0];
    assert_eq!(entry.state(), ArtifactStateRecord::Durable);
    assert_eq!(entry.snapshot_id(), TOKEN);
    assert_eq!(entry.snapshot_total_size_bytes(), u64::MAX);
    assert_eq!(entry.checksum(), Some(&checksum));
    assert!(matches!(
        journal.stage_ic_snapshot_artifact(TOKEN, &metadata, &raw),
        Err(IcSnapshotArtifactError::OriginalMismatch)
    ));
    drop(journal);
    fs::rename(&canonical, root.join("retained-canonical")).unwrap();
    let journal = DownloadJournalGuard::open(&layout, INTENT).unwrap();
    assert!(journal.record().unwrap().resume_view().is_complete);
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one rejection fixture checks unchanged partial bytes and journal evidence across all consuming failure paths"
)]
fn partial_drop_and_coverage_rejections_retain_bytes_without_completion_or_recreation() {
    for action in [
        "drop",
        "finish",
        "gap",
        "duplicate",
        "mixed",
        "duplicate-chunk",
        "io",
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut journal = journal(&layout, metadata.metadata().taken_at_timestamp);
        let original = fs::read(journal.path()).unwrap();
        let writer = journal
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let writer = append(
            writer,
            SnapshotDataKind::WasmModule { offset: 0, size: 2 },
            vec![8; 2],
        )
        .unwrap();
        let path = writer.path().to_owned();
        match action {
            "drop" => drop(writer),
            "finish" => assert!(matches!(
                writer.finish(),
                Err(IcSnapshotArtifactError::IncompleteCoverage)
            )),
            "gap" => assert!(matches!(
                append(
                    writer,
                    SnapshotDataKind::WasmModule { offset: 3, size: 1 },
                    vec![1]
                ),
                Err(IcSnapshotArtifactError::Coverage(
                    IcSnapshotDataCoverageError::NoncontiguousRange
                ))
            )),
            "duplicate" => assert!(matches!(
                append(
                    writer,
                    SnapshotDataKind::WasmModule { offset: 0, size: 2 },
                    vec![1; 2]
                ),
                Err(IcSnapshotArtifactError::Coverage(
                    IcSnapshotDataCoverageError::NoncontiguousRange
                ))
            )),
            "mixed" => {
                let different = IcSnapshotMetadataRequest::new(TARGET, &[1]).unwrap();
                let metadata = IcSnapshotMetadataReply::decode(&different, &raw).unwrap();
                let request = IcSnapshotDataRequest::new(
                    &metadata,
                    SnapshotDataKind::StableMemory { offset: 0, size: 1 },
                )
                .unwrap();
                let raw =
                    candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![1] }).unwrap();
                let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
                assert!(matches!(
                    writer.append(&reply),
                    Err(IcSnapshotArtifactError::Coverage(
                        IcSnapshotDataCoverageError::MetadataMismatch
                    ))
                ));
            }
            "duplicate-chunk" => {
                let hash = metadata.metadata().wasm_chunk_store[1].hash.clone();
                let writer = append(
                    writer,
                    SnapshotDataKind::WasmChunk { hash: hash.clone() },
                    vec![],
                )
                .unwrap();
                assert!(matches!(
                    append(writer, SnapshotDataKind::WasmChunk { hash }, vec![]),
                    Err(IcSnapshotArtifactError::Coverage(
                        IcSnapshotDataCoverageError::DuplicateChunk
                    ))
                ));
            }
            "io" => {
                // A real read-only descriptor makes the next write fail on every Unix host.
                let mut writer = writer;
                writer.regions[0] = File::open(path.join("wasm-module.bin")).unwrap();
                assert!(matches!(
                    append(
                        writer,
                        SnapshotDataKind::WasmModule { offset: 2, size: 1 },
                        vec![1]
                    ),
                    Err(IcSnapshotArtifactError::Io(_))
                ));
            }
            _ => unreachable!(),
        }
        assert_eq!(fs::read(path.join("wasm-module.bin")).unwrap(), [8; 2]);
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        assert!(
            matches!(journal.stage_ic_snapshot_artifact(TOKEN, &metadata, &raw), Err(IcSnapshotArtifactError::Io(ref error)) if error.kind() == io::ErrorKind::AlreadyExists)
        );
        drop(journal);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn changed_unsafe_extra_and_missing_files_reject_before_any_journal_transition() {
    for action in [
        "changed",
        "truncated",
        "extra",
        "empty-directory",
        "missing",
        "symlink",
        "occupied-chunk",
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut journal = journal(&layout, metadata.metadata().taken_at_timestamp);
        let original = fs::read(journal.path()).unwrap();
        let writer = journal
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let path = writer.path().to_owned();
        if action == "occupied-chunk" {
            let hash = &metadata.metadata().wasm_chunk_store[0].hash;
            fs::write(
                path.join(format!("chunk-{}.bin", hex_bytes(hash))),
                b"original occupied bytes",
            )
            .unwrap();
            assert!(
                matches!(append(writer, SnapshotDataKind::WasmChunk { hash: hash.clone() }, vec![0, 255, 17]), Err(IcSnapshotArtifactError::Io(ref error)) if error.kind() == io::ErrorKind::AlreadyExists)
            );
        } else {
            let writer = complete(writer);
            match action {
                "changed" => fs::write(path.join("wasm-module.bin"), [0; 5]).unwrap(),
                "truncated" => fs::write(path.join("stable-memory.bin"), []).unwrap(),
                "extra" => fs::write(path.join("unexpected"), [0]).unwrap(),
                "empty-directory" => fs::create_dir(path.join("unexpected")).unwrap(),
                "missing" => fs::remove_file(path.join("metadata.candid")).unwrap(),
                "symlink" => {
                    fs::rename(path.join("wasm-memory.bin"), root.join("retained-memory")).unwrap();
                    symlink(root.join("retained-memory"), path.join("wasm-memory.bin")).unwrap();
                }
                _ => unreachable!(),
            }
            let error = writer.finish().unwrap_err();
            match action {
                "changed" | "truncated" => assert!(matches!(
                    error,
                    IcSnapshotArtifactError::Checksum(ChecksumError::ChecksumMismatch { .. })
                )),
                "extra" | "empty-directory" | "missing" => {
                    assert!(matches!(error, IcSnapshotArtifactError::UnexpectedEntry));
                }
                "symlink" => assert!(matches!(error, IcSnapshotArtifactError::Artifact(_))),
                _ => unreachable!(),
            }
        }
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        assert!(path.exists());
        assert!(!root.join(format!("artifacts/{TARGET}")).exists());
        drop(journal);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn original_mismatches_reject_before_creation_and_replaced_custody_receives_no_writes() {
    for action in [
        "raw",
        "timestamp",
        "token",
        "canonical",
        "stage",
        "parent",
        "layout",
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let timestamp = if action == "timestamp" {
            1
        } else {
            metadata.metadata().taken_at_timestamp
        };
        let mut journal = journal(&layout, timestamp);
        let original = fs::read(journal.path()).unwrap();
        if action == "canonical" {
            fs::create_dir(root.join(format!("artifacts/{TARGET}"))).unwrap();
        }
        if matches!(action, "raw" | "timestamp" | "token" | "canonical") {
            let bytes = if action == "raw" {
                b"wrong metadata".as_slice()
            } else {
                &raw
            };
            let token = if action == "token" { "wrong" } else { TOKEN };
            let error = journal
                .stage_ic_snapshot_artifact(token, &metadata, bytes)
                .unwrap_err();
            match action {
                "raw" | "timestamp" => assert!(matches!(error, IcSnapshotArtifactError::OriginalMismatch)),
                "token" => assert!(matches!(error, IcSnapshotArtifactError::Journal(DownloadJournalError::Record(crate::model::download_journal::DownloadJournalRecordError::SnapshotMismatch)))),
                "canonical" => assert!(matches!(error, IcSnapshotArtifactError::Io(ref error) if error.kind() == io::ErrorKind::AlreadyExists)),
                _ => unreachable!(),
            }
            assert!(!root.join(format!("artifacts/{TARGET}.tmp")).exists());
            assert_eq!(fs::read(journal.path()).unwrap(), original);
        } else {
            let writer = journal
                .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
                .unwrap();
            let path = writer.path().to_owned();
            let replaced = match action {
                "stage" => path.clone(),
                "parent" => root.join("artifacts"),
                "layout" => root.clone(),
                _ => unreachable!(),
            };
            let retained = replaced.with_extension("retained");
            fs::rename(&replaced, &retained).unwrap();
            fs::create_dir(&replaced).unwrap();
            let error = append(
                writer,
                SnapshotDataKind::WasmModule { offset: 0, size: 1 },
                vec![1],
            )
            .unwrap_err();
            assert!(matches!(
                error,
                IcSnapshotArtifactError::CustodyChanged
                    | IcSnapshotArtifactError::Journal(DownloadJournalError::Persistence(
                        crate::ops::persistence::PersistenceError::LayoutChanged { .. }
                    ))
            ));
            assert_eq!(fs::read_dir(&replaced).unwrap().count(), 0);
            fs::remove_dir(&replaced).unwrap();
            fs::rename(&retained, &replaced).unwrap();
            assert_eq!(
                fs::read(path.join("wasm-module.bin")).unwrap(),
                [] as [u8; 0]
            );
            assert_eq!(fs::read(journal.path()).unwrap(), original);
        }
        drop(journal);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn zero_regions_maximum_chunks_and_streamed_maximum_replies_keep_exact_bytes() {
    for lane in ["empty", "chunks", "stream"] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let mut values = values();
        values.wasm_module_size = if lane == "stream" {
            u64::try_from(MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES).unwrap() * 2
        } else {
            0
        };
        values.wasm_memory_size = 0;
        values.stable_memory_size = 0;
        let chunks = if lane == "chunks" {
            (0u16..1024)
                .map(|index| index.to_be_bytes().to_vec())
                .collect::<Vec<_>>()
        } else if lane == "stream" {
            vec![vec![42; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES]]
        } else {
            vec![]
        };
        values.wasm_chunk_store = chunks
            .iter()
            .map(|chunk| ChunkHash {
                hash: Sha256::digest(chunk).to_vec(),
            })
            .collect();
        let raw = candid::encode_one(values).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut journal = journal(&layout, metadata.metadata().taken_at_timestamp);
        let mut writer = journal
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        for offset in [0, u64::try_from(MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES).unwrap()] {
            if lane == "stream" {
                writer = append(
                    writer,
                    SnapshotDataKind::WasmModule {
                        offset,
                        size: u64::try_from(MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES).unwrap(),
                    },
                    vec![43; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES],
                )
                .unwrap();
            }
        }
        for chunk in chunks {
            writer = append(
                writer,
                SnapshotDataKind::WasmChunk {
                    hash: Sha256::digest(&chunk).to_vec(),
                },
                chunk,
            )
            .unwrap();
        }
        writer.finish().unwrap();
        assert_eq!(
            journal.record().unwrap().artifacts()[0].state(),
            ArtifactStateRecord::Durable
        );
        let canonical = root.join(format!("artifacts/{TARGET}"));
        assert_eq!(
            fs::metadata(canonical.join("wasm-module.bin"))
                .unwrap()
                .len(),
            metadata.metadata().wasm_module_size
        );
        assert_eq!(
            fs::read_dir(canonical).unwrap().count(),
            6 + metadata.metadata().wasm_chunk_store.len()
        );
        drop(journal);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn closing_publication_custody_drift_rejects_and_retains_the_completed_local_evidence() {
    for action in ["layout", "parent", "canonical"] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut journal = journal(&layout, metadata.metadata().taken_at_timestamp);
        let writer = journal
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let canonical = root.join(format!("artifacts/{TARGET}"));
        let replaced = match action {
            "layout" => root.clone(),
            "parent" => root.join("artifacts"),
            "canonical" => canonical.clone(),
            _ => unreachable!(),
        };
        let retained = replaced.with_extension("retained");
        let error = complete(writer)
            .finish_with(|journal, target, token| {
                journal.finalize_artifact(target, token)?;
                fs::rename(&replaced, &retained).unwrap();
                fs::create_dir(&replaced).unwrap();
                Ok(())
            })
            .unwrap_err();
        assert!(matches!(
            error,
            IcSnapshotArtifactError::CustodyChanged
                | IcSnapshotArtifactError::Journal(DownloadJournalError::Persistence(
                    crate::ops::persistence::PersistenceError::LayoutChanged { .. }
                ))
        ));
        assert_eq!(fs::read_dir(&replaced).unwrap().count(), 0);
        // The test alone restores its retained directory, without recopying or repairing bytes.
        fs::remove_dir(&replaced).unwrap();
        fs::rename(&retained, &replaced).unwrap();
        let entry = &journal.record().unwrap().artifacts()[0];
        assert_eq!(entry.state(), ArtifactStateRecord::Durable);
        assert_eq!(
            checksum_directory(&canonical).unwrap(),
            *entry.checksum().unwrap()
        );
        assert_eq!(
            fs::read(canonical.join("wasm-module.bin")).unwrap(),
            [8, 8, 10, 10, 10]
        );
        drop(journal);
        let reopened = DownloadJournalGuard::open(&layout, INTENT).unwrap();
        assert!(reopened.record().unwrap().resume_view().is_complete);
        drop(reopened);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn acknowledged_process_death_retains_partial_or_exact_verified_publication_recovery() {
    const CHILD: &str = "IC_BACKUP_ARTIFACT_CHILD";
    const ROOT: &str = "IC_BACKUP_ARTIFACT_ROOT";
    if let Ok(point) = std::env::var(CHILD) {
        let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut journal = journal(&layout, metadata.metadata().taken_at_timestamp);
        let writer = journal
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        if point == "partial" {
            let _writer = append(
                writer,
                SnapshotDataKind::WasmModule { offset: 0, size: 2 },
                vec![8; 2],
            )
            .unwrap();
            hold_at_acknowledged_barrier(&root);
        }
        complete(writer)
            .finish_with(|journal, target, token| {
                if point == "published" {
                    let entry = journal.record().unwrap().artifact(target, token).unwrap();
                    commit_artifact_directory(
                        &root.join(entry.staging_path()),
                        &root.join(entry.artifact_path()),
                        entry.checksum().unwrap().hash(),
                    )
                    .unwrap();
                }
                hold_at_acknowledged_barrier(&root);
            })
            .unwrap();
        unreachable!();
    }
    for point in ["partial", "verified", "published"] {
        let root = root();
        let mut child = Command::new(std::env::current_exe().unwrap()).arg("--exact")
            .arg("ops::persistence::download_journal::ic_snapshot_artifact::tests::acknowledged_process_death_retains_partial_or_exact_verified_publication_recovery")
            .arg("--nocapture").env(CHILD, point).env(ROOT, &root).spawn().unwrap();
        kill_child_at_acknowledged_barrier(&mut child, &root);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let mut journal = DownloadJournalGuard::open(&layout, INTENT).unwrap();
        let entry = &journal.record().unwrap().artifacts()[0];
        let canonical = root.join(entry.artifact_path());
        if point == "partial" {
            assert_eq!(entry.state(), ArtifactStateRecord::Created);
            assert_eq!(
                fs::read(root.join(entry.staging_path()).join("wasm-module.bin")).unwrap(),
                [8; 2]
            );
            assert!(!canonical.exists());
        } else {
            assert_eq!(entry.state(), ArtifactStateRecord::ChecksumVerified);
            let checksum = entry.checksum().unwrap().clone();
            assert_eq!(canonical.exists(), point == "published");
            journal.finalize_artifact(TARGET, TOKEN).unwrap();
            assert_eq!(checksum_directory(&canonical).unwrap(), checksum);
            assert_eq!(
                journal.record().unwrap().artifacts()[0].state(),
                ArtifactStateRecord::Durable
            );
        }
        drop(journal);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
