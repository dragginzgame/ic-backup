//! Fresh original source-byte preparation; payloads confer no upload permission.

mod plan;
mod session;
pub use plan::IcSnapshotDataUploadPreparationError;

use super::{
    DownloadJournalGuard, File, IcSnapshotArtifactError, REGIONS, member,
    verification::open_regular_child, with_artifact_directory,
};
use crate::model::{
    ic_snapshot_data::{MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES, validate_kind},
    ic_snapshot_metadata::IcSnapshotMetadataReply,
    ic_snapshot_upload::{
        IcSnapshotDataUploadPlan, IcSnapshotUploadError, IcSnapshotUploadRequest,
    },
    operation_plan::OperationPlanRecord,
};
use ic_platform::management_canister_types::SnapshotDataKind;
use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;

impl DownloadJournalGuard<'_> {
    pub(crate) fn upload_session<'a, 'workflow, 'request, 'source>(
        &'a self,
        snapshot: &'a str,
        upload: &'a IcSnapshotDataUploadPlan<'workflow, 'request, 'source>,
    ) -> Result<
        session::UploadSession<'a, 'workflow, 'request, 'source>,
        IcSnapshotUploadArtifactError,
    > {
        session::UploadSession::new(self, snapshot, upload)
    }
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
        let checksum = self.verify_ic_snapshot_artifact(plan, snapshot, metadata)?;
        Ok(IcSnapshotUploadRequest::metadata(
            plan, metadata, &checksum,
        )?)
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
        let metadata = metadata_upload.source();
        metadata_upload.validate_data_destination(snapshot_id)?;
        validate_kind(metadata, &source_kind).map_err(IcSnapshotUploadError::from)?;
        self.with_upload_source(plan, snapshot, metadata_upload, |directory| {
            let chunk = read_chunk(directory, metadata, &source_kind)?;
            Ok(IcSnapshotUploadRequest::data(
                metadata_upload,
                snapshot_id,
                source_kind,
                &chunk,
            )?)
        })
    }

    fn with_upload_source<T>(
        &self,
        plan: &OperationPlanRecord,
        snapshot: &str,
        metadata_upload: &IcSnapshotUploadRequest<'_>,
        read: impl FnOnce(&File) -> Result<T, IcSnapshotUploadArtifactError>,
    ) -> Result<T, IcSnapshotUploadArtifactError> {
        let metadata = metadata_upload.source();
        self.require_upload_source(plan, snapshot, metadata_upload)?;
        let entry = self
            .record
            .artifact(metadata.request().target(), snapshot)
            .map_err(super::DownloadJournalError::from)
            .map_err(IcSnapshotArtifactError::from)?;
        with_artifact_directory(self.layout.root(), entry.artifact_path(), read, || {
            self.require_upload_source(plan, snapshot, metadata_upload)
        })
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
    match member::select(kind)? {
        member::Member::Region {
            region,
            offset,
            size,
        } => {
            let length = metadata.region_sizes()[region.index()];
            let (mut file, _) =
                open_regular_child(directory, REGIONS[region.index()], Some(length), length)?;
            file.seek(SeekFrom::Start(offset))?;
            read_chunk_bytes(file, size)
        }
        member::Member::Chunk { name } => {
            let (mut file, original) = open_regular_child(
                directory,
                &name,
                None,
                MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
            )?;
            file.seek(SeekFrom::Start(0))?;
            read_chunk_bytes(file, original.len())
        }
    }
}

// Both individual preparation and batch binding buffer one exact admitted chunk.
// Full-tree verification and closing custody checks remain with their callers.
fn read_chunk_bytes(reader: impl Read, size: u64) -> Result<Vec<u8>, IcSnapshotArtifactError> {
    #[cfg(test)]
    let reader = super::read_count::Reader(reader);
    let limit = usize::try_from(size)
        .ok()
        .filter(|size| *size <= MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES)
        .ok_or(IcSnapshotArtifactError::FileShape)?;
    ic_host_artifacts::artifact::read_exact_reader(reader.take(size), limit).map_err(|error| {
        use ic_host_artifacts::artifact::ArtifactError;
        match error {
            ArtifactError::LimitExceeded { .. } | ArtifactError::LengthMismatch { .. } => {
                IcSnapshotArtifactError::FileShape
            }
            error => IcSnapshotArtifactError::Io(error.into()),
        }
    })
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
