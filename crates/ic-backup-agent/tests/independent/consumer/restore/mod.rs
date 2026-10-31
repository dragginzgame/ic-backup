//! Same-release load/start and terminal replay in the normal consumer process.

use super::evidence::{AcceptanceRecord, AdmissionRecord};
use super::*;
use ic_backup::{
    model::restore_safety::{
        RestoreSafetyEvidence, RestoreSafetyLaneRecord, RestoreSafetyObservation,
        RestoreSafetyObservationInput, RestoreSafetyRequest, RestoreSafetyRequestInput,
        RestoreSafetyRequirementRecord, RestoreSafetyRequirementRequest, TargetRestoreEvidence,
    },
    ops::persistence::{
        DownloadJournalGuard, JournalLock, RestoreReferenceReleaseOutcome,
        RestoreTerminalEvidenceProvider, create_restore_safety_requirement,
        read_complete_execution_workflow_locked,
    },
    view::restore_reference_release::TerminalRestoreEvidence,
    workflow::ic_snapshot_restore::{IcRestoreSource, restore_snapshot},
};
use ic_platform::management_canister_types::CanisterStatusType;

impl Driver {
    pub(super) fn admit(&self) -> io::Result<Digest> {
        let raw = read(&self.root.join(format!("admission-{}.json", self.mode)))?;
        let evidence: AdmissionRecord = serde_json::from_slice(&raw)?;
        let caller = ic_platform::agent::Identity::sender(
            &ic_platform::agent::BasicIdentity::from_raw_key(&[7; 32]),
        )
        .map_err(error)?
        .to_text();
        if !evidence.context.matches(&self.config.context)
            || evidence.target != self.config.target
            || evidence.controllers != [caller]
            || evidence.module_hash != self.config.context.release()
            || evidence.status != CanisterStatusType::Stopped
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        // Parent-owned fresh accounted admission is valid during this exclusive
        // phase: one controller, fixed membership, stopped code, no timers/calls.
        // The parent cannot issue another application effect until this process exits.
        Ok(Digest::from_bytes(&raw))
    }

