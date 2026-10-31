//! Normal-dependency opaque application ingress; fixture authority is supplied by the parent.
use super::evidence::ObservedContext;
use super::{ApplicationConfig, AttemptBudgetRecord, command::Mode};
use super::{error, read, retain};
use ic_backup::{
    model::{
        application_ingress::{
            ApplicationIngressPayload, ApplicationMutationRequest, ApplicationObservationRequest,
        },
        artifacts::ArtifactChecksumRecord as Digest,
        attempt_journal::{
            MutationOutcomeRecord, MutationReceiptRequest, ObservationOutcomeRecord,
            ObservationReceiptRequest,
        },
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, ExecutionStageGuard},
    workflow::execution_workflow::{
        ExecutionWorkflowStageDriver, RetainedStageEvidence, run_new_execution_workflow,
        run_retained_execution_workflow,
    },
};
use ic_backup_agent::{
    AgentApplicationIngressProvider, ApplicationIngressProviderError as ProviderError,
    ApplicationIngressResponse, PreparedUpdate, ReservedUpdate,
};
use ic_platform::agent::RequestId;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io, path::Path};

#[derive(Deserialize)]
struct ApplicationAdmission {
    context: ObservedContext,
    target: String,
}

fn payload(config: &ApplicationConfig, method: &str) -> ApplicationIngressPayload {
    let original = 73_u64.to_le_bytes();
    ApplicationIngressPayload::new(
        &config.connection.target,
        &config.connection.target,
        method,
        if method == "apply" { &original } else { &[] },
        if method == "apply" {
            config.reply_limit
        } else {
            16
        },
    )
    .unwrap()
}
fn declaration(payload: &ApplicationIngressPayload) -> Value {
    json!({"receiver":payload.receiver(),"target":payload.target(),"mode":"update","method":payload.method(),"arguments":payload.arguments(),"reply_limit":payload.reply_limit()})
}
fn plan(config: &ApplicationConfig, payload: &ApplicationIngressPayload) -> OperationPlanRecord {
    config.connection.singleton(
        0,
        &payload.digest(),
        AttemptBudgetRecord::new(1, 1).unwrap(),
    )
}

fn lane(request: &ReservedUpdate<'_>) -> &'static str {
    match request {
        ReservedUpdate::ApplicationMutation(_) => "mutation",
        ReservedUpdate::ApplicationObservation(_) => "observation",
        _ => panic!("application request only"),
    }
}
fn ingress(
    root: &Path,
    request: &ReservedUpdate<'_>,
    prepared: &PreparedUpdate<'_>,
) -> Result<(), ProviderError> {
    let lane = lane(request);
    // Original lane/attempt names are create-only; changed expiry/request IDs never
    // allow another dispatch under a reconstructed pending request.
    retain(
        &root.join(format!("{lane}-envelope.cbor")),
        prepared.envelope(),
    )
    .map_err(|_| ProviderError::Unavailable)?;
    retain(
        &root.join(format!("{lane}-request.txt")),
        prepared.request_id().to_string().as_bytes(),
    )
    .map_err(|_| ProviderError::Unavailable)
}
fn reply(
    root: &Path,
    mode: Mode,
    request: &ReservedUpdate<'_>,
    id: &RequestId,
    bytes: &[u8],
) -> Result<(), ProviderError> {
    if mode == Mode::ApplicationRefuseReply {
        return Err(ProviderError::Unavailable);
    }
    retain(&root.join(format!("{}-reply.bin", lane(request))), bytes)
        .map_err(|_| ProviderError::Unavailable)?;
    retain(
        &root.join(format!("{}-reply-id.txt", lane(request))),
        id.to_string().as_bytes(),
    )
    .map_err(|_| ProviderError::Unavailable)?;
    if mode == Mode::ApplicationDie {
        std::process::exit(75);
    }
    Ok(())
}
fn qualify(response: &ApplicationIngressResponse) {
    // Only this isolated application has exclusive ingress custody and a monotonic
    // count; no generic application or Root-paid attribution is inferred.
    assert_eq!(
        response.reply(),
        [1_u64.to_le_bytes(), 73_u64.to_le_bytes()].concat()
    );
    assert!(!format!("{response:?}").contains("opaque"));
}
struct Driver<'a> {
    root: &'a Path,
    config: &'a ApplicationConfig,
    plan: &'a OperationPlanRecord,
    payload: &'a ApplicationIngressPayload,
    mode: Mode,
    callbacks: usize,
}
impl ExecutionWorkflowStageDriver for Driver<'_> {
    type Error = io::Error;
    fn bind_stage(
        &mut self,
        _: &ExecutionWorkflowRecord,
        _: u64,
        _: &[ic_backup::model::execution_workflow::ExecutionStagePredecessorRecord],
    ) -> io::Result<OperationPlanRecord> {
        self.callbacks += 1;
        Ok(self.plan.clone())
    }
    async fn execute_stage(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<Digest> {
        self.callbacks += 1;
        let admission: ApplicationAdmission =
            serde_json::from_slice(&read(&self.root.join("admission.json"))?)?;
        assert!(admission.context.matches(&self.config.connection.context));
        assert_eq!(admission.target, self.config.connection.target);
        let authority = self.plan.attempt_authority(0).map_err(error)?;
        self.payload
            .validate_mutation_binding(authority.binding())
            .map_err(error)?;
        let mut journal =
            AttemptJournalGuard::open(stage.layout().map_err(error)?, &authority).map_err(error)?;
        journal
            .reserve_planned_mutation(&self.plan.digest())
            .map_err(error)?;
        let request = ApplicationMutationRequest::new(
            self.plan,
            0,
            journal.record().map_err(error)?,
            self.payload,
        )
        .map_err(error)?;
        let transport = self.config.connection.transport(self.plan);
        let root = stage.layout().map_err(error)?.root();
        let mut provider = AgentApplicationIngressProvider::new(
            &transport,
            |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                ingress(root, request, prepared)
            },
            |request: &ReservedUpdate<'_>, id: &RequestId, bytes: &[u8]| {
                reply(root, self.mode, request, id, bytes)
            },
        );
        let response = provider
            .submit_mutation(&request, journal.record().map_err(error)?)
            .await
            .map_err(error)?;
        assert_eq!(
            read(&root.join("mutation-request.txt"))?,
            response.request_id().to_string().as_bytes()
        );
        assert_eq!(
            read(&root.join("mutation-reply-id.txt"))?,
            response.request_id().to_string().as_bytes()
        );
        assert_eq!(read(&root.join("mutation-reply.bin"))?, response.reply());
        qualify(&response);
        assert_eq!(response.request(), &self.payload.digest());
        journal
            .record_mutation(MutationReceiptRequest {
                attempt: request.mutation_attempt(),
                request: self.payload.digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: response.evidence().hash().into(),
            })
            .map_err(error)?;
        Ok(Digest::from_bytes(response.reply()))
    }
}

