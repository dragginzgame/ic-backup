//! Real bounded downloads under one original workflow; this is an isolated test driver.

use super::stage;
use super::{backend::Backend, management};
use ic_backup::workflow::ic_snapshot_capture::IcSnapshotCaptureReplyError;
use ic_backup::workflow::ic_snapshot_transfer_read::IcSnapshotTransferReadReplyError;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
        },
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, ExecutionStageGuard, create_execution_workflow,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_observation::IcObservationProviderError},
    workflow::{
        ic_snapshot_capture::{IcSnapshotCaptureExecutionError, capture_snapshot},
        ic_snapshot_metadata::{IcSnapshotMetadataExecutionError, read_snapshot_metadata},
        ic_snapshot_transfer_read::IcSnapshotTransferReadExecutionError,
    },
};
use std::{convert::Infallible, fs, os::unix::fs::DirBuilderExt};

pub(crate) use super::steps::{CaptureFailure, ReadFailure};
use super::steps::{CaptureProvider, ReadProvider};

pub(super) fn workflow(backend: &Backend) -> ExecutionWorkflowRecord {
    stage::catalog(
        backend,
        &ArtifactChecksumRecord::from_bytes(b"capture contract"),
    )
}
async fn capture(
    backend: &mut Backend,
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    failure: CaptureFailure,
) -> Option<(
    ic_backup::model::ic_snapshot_reply::IcSnapshotInfo,
    ExecutionStagePredecessorRecord,
)> {
    let payload = management(backend, Method::TakeCanisterSnapshot, None);
    let plan = backend.plan(&payload.digest(), 0);
    let binding = ExecutionStageBindingRecord::new(workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(layout, binding.clone(), plan).unwrap();
    let stage_layout = stage.layout().unwrap();
    stage_layout
        .retain_restore(
            &backend.root.join("unfinished-capture-reference.json"),
            stage.plan().digest().hash(),
        )
        .unwrap();
    let references = stage_layout.restore_references().unwrap();
    let calls = backend.calls;
    let result = super::steps::capture(backend, &stage, failure).await;
    let journal =
        AttemptJournalGuard::open(stage_layout, &stage.plan().attempt_authority(0).unwrap())
            .unwrap();
    if failure != CaptureFailure::None {
        if failure == CaptureFailure::Lost {
            assert!(matches!(
                result,
                Err(IcSnapshotCaptureExecutionError::Provider(
                    IcMutationProviderError::Indeterminate
                ))
            ));
        } else {
            let Err(IcSnapshotCaptureExecutionError::AfterReply {
                source: IcSnapshotCaptureReplyError::Association(_),
                acknowledgement,
            }) = result
            else {
                panic!("retained malformed capture acknowledgement")
            };
            assert_eq!(acknowledgement.input().reply, [] as [u8; 0]);
        }
        assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
        assert!(!journal.record().unwrap().view().applied);
        let original = fs::read(journal.path()).unwrap();
        drop(journal);
        assert!(
            stage
                .reconcile_checkpoint(ArtifactChecksumRecord::from_bytes(b"not qualified"))
                .is_err()
        );
        drop(stage);
        let (stage, progress) =
            ExecutionStageGuard::resume(layout, &workflow.digest(), 0, &binding.digest()).unwrap();
        assert_eq!(progress.attempts.mutations_used, 1);
        assert_eq!(progress.applied_operations, 0);
        let oracle = stage.layout().unwrap().root().join("capture-oracle.candid");
        let mut provider = CaptureProvider {
            backend,
            failure,
            oracle: &oracle,
            submitted: false,
        };
        assert!(
            capture_snapshot(
                &stage,
                0,
                &payload,
                &mut provider,
                async |_| -> Result<(), Infallible> {
                    panic!("pending capture never reaches fresh admission")
                }
            )
            .await
            .is_err()
        );
        assert!(!provider.submitted);
        assert_eq!(
            fs::read(stage.layout().unwrap().root().join("attempt-0.json")).unwrap(),
            original
        );
        assert_eq!(
            stage.layout().unwrap().restore_references().unwrap(),
            references
        );
        assert_eq!(provider.backend.calls, calls + 1);
        assert!(!layout.root().join("execution-stage-7").exists());
        assert!(!layout.root().join("execution-stage-9").exists());
        return None;
    }
    drop(journal);
    let (snapshot, evidence) = result.unwrap();
    Some((snapshot, stage.reconcile_checkpoint(evidence).unwrap()))
}

async fn prepare() -> (Backend, BackupLayoutGuard, ExecutionWorkflowRecord) {
    let backend = super::steps::stopped_backend().await;
    let root = backend.root.join("planned-workflow");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let workflow = workflow(&backend);
    create_execution_workflow(&layout, &workflow).unwrap();
    (backend, layout, workflow)
}

pub(crate) async fn capture_failure(failure: CaptureFailure) {
    assert!(failure != CaptureFailure::None);
    let (mut backend, layout, workflow) = prepare().await;
    assert!(
        capture(&mut backend, &layout, &workflow, failure)
            .await
            .is_none()
    );

    backend.close().await;
}

pub(crate) async fn metadata_failure(failure: ReadFailure) {
    assert_eq!(failure.position(), Some(0));
    let (mut backend, layout, workflow) = prepare().await;
    let (snapshot, capture) = capture(&mut backend, &layout, &workflow, CaptureFailure::None)
        .await
        .unwrap();
    let payload = IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    let plan = backend.plan(&payload.digest(), 7);
    let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![capture]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan.clone()).unwrap();
    stage
        .layout()
        .unwrap()
        .retain_restore(
            &backend.root.join("unfinished-metadata-reference.json"),
            plan.digest().hash(),
        )
        .unwrap();
    let references = stage.layout().unwrap().restore_references().unwrap();
    let oracle = stage
        .layout()
        .unwrap()
        .root()
        .join("discarded-metadata-oracle.candid");
    let calls = backend.calls;
    let mut provider = ReadProvider {
        backend: &mut backend,
        failure,
        oracle: &oracle,
        calls: 0,
    };
    let result = read_snapshot_metadata(
        &stage,
        7,
        &payload,
        &mut provider,
        async |_| Ok::<_, Infallible>(()),
        async |_, _| -> Result<_, Infallible> { panic!("lost/malformed metadata cannot qualify") },
    )
    .await;
    if matches!(failure, ReadFailure::Lost(_)) {
        assert!(matches!(
            result,
            Err(IcSnapshotMetadataExecutionError::Read(
                IcSnapshotTransferReadExecutionError::Provider(
                    IcObservationProviderError::Indeterminate
                )
            ))
        ));
    } else {
        let Err(IcSnapshotMetadataExecutionError::Read(
            IcSnapshotTransferReadExecutionError::AfterReply {
                source: IcSnapshotTransferReadReplyError::Association(_),
                response,
            },
        )) = result
        else {
            panic!("retain malformed metadata")
        };
        assert_eq!(response.input().reply, [] as [u8; 0]);
    }
    assert_eq!(provider.calls, 1);
    assert_eq!(provider.backend.calls, calls + 1);
    let original = fs::read(stage.layout().unwrap().root().join("attempt-7.json")).unwrap();
    assert!(
        stage
            .reconcile_checkpoint(ArtifactChecksumRecord::from_bytes(b"no metadata outcome"))
            .is_err()
    );
    drop(stage);
    let (stage, progress) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 7, &binding.digest()).unwrap();
    assert_eq!(progress.attempts.mutations_used, 1);
    assert_eq!(progress.applied_operations, 0);
    assert!(
        read_snapshot_metadata(
            &stage,
            7,
            &payload,
            &mut provider,
            async |_| -> Result<(), Infallible> { panic!("pending read never admitted") },
            async |_, _| -> Result<_, Infallible> { panic!("no reissue") }
        )
        .await
        .is_err()
    );
    assert_eq!(provider.calls, 1);
    assert_eq!(
        fs::read(stage.layout().unwrap().root().join("attempt-7.json")).unwrap(),
        original
    );
    assert_eq!(
        stage.layout().unwrap().restore_references().unwrap(),
        references
    );
    assert!(!layout.root().join("execution-stage-9").exists());
    assert!(!stage.layout().unwrap().root().join("artifacts").exists());
    assert_eq!(provider.backend.calls, calls + 1);

    backend.close().await;
}
