//! Shared real capture, metadata and transfer steps; cases retain their own assertions.

use super::{
    Fault,
    backend::Backend,
    evidence::{MetadataEvidence, retain_bytes},
    lifecycle, management, status,
};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        download_journal::DownloadArtifactRequest,
        execution_workflow::ExecutionStagePredecessorRecord,
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest},
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_data::IcSnapshotDataReply,
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadRequest, IcSnapshotTransferReadResponse,
        },
        ic_update_response::IcUpdateResponseInput,
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard,
        IcSnapshotArtifactWriter,
    },
    ports::{
        ic_mutation::{IcMutationProvider, IcMutationProviderError},
        ic_observation::IcObservationProviderError,
        ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
    },
    workflow::{
        ic_snapshot_capture::{IcSnapshotCaptureExecutionError, capture_snapshot},
        ic_snapshot_download::{IcSnapshotDownloadExecutionError, download_snapshot},
        ic_snapshot_metadata::read_snapshot_metadata,
    },
};
use ic_platform::management_canister_types::CanisterStatusType;
use std::{convert::Infallible, fs, io, os::unix::fs::DirBuilderExt, path::Path};

pub(super) async fn stopped_backend() -> Backend {
    let mut backend = Backend::new().await;
    // Actual fixture setup keeps stop/status under the original accounting owner.
    // This isolated application has no external effects or concurrent controllers.
    assert!(
        lifecycle(
            &mut backend,
            Method::StopCanister,
            None,
            Fault::None,
            async |_| None
        )
        .await
        .is_none()
    );
    status(&mut backend, CanisterStatusType::Stopped).await;
    backend
}

pub(super) fn artifact_journal<'layout>(
    layout: &'layout BackupLayoutGuard,
    plan: &OperationPlanRecord,
    target: &str,
    snapshot: &IcSnapshotInfo,
    token: &str,
    restore: &Path,
) -> io::Result<DownloadJournalGuard<'layout>> {
    fs::DirBuilder::new()
        .mode(0o700)
        .create(layout.root().join("artifacts"))?;
    // Retain the exact unfinished restore dependency before publishing its source journal.
    layout
        .retain_restore(restore, plan.digest().hash())
        .map_err(io::Error::other)?;
    DownloadJournalGuard::create(
        layout,
        plan.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: target.into(),
            snapshot_id: token.into(),
            snapshot_taken_at_timestamp: snapshot.taken_at_timestamp(),
            snapshot_total_size_bytes: snapshot.total_size(),
        }],
    )
    .map_err(io::Error::other)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReadFailure {
    None,
    Lost(usize),
    Malformed(usize),
}

impl ReadFailure {
    pub(super) fn position(self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Lost(index) | Self::Malformed(index) => Some(index),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum CaptureFailure {
    None,
    Lost,
    Malformed,
}

pub(super) struct CaptureProvider<'a> {
    pub(super) backend: &'a mut Backend,
    pub(super) failure: CaptureFailure,
    pub(super) oracle: &'a Path,
    pub(super) submitted: bool,
}
impl IcMutationProvider for CaptureProvider<'_> {
    async fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        request.validate_journal(journal).unwrap();
        assert!(!self.submitted);
        self.submitted = true;
        let payload = request.payload();
        assert_eq!(payload.method(), Method::TakeCanisterSnapshot);
        assert_eq!(payload.receiver(), "aaaaa-aa");
        assert_eq!(payload.target(), self.backend.target.to_text());
        let actual = self.backend.plan(
            &payload.digest(),
            request.authority().binding().operation_sequence(),
        );
        assert_eq!(request.plan().context(), actual.context());
        let raw = self
            .backend
            .management(payload.method().name(), payload.arguments())
            .await;
        fs::write(self.oracle, &raw).unwrap();
        if self.failure == CaptureFailure::Lost {
            return Err(IcMutationProviderError::Indeterminate);
        }
        let reply = if self.failure == CaptureFailure::Malformed {
            Vec::new()
        } else {
            raw
        };
        IcMutationAcknowledgement::new(IcUpdateResponseInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: actual.context().clone(),
            target: self.backend.target.to_text(),
            evidence: ArtifactChecksumRecord::from_bytes(&reply),
            reply,
        })
        .map_err(|_| IcMutationProviderError::Indeterminate)
    }
}

pub(super) struct ReadProvider<'a> {
    pub(super) backend: &'a mut Backend,
    pub(super) failure: ReadFailure,
    pub(super) oracle: &'a Path,
    pub(super) calls: usize,
}
impl IcSnapshotTransferReadProvider for ReadProvider<'_> {
    async fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    ) -> Result<IcSnapshotTransferReadResponse, IcObservationProviderError> {
        request.validate_journal(journal).unwrap();
        let index = self.calls;
        self.calls += 1;
        let response = self.backend.read_snapshot(request, journal).await?;
        if self.failure.position() != Some(index) {
            return Ok(response);
        }
        // Retained fixture oracle is never outcome/retry evidence for the workflow.
        fs::write(self.oracle, &response.input().reply).unwrap();
        if matches!(self.failure, ReadFailure::Lost(_)) {
            return Err(IcObservationProviderError::Indeterminate);
        }
        let mut input = response.input().clone();
        input.reply.clear();
        Ok(IcSnapshotTransferReadResponse::new(input).unwrap())
    }
}

