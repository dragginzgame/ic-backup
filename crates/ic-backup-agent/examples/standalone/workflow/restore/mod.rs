//! Same-release load/start and terminal replay in the normal consumer process.

use super::{
    Application, BackupLayoutGuard, CHUNK, Digest, Driver, ExecutionStageGuard,
    ExecutionWorkflowRecord, Path, PreparedUpdate, Reply, ReservedUpdate, Stage, TOKEN, error, io,
    retain, transfer::RetainedSource,
};
use ic_backup::{
    model::restore_safety::{RestoreSafetyRequest, RestoreSafetyRequestInput},
    ops::persistence::{
        JournalLock, RestoreTerminalEvidenceProvider, create_restore_safety_requirement,
        read_complete_execution_workflow_locked,
    },
    view::restore_reference_release::TerminalRestoreEvidence,
    workflow::ic_snapshot_restore::{IcRestoreSource, restore_snapshot},
};

impl<A: Application> Driver<'_, A> {
    pub(super) fn acceptance(&self) -> io::Result<Digest> {
        let original = self.metadata_evidence()?;
        let metadata = original.decode()?;
        self.application
            .acceptance(&self.restore, &metadata, &self.destination()?)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "keep source custody, fresh safety, one authenticated dispatch and explicit receipt in one trace"
    )]
    pub(super) async fn restore_stage(
        &self,
        stage: &ExecutionStageGuard<'_>,
    ) -> io::Result<Digest> {
        let sequence = stage.binding().stage_sequence();
        let kind = Stage::from_sequence(sequence)?;
        let destination = self.destination()?;
        let accepted = if kind == Stage::Start {
            Some(self.acceptance()?)
        } else {
            None
        };
        let source_layout = BackupLayoutGuard::acquire(&self.source()).map_err(error)?;
        let original = self.metadata_evidence()?;
        let source = RetainedSource::open(self, &source_layout, &original)?;
        let source_plan = &source.plan;
        let metadata = &source.metadata;
        let journal = source.journal()?;
        journal
            .verify_ic_snapshot_artifact(source_plan, TOKEN, metadata)
            .map_err(error)?;
        let manifest = journal.record().map_err(error)?.digest();
        journal
            .read_download_manifest(source_plan, &manifest)
            .map_err(error)?;
        let restore_source = if kind == Stage::Load {
            let upload = journal
                .prepare_ic_snapshot_upload_metadata(source_plan, TOKEN, metadata)
                .map_err(error)?;
            let allocation_raw = self.raw(Stage::Allocation)?;
            let allocation = ic_backup::model::ic_snapshot_upload::IcSnapshotUploadReply::decode(
                &upload,
                &allocation_raw,
            )
            .map_err(error)?;
            let data = journal
                .prepare_ic_snapshot_data_upload_plan(
                    &self.restore,
                    Stage::Upload.sequence(),
                    TOKEN,
                    &allocation,
                    CHUNK,
                )
                .map_err(error)?;
            IcRestoreSource::uploaded(
                source.layout,
                &data,
                stage.workflow_layout(),
                &stage.binding().predecessors()[0],
            )
            .map_err(error)?
        } else {
            IcRestoreSource::existing(source.layout, source_plan, &original.request)
        };
        drop(journal);
        let payload = self.restore_payload(kind, || Ok(destination))?;
        let requirement =
            self.application
                .restore_requirement(stage.plan(), source_plan, &manifest)?;
        requirement
            .validate_plans(stage.plan(), source_plan)
            .map_err(error)?;
        if requirement.source_artifacts() != &manifest {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        create_restore_safety_requirement(
            stage.layout().map_err(error)?,
            source.layout,
            stage.plan(),
            source_plan,
            &requirement,
        )
        .map_err(error)?;
        let safety = RestoreSafetyRequest::new(
            stage.plan(),
            source_plan,
            &requirement,
            &payload,
            RestoreSafetyRequestInput {
                operation_sequence: sequence,
                challenge: self.application.admit(kind, stage.plan()).await?,
                max_remote_observations: 0,
            },
        )
        .map_err(error)?;
        let transport = &self.config.transport;
        let root = stage.layout().map_err(error)?.root();
        retain(&root.join("intent.candid"), payload.arguments())?;
        let mut provider = ic_backup_agent::AgentUpdateProvider::new(
            transport,
            |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                self.application
                    .retain_ingress(kind, root, request, prepared)
                    .map_err(|_| {
                        ic_backup::ports::ic_mutation::IcMutationProviderError::Unavailable
                    })
            },
        );
        restore_snapshot(
            stage,
            restore_source,
            &safety,
            &mut provider,
            async |request, safety| {
                self.application
                    .restore_safety(request, safety, accepted.as_ref())
                    .await
            },
            async |request, acknowledgement| {
                self.application
                    .qualify_reply(Reply {
                        root,
                        stage: kind,
                        plan: stage.plan(),
                        sequence,
                        attempt: request.mutation_attempt(),
                        request: &payload.digest(),
                        arguments: payload.arguments(),
                        bytes: &acknowledgement.input().reply,
                    })
                    .await
            },
        )
        .await
        .map(|(_, checkpoint)| checkpoint.learned_evidence().clone())
        .map_err(error)
    }

    pub(super) fn terminal(&self, layout: &BackupLayoutGuard) -> io::Result<()> {
        let journal =
            JournalLock::acquire(&layout.root().join("execution-workflow.json")).map_err(error)?;
        let accepted = self.acceptance()?;
        let source = BackupLayoutGuard::acquire(&self.source()).map_err(error)?;
        let provider = Terminal {
            driver: self,
            restore: layout,
            journal: &journal,
            source: &source,
            accepted,
        };
        source.release_restore(&journal, &provider).map_err(error)?;
        Ok(())
    }
}

struct Terminal<'a, 'application, A> {
    driver: &'a Driver<'application, A>,
    restore: &'a BackupLayoutGuard,
    journal: &'a JournalLock,
    source: &'a BackupLayoutGuard,
    accepted: Digest,
}
impl<A: Application> RestoreTerminalEvidenceProvider for Terminal<'_, '_, A> {
    type Error = io::Error;
    fn qualify_terminal(&self, path: &Path, bytes: &[u8]) -> io::Result<TerminalRestoreEvidence> {
        let workflow: ExecutionWorkflowRecord = serde_json::from_slice(bytes)?;
        if path != self.journal.journal() || workflow != self.driver.restore {
            return Err(io::ErrorKind::InvalidData.into());
        }
        read_complete_execution_workflow_locked(self.restore, self.journal, &workflow.digest())
            .map_err(error)?;
        let original = self.driver.metadata_evidence()?;
        let source = RetainedSource::open(self.driver, self.source, &original)?;
        let download = source.journal()?;
        download
            .verify_ic_snapshot_artifact(&source.plan, TOKEN, &source.metadata)
            .map_err(error)?;
        download
            .read_download_manifest(&source.plan, &download.record().map_err(error)?.digest())
            .map_err(error)?;
        // Re-admit acceptance through the already-held source journal. Calling
        // Driver::destination here would attempt to reacquire the same source lock.
        let snapshot_id = source.destination(self.driver, &download)?;
        if self
            .driver
            .application
            .acceptance(&workflow, &source.metadata, &snapshot_id)?
            != self.accepted
        {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let evidence = self
            .driver
            .application
            .terminal(&workflow, &self.accepted)?;
        if evidence.authority() != &workflow.digest() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(evidence)
    }
}
