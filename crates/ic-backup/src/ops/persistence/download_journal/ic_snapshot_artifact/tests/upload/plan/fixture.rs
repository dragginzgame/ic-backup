//! Shared retained-source and original data-stage setup.
use super::*;

pub(super) fn with_data_stage(
    check: impl FnOnce(
        &ExecutionStageGuard<'_>,
        &IcSnapshotDataUploadPlan<'_, '_, '_>,
        &DownloadJournalGuard<'_>,
    ),
) {
    with_data_stage_then(4, check, |_, _, _, ()| {});
}

pub(super) fn with_data_stage_then<T>(
    chunk_bytes: u64,
    check: impl FnOnce(
        &ExecutionStageGuard<'_>,
        &IcSnapshotDataUploadPlan<'_, '_, '_>,
        &DownloadJournalGuard<'_>,
    ) -> T,
    after: impl FnOnce(
        &BackupLayoutGuard,
        &IcSnapshotDataUploadPlan<'_, '_, '_>,
        &DownloadJournalGuard<'_>,
        T,
    ),
) {
    with_source(
        upload_values(),
        |root, _layout, original, metadata, source| {
            let payload = source
                .prepare_ic_snapshot_upload_metadata(original, TOKEN, metadata)
                .unwrap();
            let workflow = data_workflow(&payload, 64);
            let path = root.join("workflow");
            fs::create_dir(&path).unwrap();
            let workflow_layout = BackupLayoutGuard::acquire(&path).unwrap();
            create_execution_workflow(&workflow_layout, &workflow).unwrap();
            let plan = crate::model::operation_plan::OperationPlanRecord::new(
                crate::tests::support::ic_snapshot_upload::plan_request(
                    payload.binding_digest().hash(),
                    0,
                    0,
                ),
            )
            .unwrap();
            let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
            let stage = ExecutionStageGuard::prepare(&workflow_layout, binding, plan).unwrap();
            let raw = unhex("4449444c026c01b6b897890f016d7b0100031500ff");
            let allocation = IcSnapshotUploadReply::decode(&payload, &raw).unwrap();
            // Local receipt/persistence setup only; authentic IC attribution is tested separately.
            crate::tests::support::execution_workflow::record_applied(
                &stage,
                0,
                allocation.digest().hash(),
            );
            let predecessor = stage.reconcile_checkpoint(allocation.digest()).unwrap();
            let data = source
                .prepare_ic_snapshot_data_upload_plan(&workflow, 7, TOKEN, &allocation, chunk_bytes)
                .unwrap();
            let binding = data
                .bind(stage.binding(), stage.plan(), vec![predecessor])
                .unwrap();
            drop(stage);
            let stage = ExecutionStageGuard::prepare(
                &workflow_layout,
                binding,
                data.plan().unwrap().clone(),
            )
            .unwrap();
            stage
                .layout()
                .unwrap()
                .retain_restore(
                    &root.join("unfinished-upload.json"),
                    original.digest().hash(),
                )
                .unwrap();
            let proof = check(&stage, &data, &source);
            drop(stage);
            after(&workflow_layout, &data, &source, proof);
        },
    );
}
