//! Actual capture-only application fencing through public guarded workflow steps.
//! The private fixture has one target, exclusive operator custody, no timers,
//! membership mutation, release endpoint or external work. These conditions are
//! qualified here, never inferred for an application or installed as a default.

use super::{Fixture, support, support::retain_bytes as retain};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{
            AttemptJournalRecord, ObservationOutcomeRecord, ObservationReceiptRequest,
        },
        consistency::{
            ApplicationFenceBinding, ApplicationFenceEvidence, ApplicationFenceState,
            ConsistencyGuaranteeRecord, ConsistencyRequirementRecord,
        },
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        fence_acquisition::{FenceAcquisitionPayload, FenceAcquisitionRequest},
        fence_obligation::FenceObligationRecord,
        fence_reconciliation::{
            FenceReconciliationEvidence, FenceReconciliationObservation,
            FenceReconciliationObservationInput, FenceReconciliationRequest,
        },
        inventory::InventoryRecord,
        operation_plan::PlanContextRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, ExecutionStageGuard, FenceObligationRequirement,
        create_consistency_requirement, create_execution_workflow, create_fence_obligation,
    },
    ports::{
        fence_acquisition::FenceAcquisitionProviderError,
        fence_reconciliation::{FenceReconciliationProvider, FenceReconciliationProviderError},
    },
    workflow::{
        fence_acquisition::{FenceAcquisitionExecutionError, acquire_fence},
        fence_reconciliation::{observe_fence_acquisition, settle_fence_acquisition_observation},
    },
};
use ic_backup_agent::{AgentFenceAcquisitionProvider, PreparedUpdate};
use serde_json::json;
use std::{
    fs,
    future::{Future, poll_fn},
    io,
    path::{Path, PathBuf},
    task::Poll,
};

#[derive(Clone, Copy, PartialEq)]
enum Reply {
    Returned,
    Lost,
    RetainedLate,
    CancelledQualification,
    ChangedEvidence,
}