    pub(super) fn acceptance(&self) -> io::Result<Digest> {
        let raw = read(&self.root.join("acceptance.json"))?;
        let record: AcceptanceRecord = serde_json::from_slice(&raw)?;
        let original = self.metadata_evidence()?;
        let metadata = original.decode()?;
        if !record.restore.matches(&self.restore.digest())
            || !record.source_metadata.matches(&metadata.digest())
            || record.destination != self.destination()?
            || !record.complete_state
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(Digest::from_bytes(&raw))
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
        let source = BackupLayoutGuard::acquire(&self.source()).map_err(error)?;
        let source_plan = self.source_plan(&source)?;
        let original = self.metadata_evidence()?;
        let metadata = original.decode()?;
        let journal =
            DownloadJournalGuard::open(&source, source_plan.digest().hash()).map_err(error)?;
        journal
            .verify_ic_snapshot_artifact(&source_plan, TOKEN, &metadata)
            .map_err(error)?;
        let manifest = journal.record().map_err(error)?.digest();
        journal
            .read_download_manifest(&source_plan, &manifest)
            .map_err(error)?;
        let restore_source = if kind == Stage::Load {
            let upload = journal
                .prepare_ic_snapshot_upload_metadata(&source_plan, TOKEN, &metadata)
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
                &source,
                &data,
                stage.workflow_layout(),
                &stage.binding().predecessors()[0],
            )
            .map_err(error)?
        } else {
            IcRestoreSource::existing(&source, &source_plan, &original.request)
        };
        drop(journal);
        let payload = self.payload(
            if kind == Stage::Load {
                Method::LoadCanisterSnapshot
            } else {
                Method::StartCanister
            },
            (kind == Stage::Load).then_some(destination),
        );
        let requirement = RestoreSafetyRequirementRecord::new(
            stage.plan(),
            &source_plan,
            RestoreSafetyRequirementRequest {
                source_artifacts: manifest.clone(),
                safety: RestoreSafetyLaneRecord::NoIrreversibleEffects,
                expected_fence: None,
            },
        )
        .map_err(error)?;
        create_restore_safety_requirement(
            stage.layout().map_err(error)?,
            &source,
            stage.plan(),
            &source_plan,
            &requirement,
        )
        .map_err(error)?;
        let safety = RestoreSafetyRequest::new(
            stage.plan(),
            &source_plan,
            &requirement,
            &payload,
            RestoreSafetyRequestInput {
                operation_sequence: sequence,
                challenge: self.admit()?,
                max_remote_observations: 0,
            },
        )
        .map_err(error)?;
        let transport = self.config.transport(stage.plan());
        let root = stage.layout().map_err(error)?.root();
        retain(&root.join("intent.candid"), payload.arguments())?;
        let mut provider = ic_backup_agent::AgentUpdateProvider::new(
            &transport,
            |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                signed(root, request, prepared).map_err(|_| {
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
                let evidence = self.admit()?;
                RestoreSafetyObservation::new(RestoreSafetyObservationInput {
                    request: safety.digest(),
                    context: request.plan().context().clone(),
                    inventory: request.plan().inventory().clone(),
                    source_plan_intent: source_plan.digest(),
                    source_artifacts: manifest.clone(),
                    targets: vec![TargetRestoreEvidence {
                        target: payload.target().into(),
                        state: CanisterStatusType::Stopped,
                        lifecycle_evidence: evidence.clone(),
                        restored_acceptance: accepted.clone(),
                    }],
                    safety: RestoreSafetyEvidence::NoIrreversibleEffects(
                        Digest::from_hash(self.config.context.release()).map_err(error)?,
                    ),
                    evidence,
                    remote_observations: 0,
                })
                .map_err(error)
            },
            async |request, acknowledgement| {
                if kind == Stage::Load && self.mode == Mode::LoseLoad {
                    // Deliberately die after the real authenticated load and before
                    // qualifying any receipt. Retain oracle bytes separately; cold
                    // continuation must never use them to infer Applied or redispatch.
                    retain(
                        &root.join("discarded-oracle.candid"),
                        &acknowledgement.input().reply,
                    )?;
                    std::process::exit(75);
                }
                receipt(
                    root,
                    sequence,
                    request.mutation_attempt(),
                    &payload.digest(),
                    payload.arguments(),
                    &acknowledgement.input().reply,
                )
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
        let expected = if source.has_restore_references().map_err(error)? {
            RestoreReferenceReleaseOutcome::Released
        } else {
            RestoreReferenceReleaseOutcome::AlreadyReleased
        };
        assert_eq!(
            source.release_restore(&journal, &provider).map_err(error)?,
            expected
        );
        assert!(!source.has_restore_references().map_err(error)?);
        Ok(())
    }
}

struct Terminal<'a> {
    driver: &'a Driver,
    restore: &'a BackupLayoutGuard,
    journal: &'a JournalLock,
    source: &'a BackupLayoutGuard,
    accepted: Digest,
}
impl RestoreTerminalEvidenceProvider for Terminal<'_> {
    type Error = io::Error;
    fn qualify_terminal(&self, path: &Path, bytes: &[u8]) -> io::Result<TerminalRestoreEvidence> {
        let workflow: ExecutionWorkflowRecord = serde_json::from_slice(bytes)?;
        if path != self.journal.journal()
            || workflow != self.driver.restore
            || Digest::from_bytes(&read(&self.driver.root.join("acceptance.json"))?)
                != self.accepted
        {
            return Err(io::ErrorKind::InvalidData.into());
        }
        read_complete_execution_workflow_locked(self.restore, self.journal, &workflow.digest())
            .map_err(error)?;
        let original = self.driver.metadata_evidence()?;
        let metadata = original.decode()?;
        let plan = self.driver.source_plan(self.source)?;
        let download =
            DownloadJournalGuard::open(self.source, plan.digest().hash()).map_err(error)?;
        download
            .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
            .map_err(error)?;
        download
            .read_download_manifest(&plan, &download.record().map_err(error)?.digest())
            .map_err(error)?;
        // This consumer dispatches only in-process Agent requests, never children.
        // Its inspected isolated application has no external obligations or fence.
        Ok(TerminalRestoreEvidence::new(workflow.digest(), vec![]))
    }
}
