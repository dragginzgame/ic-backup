//! Fresh retained format, metadata, extent and closed-tree admission on native files.

use super::file_change::Replacement;
use super::*;
use crate::{
    model::operation_plan::OperationPlanRecord,
    ops::persistence::{DownloadIntegrityError, create_operation_plan},
};
use ic_platform::candid::contract;
use serde_json::json;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VerificationFault {
    Changed,
    Truncated,
    Long,
    Format,
    Metadata,
    Arguments,
    Chunk,
    Missing,
    Extra,
    EmptyDirectory,
    Directory,
    Symlink,
    Fifo,
    HugeChunk,
    HugeMetadata,
}

impl VerificationFault {
    fn apply(self, root: &std::path::Path, directory: &std::path::Path, chunk: &std::path::Path) {
        let module = directory.join(REGIONS[0]);
        match self {
            Self::Changed => fs::write(&module, [11; 5]).unwrap(),
            Self::Truncated => fs::write(&module, [8; 4]).unwrap(),
            Self::Long => fs::write(&module, [8; 6]).unwrap(),
            Self::Format => {
                fs::write(directory.join("format"), b"incorrect format").unwrap();
            }
            Self::Metadata => {
                fs::write(directory.join("metadata.candid"), b"wrong metadata").unwrap();
            }
            Self::Arguments => fs::write(
                directory.join("metadata-arguments.candid"),
                b"wrong request",
            )
            .unwrap(),
            Self::Chunk => fs::write(chunk, [17, 255, 0]).unwrap(),
            Self::Extra => fs::write(directory.join("extra"), b"").unwrap(),
            Self::EmptyDirectory => {
                fs::create_dir(directory.join("extra-empty")).unwrap();
            }
            Self::HugeChunk | Self::HugeMetadata => {
                let path = if self == Self::HugeChunk {
                    chunk.to_owned()
                } else {
                    directory.join("metadata.candid")
                };
                File::options()
                    .write(true)
                    .open(path)
                    .unwrap()
                    .set_len(MAX_IC_SNAPSHOT_METADATA_BYTES as u64 + 1)
                    .unwrap();
            }
            Self::Missing => {
                Replacement::Missing.apply(&module, &root.join("retained-module"));
            }
            Self::Directory => {
                Replacement::Directory.apply(&module, &root.join("retained-module"));
            }
            Self::Symlink => {
                Replacement::Symlink.apply(&module, &root.join("retained-module"));
            }
            Self::Fifo => {
                Replacement::Fifo.apply(&module, &root.join("retained-module"));
            }
        }
    }
}

fn plan() -> OperationPlanRecord {
    OperationPlanRecord::new(crate::tests::support::ic_snapshot_upload::plan_request(
        &"ef".repeat(32),
        7,
        0,
    ))
    .unwrap()
}

#[test]
fn explicit_retained_verification_keeps_exact_metadata_journal_and_plan_bytes() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = contract::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let bytes = fs::read(guard.path()).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let checksum = guard.record().unwrap().artifacts()[0]
        .checksum()
        .unwrap()
        .clone();
    drop(guard);
    let guard = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    assert_eq!(
        guard
            .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
            .unwrap(),
        checksum
    );
    assert_eq!(fs::read(guard.path()).unwrap(), bytes);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    assert_eq!(
        fs::read(root.join(format!("artifacts/{TARGET}/metadata.candid"))).unwrap(),
        raw
    );
}

