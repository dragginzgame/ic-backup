//! Isolated application authority and fault injection; never part of the supported assembly.

use super::command::Fault;
use super::evidence::{AcceptanceRecord, AdmissionRecord};
use super::{Config, Mode, Stage, error, read, retain, standalone};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord as Digest,
        attempt_journal::MutationReceiptRequest,
        execution_workflow::ExecutionWorkflowRecord,
        ic_mutation::IcMutationRequest,
        ic_snapshot_metadata::IcSnapshotMetadataReply,
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreSafetyEvidence, RestoreSafetyLaneRecord, RestoreSafetyObservation,
            RestoreSafetyObservationInput, RestoreSafetyRequest, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest, TargetRestoreEvidence,
        },
    },
    view::restore_reference_release::TerminalRestoreEvidence,
};
use ic_backup_agent::{PreparedUpdate, ReservedUpdate};
use ic_platform::management_canister_types::CanisterStatusType;
use std::{
    io,
    path::{Path, PathBuf},
};

pub(super) struct Authority {
    pub root: PathBuf,
    pub config: Config,
    pub mode: Mode,
    pub fault: Fault,
}

impl standalone::Application for Authority {
    fn admit(
        &self,
        _stage: Stage,
        plan: &OperationPlanRecord,
    ) -> impl std::future::Future<Output = io::Result<Digest>> {
        std::future::ready((|| {
            let raw = read(&self.root.join(format!("admission-{}.json", self.mode)))?;
            let evidence: AdmissionRecord = serde_json::from_slice(&raw)?;
            let caller = self.config.context.caller();
            if plan.context() != &self.config.context
                || !evidence.context.matches(&self.config.context)
                || evidence.target != self.config.target
                || evidence.controllers != [caller]
                || evidence.module_hash != self.config.context.release()
                || evidence.status != CanisterStatusType::Stopped
            {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            // The parent owns one stopped canister/controller and separately accounts
            // fresh observations before this exclusive phase. No timers/outbound work.
            Ok(Digest::from_bytes(&raw))
        })())
    }

    fn retain_ingress(
        &self,
        stage: Stage,
        root: &Path,
        request: &ReservedUpdate<'_>,
        prepared: &PreparedUpdate<'_>,
    ) -> io::Result<()> {
        if stage == Stage::Capture && self.fault == Fault::CaptureIngress {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        standalone::retain_ingress(root, request, prepared)
    }

    fn qualify_reply(
        &self,
        reply: standalone::Reply<'_>,
    ) -> impl std::future::Future<Output = io::Result<MutationReceiptRequest>> {
        std::future::ready((|| {
            if self.fault == Fault::LostReply(reply.stage, reply.sequence) {
                retain(&reply.root.join("discarded-oracle.candid"), reply.bytes)?;
                std::process::exit(75);
            }
            if reply.stage == Stage::Capture && self.fault == Fault::CaptureRetention {
                return Err(io::Error::other("fixture durable reply retention refused"));
            }
            assert_eq!(reply.plan.context(), &self.config.context);
            // Authenticated Agent reply plus exclusive parent/application custody
            // attributes the one original ingress for this inspected WAT program only.
            let receipt =
                standalone::retain_applied_reply(&reply, &Digest::from_bytes(reply.bytes))?;
            if reply.stage == Stage::Download
                && reply.sequence == 1
                && self.fault == Fault::CapturePartial
            {
                return Err(io::Error::other(
                    "fixture interrupted transfer after first admitted chunk",
                ));
            }
            Ok(receipt)
        })())
    }

    fn restore_requirement(
        &self,
        restore: &OperationPlanRecord,
        source: &OperationPlanRecord,
        manifest: &Digest,
    ) -> io::Result<RestoreSafetyRequirementRecord> {
        // Only the inspected exclusively owned fixture has no irreversible effects.
        RestoreSafetyRequirementRecord::new(
            restore,
            source,
            RestoreSafetyRequirementRequest {
                source_artifacts: manifest.clone(),
                safety: RestoreSafetyLaneRecord::NoIrreversibleEffects,
                expected_fence: None,
            },
        )
        .map_err(error)
    }

    async fn restore_safety(
        &self,
        original: &IcMutationRequest<'_>,
        request: &RestoreSafetyRequest<'_>,
        acceptance: Option<&Digest>,
    ) -> io::Result<RestoreSafetyObservation> {
        let stage = Stage::from_sequence(request.binding().operation_sequence())?;
        let evidence = self.admit(stage, original.plan()).await?;
        RestoreSafetyObservation::new(RestoreSafetyObservationInput {
            request: request.digest(),
            context: original.plan().context().clone(),
            inventory: original.plan().inventory().clone(),
            source_plan_intent: Digest::from_hash(request.requirement().source_plan_intent())
                .map_err(error)?,
            source_artifacts: request.requirement().source_artifacts().clone(),
            targets: vec![TargetRestoreEvidence {
                target: request.wire().target().into(),
                state: CanisterStatusType::Stopped,
                lifecycle_evidence: evidence.clone(),
                restored_acceptance: acceptance.cloned(),
            }],
            safety: RestoreSafetyEvidence::NoIrreversibleEffects(
                Digest::from_hash(self.config.context.release()).map_err(error)?,
            ),
            evidence,
            remote_observations: 0,
        })
        .map_err(error)
    }

    fn acceptance(
        &self,
        restore: &ExecutionWorkflowRecord,
        metadata: &IcSnapshotMetadataReply<'_>,
        destination: &[u8],
    ) -> io::Result<Digest> {
        let raw = read(&self.root.join("acceptance.json"))?;
        let record: AcceptanceRecord = serde_json::from_slice(&raw)?;
        if !record.restore.matches(&restore.digest())
            || !record.source_metadata.matches(&metadata.digest())
            || record.destination != destination
            || !record.complete_state
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(Digest::from_bytes(&raw))
    }

    fn terminal(
        &self,
        restore: &ExecutionWorkflowRecord,
        _acceptance: &Digest,
    ) -> io::Result<TerminalRestoreEvidence> {
        // This fixture uses in-process Agent calls exclusively, with no children,
        // external obligations, timers or fence. Its complete command set is empty.
        Ok(TerminalRestoreEvidence::for_in_process_restore(
            restore.digest(),
        ))
    }
}