pub(super) async fn run(root: &Path, mode: Mode) -> io::Result<()> {
    let config: ApplicationConfig = serde_json::from_slice(&read(&root.join("config.json"))?)?;
    let payload = payload(&config, "apply");
    let plan = plan(&config, &payload);
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    let layout = BackupLayoutGuard::acquire(root).map_err(error)?;
    let original = declaration(&payload);
    if mode.starts_application() {
        retain(
            &root.join("original-payload.json"),
            &serde_json::to_vec(&original)?,
        )?;
    } else {
        let retained: Value = serde_json::from_slice(&read(&root.join("original-payload.json"))?)?;
        if retained != original {
            return Err(io::ErrorKind::InvalidData.into());
        }
    }

    if mode == Mode::ApplicationObserve {
        return observe(&layout, &config, &plan, &workflow).await;
    }
    let mut driver = Driver {
        root,
        config: &config,
        plan: &plan,
        payload: &payload,
        mode,
        callbacks: 0,
    };
    if mode.starts_application() {
        run_new_execution_workflow(&layout, &workflow, &mut driver)
            .await
            .map_err(error)?;
        assert_eq!(driver.callbacks, 2);
    } else if matches!(mode, Mode::ApplicationReplay | Mode::ApplicationResume) {
        let binding: ExecutionStageBindingRecord =
            serde_json::from_slice(&read(&root.join("execution-stage-0/stage-binding.json"))?)?;
        let learned =
            Digest::from_bytes(&read(&root.join("execution-stage-0/mutation-reply.bin"))?);
        let retained = BTreeMap::from([(0, RetainedStageEvidence::new(binding.digest(), learned))]);
        let result =
            run_retained_execution_workflow(&layout, &workflow.digest(), &retained, &mut driver)
                .await;
        assert_eq!(
            driver.callbacks, 0,
            "settled replay/pending refusal never reaches signing or submission"
        );
        result.map_err(error)?;
    } else {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    Ok(())
}

async fn observe(
    layout: &BackupLayoutGuard,
    config: &ApplicationConfig,
    plan: &OperationPlanRecord,
    workflow: &ExecutionWorkflowRecord,
) -> io::Result<()> {
    let binding: ExecutionStageBindingRecord = serde_json::from_slice(&read(
        &layout.root().join("execution-stage-0/stage-binding.json"),
    )?)?;
    let stage = ExecutionStageGuard::open(layout, &workflow.digest(), 0, &binding.digest())
        .map_err(error)?;
    let root = stage.layout().map_err(error)?.root();
    let authority = plan.attempt_authority(0).map_err(error)?;
    let mut journal =
        AttemptJournalGuard::open(stage.layout().map_err(error)?, &authority).map_err(error)?;
    let payload = payload(config, "observe");
    payload
        .validate_observation_binding(authority.binding())
        .map_err(error)?;
    retain(
        &root.join("observation-payload.json"),
        &serde_json::to_vec(&declaration(&payload))?,
    )?;
    let attempt = journal
        .reserve_planned_observation(&plan.digest(), 1, payload.digest().hash())
        .map_err(error)?;
    let request =
        ApplicationObservationRequest::new(plan, 0, journal.record().map_err(error)?, &payload)
            .map_err(error)?;
    let transport = config.connection.transport(plan);
    let mut provider = AgentApplicationIngressProvider::new(
        &transport,
        |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            ingress(root, request, prepared)
        },
        |request: &ReservedUpdate<'_>, id: &RequestId, bytes: &[u8]| {
            reply(root, Mode::ApplicationObserve, request, id, bytes)
        },
    );
    let response = provider
        .submit_observation(&request, journal.record().map_err(error)?)
        .await
        .map_err(error)?;
    qualify(&response);
    assert_eq!(read(&root.join("mutation-reply.bin"))?, response.reply());
    journal
        .record_planned_observation(
            &plan.digest(),
            ObservationReceiptRequest {
                attempt,
                request: payload.digest().hash().into(),
                outcome: ObservationOutcomeRecord::Applied,
                evidence: response.evidence().hash().into(),
            },
        )
        .map_err(error)?;
    drop(journal);
    stage
        .reconcile_checkpoint(Digest::from_bytes(response.reply()))
        .map_err(error)?;
    Ok(())
}
