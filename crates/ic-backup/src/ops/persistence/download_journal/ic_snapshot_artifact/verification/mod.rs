//! Explicit fresh local checks of retained opt-in IC trees; no upload permission.

use super::super::metrics::LocalOperation;
use super::{
    ArtifactChecksumRecord, DownloadJournalGuard, FORMAT, File, IcSnapshotArtifactError,
    IcSnapshotMetadataReply, MAX_IC_SNAPSHOT_METADATA_BYTES, Mode, OFlags, REGIONS,
    check_closed_tree, check_directory_identity, checksum_relative_files, errno_to_io, hex_bytes,
    open_directory, unix_fs,
};
use crate::{
    model::{
        ic_snapshot_data::MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES, operation_plan::OperationPlanRecord,
    },
    ops::{
        artifacts::checksum_reader,
        persistence::{DownloadIntegrityError, read_operation_plan},
    },
    policy::download_integrity::validate,
};
use std::time::Instant;
use std::{io::Read, os::unix::fs::MetadataExt};

impl DownloadJournalGuard<'_> {
    /// Explicitly verify one published IC tree against exact original metadata and intent.
    ///
    /// Requires the retained full plan, unchanged held journal and complete Durable
    /// selected set. Only this target's artifact bytes are read. The fixed format,
    /// original metadata/request hashes, region lengths, known bounded chunk hashes
    /// and closed direct-child tree must match the retained checksum. No metadata
    /// defaults, generic-token/raw-ID inference or progress reconstruction occurs.
    ///
    /// Reads use no-follow descriptors and a bounded checksum buffer; journal and
    /// directory identities are rechecked at closing. These sequential observations
    /// cannot fence noncooperating writers or hold fresh byte custody after return.
    /// The returned checksum is passive local evidence. Integrations still qualify
    /// authentic complete transfer, token association, fresh permission/accounting,
    /// stable byte/command custody and upload/load/start safety. Nothing is written,
    /// spent, settled or released; ordinary resume never invokes this check.
    ///
    /// # Errors
    /// Rejects wrong original evidence, incomplete selection, changed records/custody,
    /// missing/unsafe/extra children, wrong bounded lengths/hashes and IO failure.
    pub fn verify_ic_snapshot_artifact(
        &self,
        plan: &OperationPlanRecord,
        snapshot: &str,
        metadata: &IcSnapshotMetadataReply<'_>,
    ) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
        let started = Instant::now();
        let result = (|| {
            self.admit_ic_artifact_original(plan)?;
            let entry = self
                .record
                .artifact(metadata.request().target(), snapshot)
                .map_err(super::DownloadJournalError::from)?;
            if entry.snapshot_taken_at_timestamp() != metadata.metadata().taken_at_timestamp {
                return Err(IcSnapshotArtifactError::OriginalMismatch);
            }
            let expected = entry
                .checksum()
                .ok_or(IcSnapshotArtifactError::OriginalMismatch)?;
            self.check_artifact_parent()?;
            let parent_path = self.layout.root().join("artifacts");
            let parent = open_directory(&parent_path)?;
            let path = self.layout.root().join(entry.artifact_path());
            let directory = File::from(
                unix_fs::openat(
                    &parent,
                    path.file_name()
                        .ok_or(IcSnapshotArtifactError::CustodyChanged)?,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(errno_to_io)?,
            );
            checksum_ic_tree(&directory, metadata)?.verify(expected.hash())?;
            check_directory_identity(&parent_path, &parent)?;
            check_directory_identity(&path, &directory)?;
            self.admit_ic_artifact_original(plan)?;
            Ok(expected.clone())
        })();
        self.record_ic_snapshot_metrics(
            LocalOperation::Verification,
            started,
            result.is_ok(),
            None,
        );
        result
    }

    fn admit_ic_artifact_original(
        &self,
        plan: &OperationPlanRecord,
    ) -> Result<(), DownloadIntegrityError> {
        self.check_usable()?;
        read_operation_plan(self.layout, &plan.digest())?;
        self.require_unchanged_integrity_journal()?;
        validate(plan, &self.record)?;
        Ok(())
    }
}

