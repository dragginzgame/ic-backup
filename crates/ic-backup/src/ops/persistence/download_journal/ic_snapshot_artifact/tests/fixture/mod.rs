//! Scoped native artifact setup; test sites own effects and successful cleanup.

use super::*;
use crate::{model::operation_plan::OperationPlanRecord, ops::persistence::create_operation_plan};

pub(super) struct CreatedArtifact<'a> {
    pub root: &'a PathBuf,
    pub layout: &'a BackupLayoutGuard,
    pub request: &'a IcSnapshotMetadataRequest,
    pub raw: &'a Vec<u8>,
    pub metadata: &'a IcSnapshotMetadataReply<'a>,
    pub journal: DownloadJournalGuard<'a>,
}

// Do not clean up on drop: a failed assertion must retain the original evidence.
pub(super) fn with_created(check: impl FnOnce(CreatedArtifact<'_>)) -> PathBuf {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = contract::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let journal = journal(&layout, metadata.metadata().taken_at_timestamp);
    check(CreatedArtifact {
        root: &root,
        layout: &layout,
        request: &request,
        raw: &raw,
        metadata: &metadata,
        journal,
    });
    drop(layout);
    root
}

pub(super) fn retained<'a>(
    layout: &'a BackupLayoutGuard,
    plan: &OperationPlanRecord,
    metadata: &IcSnapshotMetadataReply<'_>,
    raw: &[u8],
) -> DownloadJournalGuard<'a> {
    create_operation_plan(layout, plan).unwrap();
    let mut guard = DownloadJournalGuard::create(
        layout,
        plan.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: TARGET.into(),
            snapshot_id: TOKEN.into(),
            snapshot_taken_at_timestamp: metadata.metadata().taken_at_timestamp,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap();
    complete(
        guard
            .stage_ic_snapshot_artifact(TOKEN, metadata, raw)
            .unwrap(),
    )
    .finish()
    .unwrap();
    guard
}