#[tokio::test(flavor = "multi_thread")]
async fn discarded_acquisition_reply_is_observed_and_explicitly_settled_without_reacquisition() {
    run(Reply::Returned).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn discarded_observation_reply_stays_pending_and_cannot_become_uncertain_or_repeat() {
    run(Reply::Lost).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn retained_late_observation_settles_original_attempt_without_another_read() {
    run(Reply::RetainedLate).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn cancelled_qualification_reopens_pending_and_settles_without_another_call() {
    run(Reply::CancelledQualification).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn changed_retained_reply_refuses_settlement_without_spending_or_reading_again() {
    run(Reply::ChangedEvidence).await;
}

#[expect(
    clippy::too_many_lines,
    reason = "keep actual effect, discarded replies, settlement and reopen in one chronological trace"
)]
async fn run(reply: Reply) {
    let mut fixture = Fixture::with_wasm(support::identity(), include_str!("state.wat")).await;
    let fence = ApplicationFenceBinding {
        identity: ArtifactChecksumRecord::from_bytes(b"capture-only fixture fence"),
        membership_revision: ArtifactChecksumRecord::from_bytes(b"one immutable fixture member"),
    };
    // A unique retained acquisition nonce supplies original-request attribution,
    // in addition to the fence/revision. The fixture admits one acquire forever.
    let nonce = ArtifactChecksumRecord::from_bytes(fixture.root.to_string_lossy().as_bytes());
    let original = [
        fence.identity.hash(),
        fence.membership_revision.hash(),
        nonce.hash(),
    ]
    .concat();
    let payload =
        FenceAcquisitionPayload::new(&fixture.target, "acquire", original.as_bytes()).unwrap();
    let (plan, layout, _workflow_root) = fixture.original(&payload.digest());
    let sequence = fixture.sequence;
    let authority = plan.attempt_authority(sequence).unwrap();
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    create_execution_workflow(&layout, &workflow).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan.clone()).unwrap();
    let root = stage.layout().unwrap().root().to_path_buf();
    let requirement = ConsistencyRequirementRecord::new(
        &plan,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    );
    let retained = FenceObligationRequirement::Capture {
        requirement: &requirement,
        fence: &fence,
    };
    create_consistency_requirement(stage.layout().unwrap(), &plan, &requirement).unwrap();
    let obligation =
        FenceObligationRecord::for_capture(&plan, &requirement, sequence, &fence).unwrap();
    create_fence_obligation(stage.layout().unwrap(), &plan, retained, &obligation).unwrap();
    let obligation_bytes = fs::read(root.join("fence-obligation.json")).unwrap();
    let transport = fixture.transport(&plan);
    let mut acquisition = AgentFenceAcquisitionProvider::new(
        &transport,
        |request: &FenceAcquisitionRequest<'_>, prepared: &PreparedUpdate<'_>| {
            assert_eq!(request.authority(), &authority);
            assert!(AttemptJournalGuard::open(stage.layout().unwrap(), &authority).is_err());
            support::retain_signed(&root, prepared)
                .map_err(|_| FenceAcquisitionProviderError::Unavailable)
        },
        |_request: &FenceAcquisitionRequest<'_>,
         _id: &ic_platform::agent::RequestId,
         bytes: &[u8]| {
            assert_eq!(bytes, b"");
            super::retain_reply(&root, bytes).unwrap();
            // Discard the public acknowledgement after an actual certified empty reply.
            // It must remain pending even though this test retains raw oracle evidence.
            Err(FenceAcquisitionProviderError::Unavailable)
        },
    );
    assert!(matches!(
        acquire_fence(
            &stage,
            retained,
            &obligation,
            &payload,
            &mut acquisition,
            async |_| { Ok::<(), io::Error>(()) }
        )
        .await,
        Err(FenceAcquisitionExecutionError::Provider(
            FenceAcquisitionProviderError::Indeterminate
        ))
    ));
    let challenge =
        ArtifactChecksumRecord::from_bytes(b"one original capture observation challenge");
    // Setup and these independent fixture oracles are outside backup dispatch.
    // Rejected application writes confirm the fixture's actual active write gate.
    let rejected = fixture
        .pic
        .update_call(
            fixture.principal(),
            fixture.caller(),
            "write",
            99_u64.to_le_bytes().to_vec(),
        )
        .await
        .unwrap_err();
    assert_eq!(
        rejected.reject_code,
        ic_platform::pocket_ic::RejectCode::CanisterReject
    );
    assert_eq!(rejected.reject_message, "fenced");
    retain(
        &root.join("rejected-write.json"),
        &serde_json::to_vec(&rejected).unwrap(),
    )
    .unwrap();
    let fenced_bytes = fixture
        .pic
        .query_call(fixture.principal(), fixture.caller(), "fence_state", vec![])
        .await
        .unwrap();
    assert_eq!(
        fenced_bytes,
        [
            original.as_bytes(),
            &1_u64.to_le_bytes(),
            &42_u64.to_le_bytes()
        ]
        .concat()
    );
    retain(&root.join("actual-fence-state.bin"), &fenced_bytes).unwrap();
    let observation_root = root.join("observation");
    private_dir(&observation_root);
    // Actual identities derive from the controlled instance/signing configuration,
    // independently of passive request declarations.
    let context: PlanContextRecord = serde_json::from_value(json!({
        "network":ArtifactChecksumRecord::from_bytes(&fixture.key).hash(),
        "caller":fixture.caller().to_text(), "release":fixture.release.hash(),
    }))
    .unwrap();
    let inventory: InventoryRecord = serde_json::from_value(json!({
        "version":1,"targets":[{"canister_id":fixture.target,"parent_canister_id":null,"role":null,"module_hash":null}]
    })).unwrap();
    let mut provider = Provider {
        fixture: &fixture,
        root: observation_root,
        original: fenced_bytes,
        fence: fence.clone(),
        context,
        inventory,
        reply,
        calls: 0,
        late: None,
    };
    let result = observe_fence_acquisition(
        &stage,
        retained,
        &obligation,
        &payload,
        challenge.clone(),
        &mut provider,
        async |request| {
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.intent().authority())
                    .is_err()
            );
            // Exclusive controlled fixture custody and its known public read method
            // qualify this one original dispatch; no remote preflight is hidden here.
            Ok::<(), io::Error>(())
        },
    )
    .await;
    let guard = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    let pending_bytes = fs::read(guard.path()).unwrap();
    let view = guard.record().unwrap().view();
    assert_eq!((view.mutations_used, view.observations_used), (1, 1));
    assert_eq!(
        (view.pending_mutation, view.pending_observation),
        (Some(1), Some(2))
    );
    drop(guard);
    assert_eq!(provider.calls, 1);
    assert!(
        observe_fence_acquisition(
            &stage,
            retained,
            &obligation,
            &payload,
            ArtifactChecksumRecord::from_bytes(b"cannot reset with a new challenge"),
            &mut provider,
            async |_| -> Result<(), io::Error> { panic!("pending observation reentered") }
        )
        .await
        .is_err()
    );
    assert!(
        acquire_fence(
            &stage,
            retained,
            &obligation,
            &payload,
            &mut acquisition,
            async |_| -> Result<(), io::Error> { panic!("pending acquisition reentered") }
        )
        .await
        .is_err()
    );
    let observation = match reply {
        Reply::Returned | Reply::CancelledQualification | Reply::ChangedEvidence => {
            Some(result.unwrap())
        }
        Reply::Lost => {
            assert!(result.is_err());
            assert!(provider.late.is_none());
            None
        }
        Reply::RetainedLate => {
            assert!(result.is_err());
            provider.late.take()
        }
    };
    // Reopen the exact retained stage before settlement; no provider read is replayed.
    drop(stage);
    let stage = ExecutionStageGuard::open(&layout, &workflow.digest(), sequence, &binding.digest())
        .unwrap();
    if let Some(observation) = observation {
        if reply == Reply::CancelledQualification {
            let mut entered = false;
            {
                let mut settlement = Box::pin(settle_fence_acquisition_observation(
                    &stage,
                    retained,
                    &obligation,
                    &payload,
                    challenge.clone(),
                    &observation,
                    async |request, actual| -> io::Result<ObservationReceiptRequest> {
                        qualify_retained(request, actual, &provider, &challenge)?;
                        entered = true;
                        std::future::pending::<()>().await;
                        unreachable!("cancelled qualification completed")
                    },
                ));
                assert!(poll_fn(|cx| Poll::Ready(settlement.as_mut().poll(cx).is_pending())).await);
                assert!(AttemptJournalGuard::open(stage.layout().unwrap(), &authority).is_err());
                // Drop the incomplete workflow future while independent qualification
                // owns exclusion. Retained raw evidence is not an automatic receipt.
            }
            assert!(entered);
            let journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
            assert_eq!(fs::read(journal.path()).unwrap(), pending_bytes);
            assert_eq!(
                journal.record().unwrap().view().pending_observation,
                Some(2)
            );
        }
        if reply == Reply::ChangedEvidence {
            let path = provider.root.join("reply.candid");
            let original_path = provider.root.join("original-reply.candid");
            let original_raw = fs::read(&path).unwrap();
            fs::rename(&path, &original_path).unwrap();
            let mut changed = original_raw.clone();
            changed[0] ^= 1;
            retain(&path, &changed).unwrap();
            let rejected = settle_fence_acquisition_observation(
                &stage,
                retained,
                &obligation,
                &payload,
                challenge.clone(),
                &observation,
                async |request, actual| qualify_retained(request, actual, &provider, &challenge),
            )
            .await;
            assert!(matches!(
                rejected,
                Err(ic_backup::workflow::fence_reconciliation::FenceReconciliationSettlementError::Qualification(_))
            ));
            assert_eq!(
                fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap(),
                pending_bytes
            );
            // Explicit fixture-only correction keeps both originals and changed bytes.
            // Production settlement never rewrites, repairs or discards evidence.
            fs::rename(&path, provider.root.join("changed-reply.candid")).unwrap();
            fs::rename(&original_path, &path).unwrap();
            fs::File::open(&provider.root).unwrap().sync_all().unwrap();
            assert_eq!(fs::read(&path).unwrap(), original_raw);
        }
        assert!(
            settle_fence_acquisition_observation(
                &stage,
                retained,
                &obligation,
                &payload,
                challenge.clone(),
                &observation,
                async |_, _| -> Result<ObservationReceiptRequest, io::Error> {
                    Err(io::Error::other("independent qualification withheld"))
                }
            )
            .await
            .is_err()
        );
        assert_eq!(
            fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap(),
            pending_bytes
        );
        let outcome = settle_fence_acquisition_observation(
            &stage,
            retained,
            &obligation,
            &payload,
            challenge.clone(),
            &observation,
            async |request, actual| {
                assert!(AttemptJournalGuard::open(stage.layout().unwrap(), &authority).is_err());
                tokio::task::yield_now().await;
                assert!(AttemptJournalGuard::open(stage.layout().unwrap(), &authority).is_err());
                qualify_retained(request, actual, &provider, &challenge)
            },
        )
        .await
        .unwrap();
        assert_eq!(outcome, ObservationOutcomeRecord::Applied);
        assert!(
            settle_fence_acquisition_observation(
                &stage,
                retained,
                &obligation,
                &payload,
                challenge,
                &observation,
                async |_, _| -> Result<ObservationReceiptRequest, io::Error> {
                    panic!("settled observation reentered")
                }
            )
            .await
            .is_err()
        );
    }
    let guard = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    let view = guard.record().unwrap().view();
    assert_eq!((view.mutations_used, view.observations_used), (1, 1));
    assert_eq!(
        (view.mutations_remaining, view.observations_remaining),
        (0, 0)
    );
    if reply == Reply::Lost {
        assert_eq!(
            (view.pending_mutation, view.pending_observation),
            (Some(1), Some(2))
        );
        assert!(!view.applied);
        assert_eq!(fs::read(guard.path()).unwrap(), pending_bytes);
    } else {
        assert_eq!(
            (view.pending_mutation, view.pending_observation),
            (None, None)
        );
        assert!(view.applied);
    }
    assert_eq!(provider.calls, 1);
    assert_eq!(
        fs::read(root.join("fence-obligation.json")).unwrap(),
        obligation_bytes
    );
    assert!(!root.join("execution-settlement.json").exists());
    // Actual active state remains unchanged: settlement performs no release/update/read.
    assert_eq!(
        fixture
            .pic
            .query_call(fixture.principal(), fixture.caller(), "fence_state", vec![])
            .await
            .unwrap(),
        provider.original
    );
}

