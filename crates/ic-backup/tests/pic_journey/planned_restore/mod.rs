//! Actual same-ID load/start after complete upload, with no follow-up after lost replies.

use super::{
    Backend, Downloaded, Fault, lifecycle, management, planned_upload::Failure, status,
    verify_loaded,
};
use super::{planned_upload::ReplyFault, stage::Stage};
use crate::ready::ready;
use ic_backup::model::ic_update_response::IcUpdateResponseInput;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest},
        ic_request::{IcManagementMethodRecord as Method, IcManagementRequestRecord},
        ic_snapshot_metadata::IcSnapshotMetadataReply,
        operation_plan::{OperationPlanRecord, PlanBudgetRecord},
        restore_safety::{
            RestoreSafetyEvidence, RestoreSafetyLaneRecord, RestoreSafetyObservation,
            RestoreSafetyObservationInput, RestoreSafetyRequest, RestoreSafetyRequestInput,
            RestoreSafetyRequirementRecord, RestoreSafetyRequirementRequest, TargetRestoreEvidence,
        },
    },
    ops::persistence::{
        BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard, create_execution_workflow,
        create_restore_safety_requirement,
    },
    ports::ic_mutation::{IcMutationProvider, IcMutationProviderError},
    workflow::ic_snapshot_restore::{IcRestoreExecutionError, IcRestoreSource, restore_snapshot},
};
use ic_platform::management_canister_types::CanisterStatusType;
use std::{
    cell::RefCell,
    fs,
    io::{self, Write},
};

pub(super) struct Provider<'a, 'b> {
    pub(super) backend: &'a RefCell<&'b mut Backend>,
    pub(super) failure: Failure,
}
impl IcMutationProvider for Provider<'_, '_> {
    fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    ) -> impl std::future::Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>>
    {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            let mut backend = self.backend.borrow_mut();
            let actual = backend.plan(
                &request.payload().digest(),
                request.authority().binding().operation_sequence(),
            );
            assert_eq!(request.plan().context(), actual.context());
            assert_eq!(request.payload().target(), backend.target.to_text());
            let raw = backend.management(
                request.payload().method().name(),
                request.payload().arguments(),
            );
            let stage = match request.payload().method() {
                Method::LoadCanisterSnapshot => Stage::Load,
                Method::StartCanister => Stage::Start,
                _ => panic!("restore provider requires load or start"),
            };
            let fault = self
                .failure
                .reply_at(stage, request.authority().binding().operation_sequence());
            if fault.is_some() {
                fs::write(backend.root.join("discarded-restore-oracle.candid"), &raw).unwrap();
            }
            let reply = match fault {
                Some(ReplyFault::Lost) => return Err(IcMutationProviderError::Indeterminate),
                Some(ReplyFault::Malformed) => vec![],
                None => raw,
            };
            Ok(IcMutationAcknowledgement::new(IcUpdateResponseInput {
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                context: actual.context().clone(),
                target: backend.target.to_text(),
                evidence: ArtifactChecksumRecord::from_bytes(&reply),
                reply,
            })
            .unwrap())
        })())
    }
}
fn child(
    backend: &Backend,
    payload: &IcManagementRequestRecord,
    sequence: u64,
) -> OperationPlanRecord {
    backend.plan_with_observations(&payload.digest(), sequence, 0)
}

