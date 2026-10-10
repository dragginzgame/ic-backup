//! Fresh original source-byte preparation; payloads confer no upload permission.

mod plan;
pub use plan::IcSnapshotDataUploadPreparationError;

use super::super::metrics::LocalOperation;
use super::{
    DownloadJournalGuard, File, IcSnapshotArtifactError, Mode, OFlags, REGIONS,
    check_directory_identity, hex_bytes, open_directory, unix_fs, verification::open_regular_child,
};
use crate::model::{
    ic_snapshot_data::{MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES, validate_kind},
    ic_snapshot_metadata::IcSnapshotMetadataReply,
    ic_snapshot_upload::{IcSnapshotUploadError, IcSnapshotUploadRequest},
    operation_plan::OperationPlanRecord,
};
use ic_management_canister_types::SnapshotDataKind;
use std::io::{self, Read, Seek, SeekFrom};
use std::time::Instant;
use thiserror::Error;

impl DownloadJournalGuard<'_> {
    /// Freshly verify an original published IC source and encode exact upload metadata.
    ///
    /// Requires the full retained source plan, unchanged journal and complete Durable
    /// selection. Target remains the source canister and replacement stays absent.
    /// The payload must precede its separately retained original upload plan/reservation.
    /// Stable future bytes, authentic source, actual context/permissions and same-release
    /// restore obligations stay integration-owned. Nothing is written or dispatched.
    /// # Errors
    /// Rejects original/byte/custody drift or unrepresentable original metadata.
    pub fn prepare_ic_snapshot_upload_metadata<'source>(
        &self,
        plan: &'source OperationPlanRecord,
        snapshot: &str,
        metadata: &'source IcSnapshotMetadataReply<'source>,
    ) -> Result<IcSnapshotUploadRequest<'source>, IcSnapshotUploadArtifactError> {
        let started = Instant::now();
        let result = (|| {
            let checksum = self.verify_ic_snapshot_artifact(plan, snapshot, metadata)?;
            Ok(IcSnapshotUploadRequest::metadata(
                plan, metadata, &checksum,
            )?)
        })();
        self.record_ic_snapshot_metrics(
            LocalOperation::UploadMetadata,
            started,
            result.is_ok(),
            None,
        );
        result
    }

    /// Read one bounded exact source extent/chunk and encode a distinct destination ID.
    ///
    /// Fresh full IC-tree verification before/after descriptor-relative reading binds
    /// actual bytes to the original metadata/checksum and guarded source plan/journal.
    /// This is an explicit local byte operation, not resume or a transfer-completion view.
    /// Only one <=1 MiB chunk is buffered; no aggregate region or allowance is allocated.
    /// The caller qualifies destination allocation and retains the new exact data intent
    /// before its own reservation. No source/restore reference, journal or fence changes.
    /// # Errors
    /// Rejects wrong source declaration, ranges/hash/IDs, changed/unsafe files or custody,
    /// short reads, encoding failures and mismatching retained original evidence.
    pub fn prepare_ic_snapshot_upload_data<'source>(
        &self,
        plan: &OperationPlanRecord,
        snapshot: &str,
        metadata_upload: &IcSnapshotUploadRequest<'source>,
        snapshot_id: &[u8],
        source_kind: SnapshotDataKind,
    ) -> Result<IcSnapshotUploadRequest<'source>, IcSnapshotUploadArtifactError> {
        let started = Instant::now();
        let result = (|| {
            let metadata = metadata_upload.source();
            metadata_upload.validate_data_destination(snapshot_id)?;
            validate_kind(metadata, &source_kind).map_err(IcSnapshotUploadError::from)?;
            self.require_upload_source(plan, snapshot, metadata_upload)?;
            let entry = self
                .record
                .artifact(metadata.request().target(), snapshot)
                .map_err(super::DownloadJournalError::from)
                .map_err(IcSnapshotArtifactError::from)?;
            let parent_path = self.layout.root().join("artifacts");
            let parent = open_directory(&parent_path).map_err(IcSnapshotArtifactError::from)?;
            let path = self.layout.root().join(entry.artifact_path());
            let directory = File::from(
                unix_fs::openat(
                    &parent,
                    path.file_name()
                        .ok_or(IcSnapshotArtifactError::CustodyChanged)?,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(io::Error::from)
                .map_err(IcSnapshotArtifactError::from)?,
            );
            let chunk = read_chunk(&directory, metadata, &source_kind)?;
            let payload =
                IcSnapshotUploadRequest::data(metadata_upload, snapshot_id, source_kind, &chunk)?;
            check_directory_identity(&parent_path, &parent)?;
            check_directory_identity(&path, &directory)?;
            self.require_upload_source(plan, snapshot, metadata_upload)?;
            Ok((payload, chunk.len()))
        })();
        self.record_ic_snapshot_metrics(
            LocalOperation::UploadData,
            started,
            result.is_ok(),
            result.as_ref().ok().map(|(_, bytes)| *bytes),
        );
        result.map(|(payload, _)| payload)
    }

    fn require_upload_source(
        &self,
        plan: &OperationPlanRecord,
        snapshot: &str,
        upload: &IcSnapshotUploadRequest<'_>,
    ) -> Result<(), IcSnapshotUploadArtifactError> {
        if plan.digest() != upload.source_plan().digest() {
            return Err(IcSnapshotUploadArtifactError::SourceMismatch);
        }
        let actual = self.verify_ic_snapshot_artifact(plan, snapshot, upload.source())?;
        if actual != *upload.source_checksum() {
            return Err(IcSnapshotUploadArtifactError::SourceMismatch);
        }
        Ok(())
    }
}

