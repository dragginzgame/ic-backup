//! Metadata, durable source publication and source-bound uploads through public APIs.

use super::{
    Application, BackupLayoutGuard, CHUNK, Digest, DirBuilderExt, Driver,
    ExecutionStageBindingRecord, ExecutionStageGuard, ExecutionStagePredecessorRecord,
    ExecutionWorkflowRecord, IcSnapshotDownloadPlan, IcSnapshotReply, Method, OperationPlanRecord,
    PreparedUpdate, Reply, ReservedUpdate, Stage, TOKEN, error, fs, io, metadata, read,
};
use ic_backup::ports::ic_mutation::IcMutationProviderError;
use ic_backup::{
    model::{
        download_journal::DownloadArtifactRequest,
        ic_snapshot_metadata::IcSnapshotMetadataReply,
        ic_snapshot_upload::{
            IcSnapshotUploadReply, IcSnapshotUploadReplyKind, IcSnapshotUploadRequest,
        },
    },
    ops::persistence::{DownloadJournalGuard, read_operation_plan},
    workflow::{
        ic_snapshot_download::download_snapshot,
        ic_snapshot_metadata::read_snapshot_metadata,
        ic_snapshot_upload::{allocate_snapshot, upload_snapshot_data},
    },
};
use ic_backup_agent::{AgentSnapshotTransferReadProvider, AgentUpdateProvider};

// Own one freshly admitted source scope; returned journals borrow this held layout.
// No source or metadata evidence is cached between driver calls.
struct RetainedSource<'a> {
    layout: BackupLayoutGuard,
    plan: OperationPlanRecord,
    metadata: IcSnapshotMetadataReply<'a>,
}
impl<'a> RetainedSource<'a> {
    fn open(
        driver: &Driver<'_, impl Application>,
        original: &'a metadata::MetadataEvidence,
    ) -> io::Result<Self> {
        let metadata = original.decode()?;
        let layout = BackupLayoutGuard::acquire(&driver.source()).map_err(error)?;
        let plan = driver.source_plan(&layout, &metadata)?;
        Ok(Self {
            layout,
            plan,
            metadata,
        })
    }
    fn journal(&self) -> io::Result<DownloadJournalGuard<'_>> {
        DownloadJournalGuard::open(&self.layout, self.plan.digest().hash()).map_err(error)
    }

    fn upload(
        &self,
        journal: &DownloadJournalGuard<'_>,
    ) -> io::Result<IcSnapshotUploadRequest<'_>> {
        journal
            .prepare_ic_snapshot_upload_metadata(&self.plan, TOKEN, &self.metadata)
            .map_err(error)
    }
}