#[expect(
    clippy::too_many_arguments,
    reason = "fixture passes existing guarded source and complete original upload evidence explicitly"
)]
#[expect(
    clippy::too_many_lines,
    reason = "keep the complete upload/load/verification/start chronology and stopped failure proof together"
)]
pub(super) fn run(
    backend: &mut Backend,
    source_plan: &OperationPlanRecord,
    source_layout: &BackupLayoutGuard,
    source_journal: &DownloadJournalGuard<'_>,
    metadata: &IcSnapshotMetadataReply<'_>,
    destination: &[u8],
    source: &Downloaded,
    failure: Failure,
    restore_source: &IcRestoreSource<'_>,
) {
    // Independently change live state so successful load must restore the original.
    assert!(lifecycle(backend, Method::StartCanister, None, Fault::None, |_| None).is_none());
    backend.write_state(99);
    backend.assert_state(99);
    backend.clear_fixture_chunk_store();
    assert!(lifecycle(backend, Method::StopCanister, None, Fault::None, |_| None).is_none());
    let load = management(
        backend,
        Method::LoadCanisterSnapshot,
        Some(destination.to_vec()),
    );
    let start = management(backend, Method::StartCanister, None);
    let load_plan = child(backend, &load, 17);
    let start_plan = child(backend, &start, 3);
    let mut allocation = backend.plan_request(&load.digest(), 17, 0);
    allocation.graph = EffectGraphRecord::new(vec![
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 17,
            depends_on: vec![],
        })
        .unwrap(),
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 3,
            depends_on: vec![17],
        })
        .unwrap(),
    ])
    .unwrap();
    allocation
        .operations
        .push(start_plan.operations()[0].clone());
    allocation.budget = PlanBudgetRecord::new(2, 0).unwrap();
    let workflow = ExecutionWorkflowRecord::new(OperationPlanRecord::new(allocation).unwrap());
    let root = backend.root.join("original-restore-workflow");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, &workflow).unwrap();
    source_layout
        .retain_restore(
            &root.join("execution-workflow.json"),
            workflow.digest().hash(),
        )
        .unwrap();
    let source_bytes = fs::read(source_journal.path()).unwrap();
    let source_references = source_layout.restore_references().unwrap();
    let backend = RefCell::new(backend);
    let mut predecessor = None;
    let mut acceptance = None;
    for (sequence, payload, plan) in [(17, &load, load_plan), (3, &start, start_plan)] {
        status(&mut backend.borrow_mut(), CanisterStatusType::Stopped);
        let binding = ExecutionStageBindingRecord::new(
            &workflow,
            sequence,
            &plan,
            predecessor.iter().cloned().collect(),
        )
        .unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap();
        let requirement = RestoreSafetyRequirementRecord::new(
            stage.plan(),
            source_plan,
            RestoreSafetyRequirementRequest {
                source_artifacts: source.manifest.clone(),
                safety: RestoreSafetyLaneRecord::NoIrreversibleEffects,
                expected_fence: None,
            },
        )
        .unwrap();
        create_restore_safety_requirement(
            stage.layout().unwrap(),
            source_layout,
            stage.plan(),
            source_plan,
            &requirement,
        )
        .unwrap();
        let safety = RestoreSafetyRequest::new(
            stage.plan(),
            source_plan,
            &requirement,
            payload,
            RestoreSafetyRequestInput {
                operation_sequence: sequence,
                challenge: ArtifactChecksumRecord::from_bytes(
                    format!(
                        "fresh exclusive boundary {}:{sequence}",
                        workflow.digest().hash()
                    )
                    .as_bytes(),
                ),
                max_remote_observations: 0,
            },
        )
        .unwrap();
        let mut provider = Provider {
            backend: &backend,
            failure,
        };
        let result = ready(restore_snapshot(
            &stage,
            restore_source.clone(),
            &safety,
            &mut provider,
            async |request, safety| {
                assert_eq!(
                    request.payload().target(),
                    backend.borrow().target.to_text()
                );
                assert_eq!(
                    source_journal
                        .verify_ic_snapshot_artifact(source_plan, super::TOKEN, metadata)
                        .unwrap(),
                    source.checksum
                );
                // This isolated one-owner Wasm fixture has no external work or timers.
                // A separately accounted fresh stopped/controller observation precedes
                // this boundary; exclusive custody prevents intervening mutation.
                Ok::<_, io::Error>(
                    RestoreSafetyObservation::new(RestoreSafetyObservationInput {
                        request: safety.digest(),
                        context: request.plan().context().clone(),
                        inventory: request.plan().inventory().clone(),
                        source_plan_intent: source_plan.digest(),
                        source_artifacts: source.manifest.clone(),
                        targets: vec![TargetRestoreEvidence {
                            target: backend.borrow().target.to_text(),
                            state: CanisterStatusType::Stopped,
                            lifecycle_evidence: ArtifactChecksumRecord::from_bytes(
                                b"exclusive fixture fresh stopped/control history",
                            ),
                            restored_acceptance: acceptance.clone(),
                        }],
                        safety: RestoreSafetyEvidence::NoIrreversibleEffects(
                            ArtifactChecksumRecord::from_bytes(
                                b"closed fixture has no irreversible effects",
                            ),
                        ),
                        evidence: ArtifactChecksumRecord::from_bytes(
                            b"exclusive original fixture safety",
                        ),
                        remote_observations: 0,
                    })
                    .unwrap(),
                )
            },
            async |request, acknowledgement| {
                // Exact successful original simulator ingress is attributed independently
                // of its empty tuple; retain both original byte sets before Applied.
                for (name, bytes) in [
                    ("request.candid", request.payload().arguments()),
                    ("reply.candid", acknowledgement.input().reply.as_slice()),
                ] {
                    let file = fs::File::create(stage.layout().unwrap().root().join(name))?;
                    (&file).write_all(bytes)?;
                    file.sync_all()?;
                }
                fs::File::open(stage.layout().unwrap().root())?.sync_all()?;
                Ok(MutationReceiptRequest {
                    attempt: request.mutation_attempt(),
                    request: request.payload().digest().hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: acknowledgement.input().evidence.hash().into(),
                })
            },
        ));
        if failure
            .reply_at(Stage::from_sequence(sequence), sequence)
            .is_some()
        {
            match result {
                Err(IcRestoreExecutionError::Provider(IcMutationProviderError::Indeterminate)) => {}
                Err(IcRestoreExecutionError::AfterReply {
                    acknowledgement, ..
                }) => assert_eq!(acknowledgement.input().reply, [] as [u8; 0]),
                result => panic!("original restore safe stop: {result:?}"),
            }
            let calls = backend.borrow().calls;
            let journal = fs::read(
                stage
                    .layout()
                    .unwrap()
                    .root()
                    .join(format!("attempt-{sequence}.json")),
            )
            .unwrap();
            drop(safety);
            drop(stage);
            let (stage, progress) = ExecutionStageGuard::resume(
                &layout,
                binding.workflow(),
                sequence,
                &binding.digest(),
            )
            .unwrap();
            assert_eq!(progress.attempts.mutations_used, 1);
            assert_eq!(progress.applied_operations, 0);
            let safety = RestoreSafetyRequest::new(
                stage.plan(),
                source_plan,
                &requirement,
                payload,
                RestoreSafetyRequestInput {
                    operation_sequence: sequence,
                    challenge: ArtifactChecksumRecord::from_bytes(b"reopen grants no restart"),
                    max_remote_observations: 0,
                },
            )
            .unwrap();
            assert!(
                ready(restore_snapshot(
                    &stage,
                    restore_source.clone(),
                    &safety,
                    &mut provider,
                    async |_, _| -> Result<_, io::Error> { panic!("pending never admits") },
                    async |_, _| -> Result<_, io::Error> { panic!("pending never settles") }
                ))
                .is_err()
            );
            assert_eq!(backend.borrow().calls, calls);
            assert_eq!(
                fs::read(
                    stage
                        .layout()
                        .unwrap()
                        .root()
                        .join(format!("attempt-{sequence}.json"))
                )
                .unwrap(),
                journal
            );
            if sequence == 17 {
                assert!(!root.join("execution-stage-3").exists());
            }
            assert_eq!(fs::read(source_journal.path()).unwrap(), source_bytes);
            assert_eq!(
                source_layout.restore_references().unwrap(),
                source_references
            );
            return;
        }
        let (_, checkpoint) = result.unwrap();
        predecessor = Some(checkpoint);
        drop(safety);
        drop(stage);
        if sequence == 17 {
            acceptance = Some(verify_loaded(
                &mut backend.borrow_mut(),
                metadata,
                destination,
                &source.chunks,
            ));
        }
    }
    backend.borrow().assert_state(42);
    assert_eq!(fs::read(source_journal.path()).unwrap(), source_bytes);
    assert_eq!(
        source_layout.restore_references().unwrap(),
        source_references
    );
}