fn checksum_ic_tree(
    directory: &File,
    metadata: &IcSnapshotMetadataReply<'_>,
) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
    let mut checksums = vec![
        ("format".into(), ArtifactChecksumRecord::from_bytes(FORMAT)),
        (
            "metadata.candid".into(),
            metadata.payload_checksum().clone(),
        ),
        (
            "metadata-arguments.candid".into(),
            ArtifactChecksumRecord::from_bytes(metadata.request().arguments()),
        ),
    ];
    let values = metadata.metadata();
    let sizes = [
        values.wasm_module_size,
        values.wasm_memory_size,
        values.stable_memory_size,
    ];
    // Empty expected digests here name the closed tree; the actual region hashes
    // below are admitted by the existing retained whole-tree checksum owner.
    let empty = ArtifactChecksumRecord::from_bytes(&[]);
    checksums.extend(REGIONS.map(|name| (name.into(), empty.clone())));
    for chunk in &values.wasm_chunk_store {
        checksums.push((
            format!("chunk-{}.bin", hex_bytes(&chunk.hash)).into(),
            ArtifactChecksumRecord::from_digest(
                chunk
                    .hash
                    .as_slice()
                    .try_into()
                    .map_err(|_| IcSnapshotArtifactError::OriginalMismatch)?,
            ),
        ));
    }
    check_closed_tree(directory, &checksums)?;
    for (index, (name, checksum)) in checksums.iter_mut().enumerate() {
        let (length, maximum) = match index {
            0 => (Some(FORMAT.len() as u64), FORMAT.len() as u64),
            1 => (None, MAX_IC_SNAPSHOT_METADATA_BYTES as u64),
            2 => {
                let length = metadata.request().arguments().len() as u64;
                (Some(length), length)
            }
            3..=5 => (Some(sizes[index - 3]), sizes[index - 3]),
            _ => (None, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64),
        };
        let actual = checksum_child(
            directory,
            name.to_str()
                .ok_or(IcSnapshotArtifactError::UnexpectedEntry)?,
            length,
            maximum,
        )?;
        if !(3..=5).contains(&index) {
            actual.verify(checksum.hash())?;
        }
        *checksum = actual;
    }
    check_closed_tree(directory, &checksums)?;
    Ok(checksum_relative_files(checksums))
}

fn checksum_child(
    directory: &File,
    name: &str,
    length: Option<u64>,
    maximum: u64,
) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let (mut file, original) = open_regular_child(directory, name, length, maximum)?;
    // Read at most the observed length plus one, detecting shrinking/growing files
    // without an unbounded read even if a noncooperating writer changes the file.
    let limit = original
        .len()
        .checked_add(1)
        .ok_or(IcSnapshotArtifactError::FileShape)?;
    let mut bounded = (&mut file).take(limit);
    let checksum = checksum_reader(&mut bounded)?;
    if bounded.limit() != 1 {
        return Err(IcSnapshotArtifactError::FileShape);
    }
    let held = file.metadata()?;
    let current =
        File::from(unix_fs::openat(directory, name, flags, Mode::empty()).map_err(errno_to_io)?)
            .metadata()?;
    if !current.is_file()
        || original.dev() != current.dev()
        || original.ino() != current.ino()
        || original.len() != current.len()
        || original.len() != held.len()
    {
        return Err(IcSnapshotArtifactError::CustodyChanged);
    }
    Ok(checksum)
}

pub(super) fn open_regular_child(
    directory: &File,
    name: &str,
    length: Option<u64>,
    maximum: u64,
) -> Result<(File, std::fs::Metadata), IcSnapshotArtifactError> {
    let file = File::from(
        unix_fs::openat(
            directory,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(errno_to_io)?,
    );
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.len() > maximum
        || length.is_some_and(|length| length != metadata.len())
    {
        return Err(IcSnapshotArtifactError::FileShape);
    }
    Ok((file, metadata))
}
