//! Scoped original download declarations; callers choose journal/writer custody and cleanup.

use super::*;
use crate::model::ic_snapshot_metadata::IcSnapshotMetadataRequest;

pub(super) struct DownloadFixture<'a> {
    pub root: &'a PathBuf,
    pub layout: &'a BackupLayoutGuard,
    pub workflow: &'a ExecutionWorkflowRecord,
    pub binding: &'a ExecutionStageBindingRecord,
    pub request: &'a IcSnapshotMetadataRequest,
    pub raw: &'a Vec<u8>,
    pub metadata: &'a IcSnapshotMetadataReply<'a>,
    pub download: &'a IcSnapshotDownloadPlan<'a, 'a>,
    pub stage: ExecutionStageGuard<'a>,
}

pub(super) fn with_download(check: impl FnOnce(DownloadFixture<'_>)) -> PathBuf {
    let workflow = workflow(256);
    let request = source();
    let raw = raw_metadata();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
    let (root, layout, binding) = prepare(&workflow, &metadata, &download);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    check(DownloadFixture {
        root: &root,
        layout: &layout,
        workflow: &workflow,
        binding: &binding,
        request: &request,
        raw: &raw,
        metadata: &metadata,
        download: &download,
        stage,
    });
    drop(layout);
    // Failed assertions retain this root; successful cleanup stays at each test site.
    root
}

pub(super) fn writer<'journal, 'layout, 'metadata>(
    journal: &'journal mut DownloadJournalGuard<'layout>,
    metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
    raw: &[u8],
) -> IcSnapshotArtifactWriter<'journal, 'layout, 'metadata> {
    journal
        .stage_ic_snapshot_artifact(TOKEN, metadata, raw)
        .unwrap()
}

// The scenarios keep typed errors, retained bytes and no-reentry checks at their boundary.
pub(super) fn assert_progress(
    stage: &ExecutionStageGuard<'_>,
    applied: usize,
    mutations: u32,
) -> crate::policy::execution_progress::ExecutionProgressView {
    let progress =
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
    assert_eq!(progress.applied_operations, applied);
    assert_eq!(progress.attempts.mutations_used, mutations);
    progress
}
