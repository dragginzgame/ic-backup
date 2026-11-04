use super::*;

// Test fixture assembly over the same bounded reader and encoder used by complete
// upload preparation. This has no production or public single-chunk entry point.
impl DownloadJournalGuard<'_> {
    pub(in super::super) fn prepare_source_fixture_chunk<'source>(
        &self,
        plan: &OperationPlanRecord,
        snapshot: &str,
        metadata: &IcSnapshotUploadRequest<'source>,
        destination: &[u8],
        kind: ic_platform::management_canister_types::SnapshotDataKind,
    ) -> Result<IcSnapshotUploadRequest<'source>, IcSnapshotUploadArtifactError> {
        metadata.validate_data_destination(destination)?;
        crate::model::ic_snapshot_data::validate_kind(metadata.source(), &kind)
            .map_err(IcSnapshotUploadError::from)?;
        self.with_upload_source(plan, snapshot, metadata, |directory| {
            let bytes = read_chunk(directory, metadata.source(), &kind)?;
            Ok(IcSnapshotUploadRequest::data(
                metadata,
                destination,
                kind,
                &bytes,
            )?)
        })
    }
}