pub(super) async fn capture(
    backend: &mut Backend,
    stage: &ExecutionStageGuard<'_>,
    failure: CaptureFailure,
) -> Result<(IcSnapshotInfo, ArtifactChecksumRecord), IcSnapshotCaptureExecutionError<Infallible>> {
    let payload = management(backend, Method::TakeCanisterSnapshot, None);
    let layout = stage.layout().unwrap();
    let oracle = layout.root().join("capture-oracle.candid");
    let mut provider = CaptureProvider {
        backend,
        failure,
        oracle: &oracle,
        submitted: false,
    };
    retain_bytes(layout, "capture-arguments.candid", payload.arguments());
    let result = capture_snapshot(stage, 0, &payload, &mut provider, async |request| {
        // Only this isolated stopped application qualifies direct control, full
        // drain/no-external-effects consistency and original ingress custody.
        assert_eq!(request.plan().context(), stage.plan().context());
        assert_eq!(request.payload().digest(), payload.digest());
        Ok::<(), Infallible>(())
    })
    .await;
    assert!(provider.submitted);
    let acknowledgement = result?;
    retain_bytes(layout, "capture.candid", &acknowledgement.input().reply);
    let reply = IcSnapshotReply::decode(&payload, &acknowledgement.input().reply).unwrap();
    assert_eq!(reply.snapshots().len(), 1);
    let mut journal =
        AttemptJournalGuard::open(layout, &stage.plan().attempt_authority(0).unwrap()).unwrap();
    let attempt = journal.record().unwrap().view().pending_mutation.unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: payload.digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: reply.digest().hash().into(),
        })
        .unwrap();
    Ok((reply.snapshots()[0].clone(), reply.digest()))
}

pub(super) async fn metadata(
    backend: &mut Backend,
    stage: &ExecutionStageGuard<'_>,
    request: IcSnapshotMetadataRequest,
) -> io::Result<(MetadataEvidence, ExecutionStagePredecessorRecord)> {
    let layout = stage.layout().unwrap();
    let (response, checkpoint) = read_snapshot_metadata(
        stage,
        7,
        &request,
        backend,
        async |read| {
            assert_eq!(read.plan().context(), stage.plan().context());
            assert_eq!(read.payload().digest(), request.digest());
            Ok::<(), Infallible>(())
        },
        async |read, response| {
            // Exact isolated ingress attribution and durable byte retention precede
            // the explicit receipt. Wire shape alone grants no qualification.
            retain_bytes(
                layout,
                "metadata-arguments.candid",
                read.payload().arguments(),
            );
            retain_bytes(layout, "metadata.candid", &response.input().reply);
            Ok::<_, Infallible>(MutationReceiptRequest {
                attempt: read.mutation_attempt(),
                request: read.payload().digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: response.input().evidence.hash().into(),
            })
        },
    )
    .await
    .map_err(io::Error::other)?;
    Ok((
        MetadataEvidence {
            request,
            raw: response.input().reply.clone(),
        },
        checkpoint,
    ))
}

pub(super) async fn download(
    backend: &mut Backend,
    stage: &ExecutionStageGuard<'_>,
    download: &IcSnapshotDownloadPlan<'_, '_>,
    writer: IcSnapshotArtifactWriter<'_, '_, '_>,
    failure: ReadFailure,
) -> Result<(ArtifactChecksumRecord, Vec<Vec<u8>>), IcSnapshotDownloadExecutionError<Infallible>> {
    let layout = stage.layout().unwrap();
    let oracle = layout.root().join("discarded-data-oracle.candid");
    let mut provider = ReadProvider {
        backend,
        failure,
        oracle: &oracle,
        calls: 0,
    };
    let mut chunks = Vec::new();
    let result = download_snapshot(
        stage,
        download,
        writer,
        &mut provider,
        async |read| {
            assert_eq!(read.plan().context(), stage.plan().context());
            Ok::<(), Infallible>(())
        },
        async |read, response| {
            let sequence = read.authority().binding().operation_sequence();
            retain_bytes(
                layout,
                &format!("data-{sequence}-arguments.candid"),
                read.payload().arguments(),
            );
            retain_bytes(
                layout,
                &format!("data-{sequence}.candid"),
                &response.input().reply,
            );
            chunks.push(
                IcSnapshotDataReply::decode(
                    &download.requests()[usize::try_from(sequence).unwrap()],
                    &response.input().reply,
                )
                .unwrap()
                .chunk()
                .to_vec(),
            );
            Ok::<_, Infallible>(MutationReceiptRequest {
                attempt: read.mutation_attempt(),
                request: read.payload().digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: response.input().evidence.hash().into(),
            })
        },
    )
    .await;
    assert_eq!(
        provider.calls,
        failure
            .position()
            .map_or(download.requests().len(), |index| index + 1)
    );
    result.map(|checksum| (checksum, chunks))
}
