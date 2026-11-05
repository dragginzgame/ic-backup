//! Real bounded downloads under one original workflow; this is an isolated test driver.

use super::stage::{self, Stage};
use super::{backend::Backend, management};
use ic_backup::workflow::ic_snapshot_capture::IcSnapshotCaptureReplyError;
use ic_backup::workflow::ic_snapshot_transfer_read::IcSnapshotTransferReadReplyError;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        download_journal::ArtifactStateRecord,
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
        },
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard,
        create_execution_workflow, read_execution_progress,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_observation::IcObservationProviderError},
    workflow::{
        ic_snapshot_capture::{IcSnapshotCaptureExecutionError, capture_snapshot},
        ic_snapshot_download::IcSnapshotDownloadExecutionError,
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
        &Stage::CAPTURE,
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

#[expect(
    clippy::too_many_lines,
    reason = "keep the isolated ingress, spending and retained-byte chronology in one qualification trace"
)]
pub(crate) async fn run(failure: ReadFailure) {
    let (mut backend, layout, workflow) = prepare().await;
    let (snapshot, capture) = capture(&mut backend, &layout, &workflow, CaptureFailure::None)
        .await
        .unwrap();
    let metadata_request =
        IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    let metadata_plan = backend.plan(&metadata_request.digest(), 7);
    let metadata_binding =
        ExecutionStageBindingRecord::new(&workflow, 7, &metadata_plan, vec![capture]).unwrap();
    let metadata_stage =
        ExecutionStageGuard::prepare(&layout, metadata_binding.clone(), metadata_plan.clone())
            .unwrap();
    let (evidence, original) =
        super::steps::metadata(&mut backend, &metadata_stage, metadata_request)
            .await
            .unwrap();
    let metadata_request = &evidence.request;
    let raw_metadata = &evidence.raw;
    let metadata = IcSnapshotMetadataReply::decode(metadata_request, raw_metadata).unwrap();
    drop(metadata_stage);
    let download = IcSnapshotDownloadPlan::new(&workflow, 9, &metadata, 32 * 1024).unwrap();
    let binding = download
        .bind(&metadata_binding, &metadata_plan, vec![original])
        .unwrap();
    let plan = download.plan().unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan.clone()).unwrap();
    let stage_layout = stage.layout().unwrap();
    let token = "qualified-original-raw-snapshot";
    let mut artifacts = super::steps::artifact_journal(
        stage_layout,
        plan,
        &backend.target.to_text(),
        &snapshot,
        token,
        &backend.root.join("unfinished-restore.json"),
    )
    .unwrap();
    let references = stage_layout.restore_references().unwrap();
    let writer = artifacts
        .stage_ic_snapshot_artifact(token, &metadata, raw_metadata)
        .unwrap();
    let calls_before_data = backend.calls;
    let result = super::steps::download(&mut backend, &stage, &download, writer, failure).await;
    if let Some(index) = failure.position() {
        if matches!(failure, ReadFailure::Malformed(_)) {
            let Err(IcSnapshotDownloadExecutionError::Read(
                IcSnapshotTransferReadExecutionError::AfterReply {
                    source: IcSnapshotTransferReadReplyError::Association(_),
                    response,
                },
            )) = result
            else {
                panic!("retained malformed reply")
            };
            assert_eq!(response.input().reply, [] as [u8; 0]);
        } else {
            assert!(matches!(
                result,
                Err(IcSnapshotDownloadExecutionError::Read(
                    IcSnapshotTransferReadExecutionError::Provider(
                        IcObservationProviderError::Indeterminate
                    )
                ))
            ));
        }
        let pending = u64::try_from(index).unwrap();
        let dispatched = u32::try_from(pending + 1).unwrap();
        let view = read_execution_progress(stage_layout, &plan.digest()).unwrap();
        assert_eq!(view.attempts.mutations_used, dispatched);
        assert_eq!(view.applied_operations, usize::try_from(pending).unwrap());
        assert_eq!(backend.calls, calls_before_data + u64::from(dispatched));
        assert_eq!(
            artifacts.record().unwrap().artifacts()[0].state(),
            ArtifactStateRecord::Created
        );
        // The same complete-plan owner rejects both pending reentry and its successor.
        for sequence in [pending + 1, pending] {
            let authority = plan.attempt_authority(sequence).unwrap();
            let mut journal = AttemptJournalGuard::open(stage_layout, &authority).unwrap();
            assert!(journal.reserve_planned_mutation(&plan.digest()).is_err());
        }
        let original =
            fs::read(stage_layout.root().join(format!("attempt-{pending}.json"))).unwrap();
        let artifact_bytes = fs::read(artifacts.path()).unwrap();
        let reference_bytes =
            fs::read(stage_layout.root().join("restore-references.json")).unwrap();
        assert!(!stage_layout.root().join("download-manifest.json").exists());
        assert!(
            !stage_layout
                .root()
                .join(artifacts.record().unwrap().artifacts()[0].artifact_path())
                .exists()
        );
        drop(artifacts);
        drop(stage);
        let (reopened, resumed) =
            ExecutionStageGuard::resume(&layout, &workflow.digest(), 9, &binding.digest()).unwrap();
        let current = reopened.layout().unwrap();
        assert_eq!(resumed, view);
        assert_eq!(
            fs::read(current.root().join(format!("attempt-{pending}.json"))).unwrap(),
            original
        );
        let artifacts = DownloadJournalGuard::open(current, plan.digest().hash()).unwrap();
        assert_eq!(fs::read(artifacts.path()).unwrap(), artifact_bytes);
        assert_eq!(
            fs::read(current.root().join("restore-references.json")).unwrap(),
            reference_bytes
        );
        assert_eq!(current.restore_references().unwrap(), references);
        assert_eq!(backend.calls, calls_before_data + u64::from(dispatched));
        assert!(ExecutionStageGuard::prepare(&layout, binding, plan.clone()).is_err());
        backend.close().await;
        return;
    }
    let (checksum, _) = result.unwrap();
    assert_eq!(
        artifacts
            .verify_ic_snapshot_artifact(plan, token, &metadata)
            .unwrap(),
        checksum
    );
    let manifest = artifacts.publish_download_manifest(plan).unwrap();
    let retained = artifacts.record().unwrap().clone();
    drop(artifacts);
    stage.reconcile_checkpoint(manifest.clone()).unwrap();
    let view = read_execution_progress(stage_layout, &plan.digest()).unwrap();
    assert_eq!(view.applied_operations, download.requests().len());
    assert_eq!(view.attempts.mutations_remaining, 0); // Original spare headroom is unassigned.
    assert_eq!(
        backend.calls,
        calls_before_data + u64::try_from(download.requests().len()).unwrap()
    );
    let calls = backend.calls;
    drop(stage);
    let (reopened, resumed) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 9, &binding.digest()).unwrap();
    assert_eq!(resumed, view);
    let journal =
        DownloadJournalGuard::open(reopened.layout().unwrap(), plan.digest().hash()).unwrap();
    assert_eq!(
        journal.read_download_manifest(plan, &manifest).unwrap(),
        retained
    );
    assert_eq!(
        reopened.layout().unwrap().restore_references().unwrap(),
        references
    );
    assert_eq!(backend.calls, calls);

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