#[test]
fn changed_unsafe_missing_extra_and_unbounded_children_reject_without_repair() {
    for case in [
        VerificationFault::Changed,
        VerificationFault::Truncated,
        VerificationFault::Long,
        VerificationFault::Format,
        VerificationFault::Metadata,
        VerificationFault::Arguments,
        VerificationFault::Chunk,
        VerificationFault::Missing,
        VerificationFault::Extra,
        VerificationFault::EmptyDirectory,
        VerificationFault::Directory,
        VerificationFault::Symlink,
        VerificationFault::Fifo,
        VerificationFault::HugeChunk,
        VerificationFault::HugeMetadata,
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let plan = plan();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = contract::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let guard = retained(&layout, &plan, &metadata, &raw);
        let journal_bytes = fs::read(guard.path()).unwrap();
        let directory = root.join(format!("artifacts/{TARGET}"));
        let hash = &metadata.metadata().wasm_chunk_store[0].hash;
        let hash = Sha256Digest::from_bytes(hash.as_slice().try_into().unwrap());
        let chunk = directory.join(format!("chunk-{hash}.bin"));
        case.apply(&root, &directory, &chunk);
        let result = guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata);
        match case {
            VerificationFault::Changed | VerificationFault::Metadata | VerificationFault::Chunk => {
                assert!(
                    matches!(
                        result,
                        Err(IcSnapshotArtifactError::Checksum(
                            ChecksumError::ChecksumMismatch { .. }
                        ))
                    ),
                    "{case:?}: {result:?}"
                );
            }
            VerificationFault::Missing
            | VerificationFault::Extra
            | VerificationFault::EmptyDirectory => assert!(
                matches!(result, Err(IcSnapshotArtifactError::UnexpectedEntry)),
                "{case:?}: {result:?}"
            ),
            VerificationFault::Symlink => assert!(
                matches!(result, Err(IcSnapshotArtifactError::Io(_))),
                "{case:?}: {result:?}"
            ),
            VerificationFault::Truncated
            | VerificationFault::Long
            | VerificationFault::Format
            | VerificationFault::Arguments
            | VerificationFault::Directory
            | VerificationFault::Fifo
            | VerificationFault::HugeChunk
            | VerificationFault::HugeMetadata => assert!(
                matches!(result, Err(IcSnapshotArtifactError::FileShape)),
                "{case:?}: {result:?}"
            ),
        }
        assert_eq!(fs::read(guard.path()).unwrap(), journal_bytes);
        assert_eq!(
            guard.record().unwrap().artifacts()[0].state(),
            crate::model::download_journal::ArtifactStateRecord::Durable
        );
        assert!(directory.exists());
    }
}

#[test]
fn maximum_chunks_and_region_bytes_verify_without_an_aggregate_buffer() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    create_operation_plan(&layout, &plan).unwrap();
    let mut values = values();
    values.wasm_module_size = MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store = (0_u16..1024)
        .map(|value| ChunkHash {
            hash: Sha256::digest(value.to_be_bytes()).to_vec(),
        })
        .collect();
    let raw = contract::encode_one(values).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let mut guard = journal(
        &layout,
        plan.digest().hash(),
        metadata.metadata().taken_at_timestamp,
    );
    let mut writer = guard
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    writer = append(
        writer,
        SnapshotDataKind::WasmModule {
            offset: 0,
            size: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
        },
        vec![42; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES],
    )
    .unwrap();
    for value in 0_u16..1024 {
        let chunk = value.to_be_bytes().to_vec();
        writer = append(
            writer,
            SnapshotDataKind::WasmChunk {
                hash: Sha256::digest(&chunk).to_vec(),
            },
            chunk,
        )
        .unwrap();
    }
    let checksum = writer.finish().unwrap();
    assert_eq!(
        guard
            .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
            .unwrap(),
        checksum
    );
    let mut extreme = ic_platform::candid::decode_one::<ReadCanisterSnapshotMetadataResult>(
        &raw,
        crate::tests::support::WIRE_ORACLE_LIMITS,
    )
    .unwrap();
    extreme.stable_memory_size = u64::MAX;
    let extreme_raw = contract::encode_one(extreme).unwrap();
    let extreme_metadata = IcSnapshotMetadataReply::decode(&request, &extreme_raw).unwrap();
    let canonical = root.join(format!("artifacts/{TARGET}"));
    fs::write(canonical.join("metadata.candid"), &extreme_raw).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &extreme_metadata),
        Err(IcSnapshotArtifactError::FileShape)
    ));
    assert_eq!(
        fs::metadata(canonical.join("stable-memory.bin"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        guard.record().unwrap().artifacts()[0].checksum(),
        Some(&checksum)
    );
}

#[test]
fn matching_forged_tree_checksums_cannot_bypass_metadata_shape_or_chunk_identity() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ForgedTreeFault {
        Extent,
        Format,
        Metadata,
        Arguments,
        Chunk,
        ExtraDirectory,
    }
    for case in [
        ForgedTreeFault::Extent,
        ForgedTreeFault::Format,
        ForgedTreeFault::Metadata,
        ForgedTreeFault::Arguments,
        ForgedTreeFault::Chunk,
        ForgedTreeFault::ExtraDirectory,
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let plan = plan();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = contract::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut guard = retained(&layout, &plan, &metadata, &raw);
        let directory = root.join(format!("artifacts/{TARGET}"));
        let hash = &metadata.metadata().wasm_chunk_store[0].hash;
        let hash = Sha256Digest::from_bytes(hash.as_slice().try_into().unwrap());
        let path = match case {
            ForgedTreeFault::Extent => directory.join(REGIONS[0]),
            ForgedTreeFault::Format => directory.join("format"),
            ForgedTreeFault::Metadata => directory.join("metadata.candid"),
            ForgedTreeFault::Arguments => directory.join("metadata-arguments.candid"),
            ForgedTreeFault::Chunk => directory.join(format!("chunk-{hash}.bin")),
            ForgedTreeFault::ExtraDirectory => directory.join("extra-empty"),
        };
        if case == ForgedTreeFault::ExtraDirectory {
            fs::create_dir(&path).unwrap();
        } else {
            fs::write(
                &path,
                if case == ForgedTreeFault::Extent {
                    b"wrong extent".as_slice()
                } else {
                    b"wrong".as_slice()
                },
            )
            .unwrap();
        }
        let checksum = checksum_directory(&directory).unwrap();
        let mut record = serde_json::to_value(&guard.record).unwrap();
        record["artifacts"][0]["checksum"] = serde_json::to_value(&checksum).unwrap();
        guard.record = serde_json::from_value(record).unwrap();
        // Deliberately corrupt both retained declarations; shape/evidence remains independent.
        crate::tests::support::write_json_durable(&guard.path(), &guard.record).unwrap();
        let bytes = fs::read(guard.path()).unwrap();
        let result = guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata);
        match case {
            ForgedTreeFault::Extent | ForgedTreeFault::Format | ForgedTreeFault::Arguments => {
                assert!(
                    matches!(result, Err(IcSnapshotArtifactError::FileShape)),
                    "{case:?}: {result:?}"
                );
            }
            ForgedTreeFault::Metadata | ForgedTreeFault::Chunk => assert!(
                matches!(
                    result,
                    Err(IcSnapshotArtifactError::Checksum(
                        ChecksumError::ChecksumMismatch { .. }
                    ))
                ),
                "{case:?}: {result:?}"
            ),
            ForgedTreeFault::ExtraDirectory => assert!(
                matches!(result, Err(IcSnapshotArtifactError::UnexpectedEntry)),
                "{case:?}: {result:?}"
            ),
        }
        assert_eq!(fs::read(guard.path()).unwrap(), bytes);
        assert!(path.exists());
    }
}

