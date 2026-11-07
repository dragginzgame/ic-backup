//! One completed source artifact; scenarios own stage preparation and faults.

use super::*;

pub(super) fn with_source(
    values: ReadCanisterSnapshotMetadataResult,
    check: impl FnOnce(
        &Path,
        &BackupLayoutGuard,
        &crate::model::operation_plan::OperationPlanRecord,
        &IcSnapshotMetadataReply<'_>,
        DownloadJournalGuard<'_>,
    ),
) {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = contract::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let source = retained(&layout, &plan, &metadata, &raw);
    // Failed assertions retain the original source and all scenario evidence.
    check(&root, &layout, &plan, &metadata, source);
}
