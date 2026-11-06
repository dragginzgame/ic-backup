//! Restore-source admission from complete original upload evidence.
use super::*;

#[test]
fn uploaded_restore_source_requires_complete_original_and_rejects_snapshot_substitution() {
    use crate::{
        model::ic_request::{
            IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord,
        },
        workflow::ic_snapshot_restore::{IcRestoreSource, IcRestoreSourceError},
    };
    for completed in [false, true] {
        with_data_stage_then(
            4,
            |stage, data, source| {
                if completed {
                    complete_data(stage, data, source)
                } else {
                    let mut journal = AttemptJournalGuard::open(
                        stage.layout().unwrap(),
                        &stage.plan().attempt_authority(0).unwrap(),
                    )
                    .unwrap();
                    journal
                        .reserve_planned_mutation(&stage.plan().digest())
                        .unwrap();
                    ExecutionStagePredecessorRecord::new(
                        7,
                        stage.binding().digest(),
                        ArtifactChecksumRecord::from_bytes(b"unsettled"),
                        ArtifactChecksumRecord::from_bytes(b"not applied"),
                    )
                }
            },
            |layout, data, source, proof| {
                let result = IcRestoreSource::uploaded(source.layout, data, layout, &proof);
                if !completed {
                    assert!(matches!(result, Err(IcRestoreSourceError::Checkpoint(_))));
                    return;
                }
                let original = result.unwrap();
                check_changed_upload_plan(source, data, layout, &proof);
                for snapshot in [data.destination().to_vec(), vec![88, 99]] {
                    let wire = IcManagementRequestRecord::new(IcManagementRequest {
                        method: IcManagementMethodRecord::LoadCanisterSnapshot,
                        target: TARGET.into(),
                        snapshot_id: Some(snapshot.clone()),
                    })
                    .unwrap();
                    let plan = crate::model::operation_plan::OperationPlanRecord::new(
                        crate::tests::support::ic_snapshot_upload::plan_request(
                            wire.digest().hash(),
                            7,
                            1,
                        ),
                    )
                    .unwrap();
                    let workflow = ExecutionWorkflowRecord::new(plan);
                    let root = root();
                    let load_layout = BackupLayoutGuard::acquire(&root).unwrap();
                    create_execution_workflow(&load_layout, &workflow).unwrap();
                    let plan = workflow.allocation().clone();
                    let sequence = plan.operations()[0].operation_sequence();
                    let binding =
                        ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![])
                            .unwrap();
                    let load = ExecutionStageGuard::prepare(&load_layout, binding, plan).unwrap();
                    let result = original.validate(&load, &wire);
                    if snapshot == data.destination() {
                        result.unwrap();
                    } else {
                        assert!(matches!(result, Err(IcRestoreSourceError::Mismatch)));
                    }
                    let wrong_target = IcManagementRequestRecord::new(IcManagementRequest {
                        method: wire.method(),
                        target: "aaaaa-aa".into(),
                        snapshot_id: Some(snapshot),
                    })
                    .unwrap();
                    assert!(matches!(
                        original.validate(&load, &wrong_target),
                        Err(IcRestoreSourceError::Mismatch)
                    ));
                    assert_eq!(
                        read_execution_progress(load.layout().unwrap(), &load.plan().digest())
                            .unwrap()
                            .attempts
                            .mutations_used,
                        0
                    );
                }
            },
        );
    }
}

pub(super) fn check_empty_restore_source(
    source: &DownloadJournalGuard<'_>,
    workflow: &ExecutionWorkflowRecord,
    data: &IcSnapshotDataUploadPlan<'_, '_, '_>,
    allocation: &IcSnapshotUploadReply<'_, '_>,
) {
    use crate::workflow::ic_snapshot_restore::{IcRestoreSource, IcRestoreSourceError};
    for matches in [false, true] {
        let path = root();
        let layout = BackupLayoutGuard::acquire(&path).unwrap();
        create_execution_workflow(&layout, workflow).unwrap();
        let plan = crate::model::operation_plan::OperationPlanRecord::new(
            crate::tests::support::ic_snapshot_upload::plan_request(
                data.metadata().binding_digest().hash(),
                0,
                0,
            ),
        )
        .unwrap();
        let binding = ExecutionStageBindingRecord::new(workflow, 0, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
        let evidence = if matches {
            allocation.digest()
        } else {
            ArtifactChecksumRecord::from_bytes(b"different allocation reply")
        };
        crate::tests::support::execution_workflow::record_applied(&stage, 0, evidence.hash());
        // Learned evidence is not authenticated merely by checkpoint replay.
        let completed = stage.reconcile_checkpoint(allocation.digest()).unwrap();
        drop(stage);
        let result = IcRestoreSource::uploaded(source.layout, data, &layout, &completed);
        if matches {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(IcRestoreSourceError::Mismatch)));
        }
    }
}

fn check_changed_upload_plan(
    source: &DownloadJournalGuard<'_>,
    original: &IcSnapshotDataUploadPlan<'_, '_, '_>,
    layout: &BackupLayoutGuard,
    completed: &ExecutionStagePredecessorRecord,
) {
    use crate::workflow::ic_snapshot_restore::{IcRestoreSource, IcRestoreSourceError};
    let workflow =
        crate::ops::persistence::read_execution_workflow(layout, &original.workflow_digest())
            .unwrap();
    let wire = unhex("4449444c026c01b6b897890f016d7b0100031500ff");
    let allocation = IcSnapshotUploadReply::decode(original.metadata(), &wire).unwrap();
    let different = source
        .prepare_ic_snapshot_data_upload_plan(&workflow, 7, TOKEN, &allocation, 2)
        .unwrap();
    assert_ne!(different.plan(), original.plan());
    assert!(matches!(
        IcRestoreSource::uploaded(source.layout, &different, layout, completed),
        Err(IcRestoreSourceError::Planning(_))
    ));
}