#[test]
fn non_durable_originals_and_replaced_layout_reject_before_artifact_reads() {
    use crate::policy::download_integrity::DownloadIntegrityPolicyError;
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    create_operation_plan(&layout, &plan).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = contract::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = journal(
        &layout,
        plan.digest().hash(),
        metadata.metadata().taken_at_timestamp,
    );
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Policy(DownloadIntegrityPolicyError::NonDurableArtifact { .. })
        ))
    ));
    let original_path = root.with_extension("retained-original");
    fs::rename(&root, &original_path).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Journal(_)
        ))
    ));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    assert!(original_path.join("download-journal.json").exists());
}

#[test]
fn exact_original_metadata_request_and_retained_declarations_are_required() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = contract::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let wrong_request = IcSnapshotMetadataRequest::new(TARGET, &[17, 255, 0]).unwrap();
    let wrong_metadata = IcSnapshotMetadataReply::decode(&wrong_request, &raw).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &wrong_metadata),
        Err(IcSnapshotArtifactError::Checksum(_))
    ));
    let mut different = values();
    different.taken_at_timestamp -= 1;
    let different_raw = contract::encode_one(different).unwrap();
    let different_metadata = IcSnapshotMetadataReply::decode(&request, &different_raw).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &different_metadata),
        Err(IcSnapshotArtifactError::OriginalMismatch)
    ));
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, "other-token", &metadata),
        Err(IcSnapshotArtifactError::Journal(_))
    ));
    let mut changed_plan = serde_json::to_value(&plan).unwrap();
    changed_plan["context"]["release"] = json!("02".repeat(32));
    let changed_plan: OperationPlanRecord = serde_json::from_value(changed_plan).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&changed_plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Plan(_)
        ))
    ));
    let bytes = fs::read(guard.path()).unwrap();
    let mut changed_journal = serde_json::to_value(&guard.record).unwrap();
    changed_journal["artifacts"][0]["snapshot_total_size_bytes"] = json!(7);
    fs::write(guard.path(), serde_json::to_vec(&changed_journal).unwrap()).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::JournalChanged
        ))
    ));
    fs::write(guard.path(), &bytes).unwrap();
    fs::rename(
        root.join("operation-plan.json"),
        root.join("retained-plan.json"),
    )
    .unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Plan(_)
        ))
    ));
    assert_eq!(fs::read(guard.path()).unwrap(), bytes);
}