fn read_chunk(
    directory: &File,
    metadata: &IcSnapshotMetadataReply<'_>,
    kind: &SnapshotDataKind,
) -> Result<Vec<u8>, IcSnapshotArtifactError> {
    let values = metadata.metadata();
    let (name, length, offset, size) = match kind {
        SnapshotDataKind::WasmModule { offset, size } => (
            REGIONS[0].to_owned(),
            Some(values.wasm_module_size),
            *offset,
            Some(*size),
        ),
        SnapshotDataKind::WasmMemory { offset, size } => (
            REGIONS[1].to_owned(),
            Some(values.wasm_memory_size),
            *offset,
            Some(*size),
        ),
        SnapshotDataKind::StableMemory { offset, size } => (
            REGIONS[2].to_owned(),
            Some(values.stable_memory_size),
            *offset,
            Some(*size),
        ),
        SnapshotDataKind::WasmChunk { hash } => {
            (format!("chunk-{}.bin", hex_bytes(hash)), None, 0, None)
        }
    };
    let maximum = length.unwrap_or(MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64);
    let (mut file, original) = open_regular_child(directory, &name, length, maximum)?;
    file.seek(SeekFrom::Start(offset))?;
    let size = size.unwrap_or(original.len());
    let limit = usize::try_from(size)
        .ok()
        .filter(|size| *size <= MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES)
        .ok_or(IcSnapshotArtifactError::FileShape)?;
    let bytes =
        ic_host_artifacts::artifact::read_reader(file.take(size), limit).map_err(|error| {
            use ic_host_artifacts::artifact::ArtifactError;
            match error {
                ArtifactError::LimitExceeded { .. } => IcSnapshotArtifactError::FileShape,
                error => IcSnapshotArtifactError::Io(error.into()),
            }
        })?;
    if u64::try_from(bytes.len()).ok() != Some(size) {
        return Err(IcSnapshotArtifactError::FileShape);
    }
    Ok(bytes)
}

/// Typed local preparation failure; all original bytes, spending and references survive.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadArtifactError {
    /// Original declared source checksum differs from the freshly verified retained tree.
    #[error("snapshot upload source checksum differs")]
    SourceMismatch,
    /// Existing guarded IC artifact admission or descriptor IO failed.
    #[error(transparent)]
    Artifact(#[from] IcSnapshotArtifactError),
    /// Canonical pure bounded upload construction failed.
    #[error(transparent)]
    Upload(#[from] IcSnapshotUploadError),
}