impl<A: Application> Driver<'_, A> {
    #[expect(
        clippy::too_many_lines,
        reason = "keep original metadata/source custody, one complete download and immutable publication in one trace"
    )]
    pub(super) async fn transfer(&self, stage: &ExecutionStageGuard<'_>) -> io::Result<Digest> {
        let request = self.metadata_request()?;
        let transport = &self.config.transport;
        let layout = stage.layout().map_err(error)?;
        let kind = Stage::from_sequence(stage.binding().stage_sequence())?;
        let mut provider = AgentSnapshotTransferReadProvider::new(transport,
            |request: &ic_backup::model::ic_snapshot_transfer_read::IcSnapshotTransferReadRequest<'_, '_>, prepared: &PreparedUpdate<'_>| {
                self.application.retain_ingress(kind, layout.root(), &ReservedUpdate::TransferRead(request), prepared)
                    .map_err(|_| ic_backup::ports::ic_observation::IcObservationProviderError::Unavailable)
            });
        let qualify = async |request: &ic_backup::model::ic_snapshot_transfer_read::IcSnapshotTransferReadRequest<'_, '_>, reply: &ic_backup::model::ic_snapshot_transfer_read::IcSnapshotTransferReadResponse| {
            self.application.qualify_reply(Reply {
                root: layout.root(), stage: kind, plan: stage.plan(),
                sequence: request.authority().binding().operation_sequence(),
                attempt: request.mutation_attempt(), request: &request.payload().digest(),
                arguments: request.payload().arguments(), bytes: &reply.input().reply,
            }).await
        };
        if stage.binding().stage_sequence() == Stage::Metadata.sequence() {
            return read_snapshot_metadata(
                stage,
                Stage::Metadata.sequence(),
                &request,
                &mut provider,
                async |_| self.application.admit(kind, stage.plan()).await.map(|_| ()),
                qualify,
            )
            .await
            .map(|(_, checkpoint)| checkpoint.learned_evidence().clone())
            .map_err(error);
        }
        let raw = self.raw(Stage::Metadata)?;
        let original = metadata::MetadataEvidence { request, raw };
        let metadata = original.decode()?;
        let download = IcSnapshotDownloadPlan::new(
            &self.capture,
            Stage::Download.sequence(),
            &metadata,
            CHUNK,
        )
        .map_err(error)?;
        let binding: ExecutionStageBindingRecord = serde_json::from_slice(&read(
            &Stage::Metadata.path(&self.root).join("stage-binding.json"),
        )?)?;
        let metadata_layout =
            BackupLayoutGuard::acquire(&Stage::Metadata.path(&self.root)).map_err(error)?;
        let metadata_plan = read_operation_plan(&metadata_layout, binding.plan()).map_err(error)?;
        if download
            .bind(
                &binding,
                &metadata_plan,
                stage.binding().predecessors().to_vec(),
            )
            .map_err(error)?
            != *stage.binding()
        {
            return Err(io::ErrorKind::InvalidData.into());
        }
        drop(metadata_layout);
        let capture = self.payload(Method::TakeCanisterSnapshot, None);
        let capture_raw = self.raw(Stage::Capture)?;
        let capture_reply = IcSnapshotReply::decode(&capture, &capture_raw).map_err(error)?;
        let snapshot = &capture_reply.snapshots()[0];
        layout
            .retain_restore(
                &self.root.join("restore/execution-workflow.json"),
                self.restore.digest().hash(),
            )
            .map_err(error)?;
        fs::DirBuilder::new()
            .mode(0o700)
            .create(layout.root().join("artifacts"))?;
        let mut journal = DownloadJournalGuard::create(
            layout,
            stage.plan().digest().hash(),
            vec![DownloadArtifactRequest {
                canister_id: self.config.target.clone(),
                snapshot_id: TOKEN.into(),
                snapshot_taken_at_timestamp: snapshot.taken_at_timestamp(),
                snapshot_total_size_bytes: snapshot.total_size(),
            }],
        )
        .map_err(error)?;
        let writer = journal
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &original.raw)
            .map_err(error)?;
        let checksum = download_snapshot(
            stage,
            &download,
            writer,
            &mut provider,
            async |_| self.application.admit(kind, stage.plan()).await.map(|_| ()),
            qualify,
        )
        .await
        .map_err(error)?;
        if journal
            .verify_ic_snapshot_artifact(stage.plan(), TOKEN, &metadata)
            .map_err(error)?
            != checksum
        {
            return Err(io::ErrorKind::InvalidData.into());
        }
        journal
            .publish_download_manifest(stage.plan())
            .map_err(error)
    }

    pub(super) fn source_plan(
        &self,
        source: &BackupLayoutGuard,
        metadata: &IcSnapshotMetadataReply<'_>,
    ) -> io::Result<OperationPlanRecord> {
        let plan =
            IcSnapshotDownloadPlan::new(&self.capture, Stage::Download.sequence(), metadata, CHUNK)
                .map_err(error)?
                .plan()
                .ok_or(io::ErrorKind::InvalidData)?
                .clone();
        read_operation_plan(source, &plan.digest()).map_err(error)
    }

    pub(super) fn destination(&self) -> io::Result<Vec<u8>> {
        let original = self.metadata_evidence()?;
        let source = RetainedSource::open(self, &original)?;
        let journal = source.journal()?;
        let upload = source.upload(&journal)?;
        let raw = self.raw(Stage::Allocation)?;
        let reply = IcSnapshotUploadReply::decode(&upload, &raw).map_err(error)?;
        let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = reply.kind() else {
            return Err(io::ErrorKind::InvalidData.into());
        };
        Ok(snapshot_id.clone())
    }

    pub(super) fn bind_upload(
        &self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> io::Result<OperationPlanRecord> {
        let original = self.metadata_evidence()?;
        let source = RetainedSource::open(self, &original)?;
        let journal = source.journal()?;
        let upload = source.upload(&journal)?;
        if Stage::from_sequence(sequence)? == Stage::Allocation {
            return Ok(self.singleton(
                workflow,
                Stage::Allocation.sequence(),
                &upload.binding_digest(),
            ));
        }
        let raw = self.raw(Stage::Allocation)?;
        let reply = IcSnapshotUploadReply::decode(&upload, &raw).map_err(error)?;
        let data = journal
            .prepare_ic_snapshot_data_upload_plan(
                workflow,
                Stage::Upload.sequence(),
                TOKEN,
                &reply,
                CHUNK,
            )
            .map_err(error)?;
        let binding: ExecutionStageBindingRecord = serde_json::from_slice(&read(
            &Stage::Allocation
                .path(&self.root)
                .join("stage-binding.json"),
        )?)?;
        let allocation =
            BackupLayoutGuard::acquire(&Stage::Allocation.path(&self.root)).map_err(error)?;
        let plan = read_operation_plan(&allocation, binding.plan()).map_err(error)?;
        data.bind(&binding, &plan, predecessors.to_vec())
            .map_err(error)?;
        Ok(data.plan().ok_or(io::ErrorKind::InvalidData)?.clone())
    }

    pub(super) async fn upload(&self, stage: &ExecutionStageGuard<'_>) -> io::Result<Digest> {
        let kind = Stage::from_sequence(stage.binding().stage_sequence())?;
        let original = self.metadata_evidence()?;
        let source = RetainedSource::open(self, &original)?;
        let journal = source.journal()?;
        let manifest = journal.record().map_err(error)?.digest();
        journal
            .read_download_manifest(&source.plan, &manifest)
            .map_err(error)?;
        journal
            .verify_ic_snapshot_artifact(&source.plan, TOKEN, &source.metadata)
            .map_err(error)?;
        let upload = source.upload(&journal)?;
        let transport = &self.config.transport;
        let root = stage.layout().map_err(error)?.root();
        let mut provider = AgentUpdateProvider::new(
            transport,
            |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                self.application
                    .retain_ingress(kind, root, request, prepared)
                    .map_err(|_| IcMutationProviderError::Unavailable)
            },
        );
        let qualify =
            async |request: &ic_backup::model::ic_snapshot_upload::IcSnapshotUploadAttempt<
                '_,
                '_,
            >,
                   reply: &ic_backup::model::ic_mutation::IcMutationAcknowledgement| {
                self.application
                    .qualify_reply(Reply {
                        root,
                        stage: kind,
                        plan: stage.plan(),
                        sequence: request.authority().binding().operation_sequence(),
                        attempt: request.mutation_attempt(),
                        request: &request.payload().binding_digest(),
                        arguments: request.payload().arguments(),
                        bytes: &reply.input().reply,
                    })
                    .await
            };
        if stage.binding().stage_sequence() == Stage::Allocation.sequence() {
            return allocate_snapshot(
                stage,
                Stage::Allocation.sequence(),
                &upload,
                &mut provider,
                async |_| self.application.admit(kind, stage.plan()).await.map(|_| ()),
                qualify,
            )
            .await
            .map(|(_, checkpoint)| checkpoint.learned_evidence().clone())
            .map_err(error);
        }
        let raw = self.raw(Stage::Allocation)?;
        let reply = IcSnapshotUploadReply::decode(&upload, &raw).map_err(error)?;
        let data = journal
            .prepare_ic_snapshot_data_upload_plan(
                &self.restore,
                Stage::Upload.sequence(),
                TOKEN,
                &reply,
                CHUNK,
            )
            .map_err(error)?;
        upload_snapshot_data(
            stage,
            &data,
            &journal,
            TOKEN,
            &mut provider,
            async |_| self.application.admit(kind, stage.plan()).await.map(|_| ()),
            qualify,
        )
        .await
        .map(|checkpoint| checkpoint.learned_evidence().clone())
        .map_err(error)
    }
}
