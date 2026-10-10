//! Real retained originals and rejection boundaries; providers here prove no IC effects.

use super::*;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_lifecycle_reply::IcLifecycleReply,
        ic_mutation::IcMutationAcknowledgementInput,
        ic_request::IcManagementMethodRecord as Method,
        restore_safety::{RestoreSafetyLaneRecord, RestoreSafetyRequestInput},
    },
    ops::persistence::{
        create_execution_workflow, create_operation_plan, create_restore_safety_requirement,
        read_execution_progress,
    },
    test_support::{
        restore_safety::{input, plan, requirement, source, wire},
        temp_dir,
    },
};
use std::{
    fs,
    io::{self, Write},
};

fn single(method: Method) -> OperationPlanRecord {
    let mut value = serde_json::to_value(plan()).unwrap();
    let index = usize::from(method == Method::StartCanister);
    let operation = value["operations"][index].clone();
    let sequence = operation["operation_sequence"].clone();
    value["operations"] = serde_json::json!([operation]);
    value["graph"]["nodes"] = serde_json::json!([{"operation_sequence":sequence,"depends_on":[]}]);
    value["budget"] = serde_json::json!({"mutations":1,"observations":1});
    serde_json::from_value(value).unwrap()
}
struct Provider {
    calls: usize,
    reply: Result<Vec<u8>, IcMutationProviderError>,
}
impl IcMutationProvider for Provider {
    fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        self.calls += 1;
        let reply = self.reply.clone()?;
        Ok(
            IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                context: request.plan().context().clone(),
                target: request.payload().target().into(),
                evidence: ArtifactChecksumRecord::from_bytes(&reply),
                reply,
            })
            .unwrap(),
        )
    }
}
fn receipt(
    request: &IcMutationRequest<'_>,
    reply: &IcMutationAcknowledgement,
) -> MutationReceiptRequest {
    MutationReceiptRequest {
        attempt: request.mutation_attempt(),
        request: request.payload().digest().hash().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: reply.input().evidence.hash().into(),
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "keep retained source, dispatch exclusion and reopen proof in one trace"
)]
fn load_and_start_qualify_under_exclusion_then_checkpoint_without_reissue() {
    for method in [Method::LoadCanisterSnapshot, Method::StartCanister] {
        let root = temp_dir("ic-backup-restore-step");
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("source")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let source = source();
        create_operation_plan(&source_layout, &source).unwrap();
        let plan = single(method);
        let workflow = ExecutionWorkflowRecord::new(plan.clone());
        create_execution_workflow(&layout, &workflow).unwrap();
        let sequence = plan.operations()[0].operation_sequence();
        let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap();
        let requirement = requirement(
            stage.plan(),
            &source,
            RestoreSafetyLaneRecord::ApplicationFenced,
        );
        create_restore_safety_requirement(
            stage.layout().unwrap(),
            &source_layout,
            stage.plan(),
            &source,
            &requirement,
        )
        .unwrap();
        let payload = wire(method);
        let safety = RestoreSafetyRequest::new(
            stage.plan(),
            &source,
            &requirement,
            &payload,
            RestoreSafetyRequestInput {
                operation_sequence: sequence,
                challenge: ArtifactChecksumRecord::from_bytes(b"fresh fixture challenge"),
                max_remote_observations: 1,
            },
        )
        .unwrap();
        let mut provider = Provider {
            calls: 0,
            reply: Ok(b"DIDL\0\0".to_vec()),
        };
        let (reply, checkpoint) = restore_snapshot(
            &stage,
            &source_layout,
            &source,
            &safety,
            &mut provider,
            |request, safety| {
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                Ok::<_, io::Error>(RestoreSafetyObservation::new(input(safety)).unwrap())
            },
            |request, reply| {
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                let file =
                    fs::File::create(stage.layout().unwrap().root().join("retained-reply.candid"))?;
                (&file).write_all(&reply.input().reply)?;
                file.sync_all()?;
                fs::File::open(stage.layout().unwrap().root())?.sync_all()?;
                Ok(receipt(request, reply))
            },
        )
        .unwrap();
        assert_eq!(
            checkpoint.learned_evidence(),
            &IcLifecycleReply::decode(&payload, &reply.input().reply)
                .unwrap()
                .digest()
        );
        assert_eq!(provider.calls, 1);
        drop(safety);
        drop(stage);
        let (stage, progress) =
            ExecutionStageGuard::resume(&layout, binding.workflow(), sequence, &binding.digest())
                .unwrap();
        assert_eq!(progress.applied_operations, 1);
        let original = fs::read(
            stage
                .layout()
                .unwrap()
                .root()
                .join(format!("attempt-{sequence}.json")),
        )
        .unwrap();
        let safety = RestoreSafetyRequest::new(
            stage.plan(),
            &source,
            &requirement,
            &payload,
            RestoreSafetyRequestInput {
                operation_sequence: sequence,
                challenge: ArtifactChecksumRecord::from_bytes(b"new challenge cannot grant retry"),
                max_remote_observations: 1,
            },
        )
        .unwrap();
        assert!(
            restore_snapshot(
                &stage,
                &source_layout,
                &source,
                &safety,
                &mut provider,
                |_, _| -> Result<_, io::Error> { panic!("Applied never admits") },
                |_, _| -> Result<_, io::Error> { panic!("Applied never settles") }
            )
            .is_err()
        );
        assert_eq!(provider.calls, 1);
        assert_eq!(
            fs::read(
                stage
                    .layout()
                    .unwrap()
                    .root()
                    .join(format!("attempt-{sequence}.json"))
            )
            .unwrap(),
            original
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "each fault keeps its original reservation, returned evidence and no-reissue proof together"
)]
fn failure_boundaries_retain_pending_or_applied_and_never_repeat_provider() {
    for failure in 0..12 {
        let root = temp_dir("ic-backup-restore-rejection");
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("source")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let source = source();
        create_operation_plan(&source_layout, &source).unwrap();
        let method = if failure >= 8 {
            Method::StartCanister
        } else {
            Method::LoadCanisterSnapshot
        };
        let plan = single(method);
        let sequence = plan.operations()[0].operation_sequence();
        let workflow = ExecutionWorkflowRecord::new(plan.clone());
        create_execution_workflow(&layout, &workflow).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
        let requirement = requirement(
            stage.plan(),
            &source,
            if failure >= 8 {
                RestoreSafetyLaneRecord::ApplicationFenced
            } else {
                RestoreSafetyLaneRecord::NoIrreversibleEffects
            },
        );
        create_restore_safety_requirement(
            stage.layout().unwrap(),
            &source_layout,
            stage.plan(),
            &source,
            &requirement,
        )
        .unwrap();
        let payload = wire(method);
        let safety = RestoreSafetyRequest::new(
            stage.plan(),
            &source,
            &requirement,
            &payload,
            RestoreSafetyRequestInput {
                operation_sequence: sequence,
                challenge: ArtifactChecksumRecord::from_bytes(b"fresh challenge"),
                max_remote_observations: 1,
            },
        )
        .unwrap();
        let mut provider = Provider {
            calls: 0,
            reply: match failure {
                2 => Err(IcMutationProviderError::Indeterminate),
                3 => Ok(vec![]),
                _ => Ok(b"DIDL\0\0".to_vec()),
            },
        };
        let requirement_path = stage
            .layout()
            .unwrap()
            .root()
            .join("restore-safety-requirement.json");
        if failure == 11 {
            fs::rename(
                &requirement_path,
                requirement_path.with_extension("retained-original"),
            )
            .unwrap();
        }
        if failure == 7 {
            fs::write(
                stage
                    .layout()
                    .unwrap()
                    .root()
                    .join("execution-settlement.json"),
                b"occupied original evidence",
            )
            .unwrap();
        }
        let result = restore_snapshot(
            &stage,
            &source_layout,
            &source,
            &safety,
            &mut provider,
            |_, safety| {
                if failure == 0 {
                    return Err(io::Error::other("fresh admission refused"));
                }
                let mut actual = input(safety);
                if failure == 1 {
                    actual.targets[0].state =
                        ic_management_canister_types::CanisterStatusType::Running;
                }
                if failure == 6 {
                    fs::write(&requirement_path, b"{}")?;
                }
                if failure == 8 {
                    actual.targets[0].restored_acceptance = None;
                }
                if let crate::model::restore_safety::RestoreSafetyEvidence::ApplicationFenced(
                    fence,
                ) = &mut actual.safety
                {
                    if failure == 9 {
                        fence.state = crate::model::consistency::ApplicationFenceState::Inactive;
                    }
                    if failure == 10 {
                        fence.controlled_execution = None;
                    }
                }
                Ok(RestoreSafetyObservation::new(actual).unwrap())
            },
            |request, reply| {
                if failure == 4 {
                    return Err(io::Error::other("durable qualification refused"));
                }
                let mut receipt = receipt(request, reply);
                if failure == 5 {
                    receipt.attempt += 1;
                }
                Ok(receipt)
            },
        );
        match (failure, result) {
            (0, Err(IcRestoreExecutionError::Admission(_)))
            | (1, Err(IcRestoreExecutionError::Safety(RestoreSafetyError::TargetNotStopped)))
            | (2, Err(IcRestoreExecutionError::Provider(IcMutationProviderError::Indeterminate)))
            | (6 | 11, Err(IcRestoreExecutionError::Retention(_)))
            | (
                8,
                Err(IcRestoreExecutionError::Safety(
                    RestoreSafetyError::RestoredAcceptanceRequired,
                )),
            )
            | (9, Err(IcRestoreExecutionError::Safety(RestoreSafetyError::FenceNotActive)))
            | (
                10,
                Err(IcRestoreExecutionError::Safety(
                    RestoreSafetyError::ControlledExecutionRequired,
                )),
            ) => {}
            (
                3..=5 | 7,
                Err(IcRestoreExecutionError::AfterReply {
                    acknowledgement, ..
                }),
            ) => assert_eq!(
                acknowledgement.input().reply,
                if failure == 3 {
                    &[][..]
                } else {
                    &b"DIDL\0\0"[..]
                }
            ),
            (_, result) => panic!("boundary {failure}: {result:?}"),
        }
        assert_eq!(
            provider.calls,
            usize::from(!matches!(failure, 0 | 1 | 6 | 8..=11))
        );
        let progress =
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
        assert_eq!(progress.attempts.mutations_used, u32::from(failure != 11));
        assert_eq!(progress.applied_operations, usize::from(failure == 7));
        let original = fs::read(
            stage
                .layout()
                .unwrap()
                .root()
                .join(format!("attempt-{sequence}.json")),
        )
        .unwrap();
        let calls = provider.calls;
        assert!(
            restore_snapshot(
                &stage,
                &source_layout,
                &source,
                &safety,
                &mut provider,
                |_, _| -> Result<_, io::Error> { panic!("pending never admits") },
                |_, _| -> Result<_, io::Error> { panic!("pending never settles") }
            )
            .is_err()
        );
        assert_eq!(provider.calls, calls);
        assert_eq!(
            fs::read(
                stage
                    .layout()
                    .unwrap()
                    .root()
                    .join(format!("attempt-{sequence}.json"))
            )
            .unwrap(),
            original
        );
    }
}