struct Provider<'a> {
    fixture: &'a Fixture,
    root: PathBuf,
    original: Vec<u8>,
    fence: ApplicationFenceBinding,
    context: PlanContextRecord,
    inventory: InventoryRecord,
    reply: Reply,
    calls: u32,
    late: Option<FenceReconciliationObservation>,
}
impl FenceReconciliationProvider for Provider<'_> {
    async fn observe_fence_acquisition(
        &mut self,
        request: &FenceReconciliationRequest<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> Result<FenceReconciliationObservation, FenceReconciliationProviderError> {
        request.validate_journal(journal).unwrap();
        assert_eq!(request.intent().plan().context(), &self.context);
        assert_eq!(request.intent().plan().inventory(), &self.inventory);
        assert_eq!(
            request.intent().plan().selected_targets(),
            std::slice::from_ref(&self.fixture.target)
        );
        assert_eq!(self.calls, 0);
        let challenge = request.intent().challenge().hash().as_bytes();
        retain(
            &self.root.join("request.json"),
            &serde_json::to_vec(&request_evidence(request, self)).unwrap(),
        )
        .unwrap();
        self.calls += 1;
        let raw = self
            .fixture
            .pic
            .update_call(
                self.fixture.principal(),
                self.fixture.caller(),
                "observe",
                challenge.to_vec(),
            )
            .await
            .unwrap();
        assert_eq!(raw, [self.original.as_slice(), challenge].concat());
        super::retain_reply(&self.root, &raw).unwrap();
        let proof = evidence(request, &raw);
        let actual = FenceReconciliationObservation::new(FenceReconciliationObservationInput {
            request: request.digest(),
            mutation_attempt: request.intent().mutation_attempt(),
            observation_attempt: request.observation_attempt(),
            context: self.context.clone(),
            inventory: self.inventory.clone(),
            selected_targets: vec![self.fixture.target.clone()],
            settlement: FenceReconciliationEvidence::AcquiredCapture {
                fence: Box::new(ApplicationFenceEvidence {
                    state: ApplicationFenceState::Active,
                    identity: self.fence.identity.clone(),
                    membership_revision: self.fence.membership_revision.clone(),
                    writes: proof.clone(),
                    membership: proof.clone(),
                    timers: proof.clone(),
                    external_work: proof.clone(),
                    drained_work: proof.clone(),
                }),
                attribution: ArtifactChecksumRecord::from_bytes(&self.original),
            },
            evidence: proof,
            remote_observations: 1,
        })
        .unwrap();
        match self.reply {
            Reply::Returned | Reply::CancelledQualification | Reply::ChangedEvidence => Ok(actual),
            Reply::Lost => Err(FenceReconciliationProviderError::Indeterminate),
            Reply::RetainedLate => {
                self.late = Some(actual);
                Err(FenceReconciliationProviderError::Indeterminate)
            }
        }
    }
}
fn qualify_retained(
    request: &FenceReconciliationRequest<'_, '_>,
    actual: &FenceReconciliationObservation,
    provider: &Provider<'_>,
    challenge: &ArtifactChecksumRecord,
) -> io::Result<ObservationReceiptRequest> {
    let retained_request: serde_json::Value =
        serde_json::from_slice(&fs::read(provider.root.join("request.json"))?)?;
    if retained_request != request_evidence(request, provider) {
        return Err(io::Error::other(
            "original durable observation request differs",
        ));
    }
    let raw = fs::read(provider.root.join("reply.candid"))?;
    let expected = [provider.original.as_slice(), challenge.hash().as_bytes()].concat();
    if raw != expected || actual.input().evidence != evidence(request, &raw) {
        return Err(io::Error::other("original durable observation differs"));
    }
    // This immutable one-member no-external-effects fixture independently
    // qualifies exact original nonce, single acquisition and fenced writes.
    let input = actual.input();
    Ok(ObservationReceiptRequest {
        attempt: input.observation_attempt,
        request: input.request.hash().into(),
        outcome: ObservationOutcomeRecord::Applied,
        evidence: input.evidence.hash().into(),
    })
}
fn request_evidence(
    request: &FenceReconciliationRequest<'_, '_>,
    provider: &Provider<'_>,
) -> serde_json::Value {
    json!({"request":request.digest().hash(), "mutation":request.intent().mutation_attempt(),
        "observation":request.observation_attempt(),"context":provider.context,"inventory":provider.inventory,
        "method":"observe","target":provider.fixture.target,"arguments":request.intent().challenge().hash().as_bytes()})
}
fn evidence(request: &FenceReconciliationRequest<'_, '_>, raw: &[u8]) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_bytes(
        &[
            request.digest().hash().as_bytes(),
            &request.intent().mutation_attempt().to_be_bytes(),
            &request.observation_attempt().to_be_bytes(),
            raw,
        ]
        .concat(),
    )
}
fn private_dir(path: &Path) {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).unwrap();
}
