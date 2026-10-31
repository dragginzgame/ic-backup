//! Real retained originals and rejection boundaries; providers here prove no IC effects.

use super::*;
use crate::model::operation_plan::{PlanBudgetRecord, PlannedOperationRecord};
use crate::test_support::operation_plan::{declaration, graph, operation_request};

use crate::ops::persistence::AttemptJournalGuard;
use crate::test_support::ready::ready;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_lifecycle_reply::IcLifecycleReply,
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
        operation_plan::OperationPlanRecord,
        restore_safety::{RestoreSafetyLaneRecord, RestoreSafetyRequestInput},
    },
    ops::persistence::{
        BackupLayoutGuard, create_execution_workflow, create_operation_plan,
        create_restore_safety_requirement, read_execution_progress,
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

fn existing<'a>(
    layout: &'a BackupLayoutGuard,
    plan: &'a OperationPlanRecord,
) -> IcRestoreSource<'a> {
    let load = wire(Method::LoadCanisterSnapshot);
    let metadata =
        IcSnapshotMetadataRequest::new(load.target(), load.snapshot_id().unwrap()).unwrap();
    IcRestoreSource::existing(layout, plan, &metadata)
}

fn single(method: Method) -> OperationPlanRecord {
    let original = plan();
    let index = usize::from(method == Method::StartCanister);
    let operation = original.operations()[index].clone();
    let mut request = declaration(&original);
    request.graph = graph([(operation.operation_sequence(), vec![])]);
    request.operations = vec![operation];
    request.budget = PlanBudgetRecord::new(1, 1).unwrap();
    OperationPlanRecord::new(request).unwrap()
}
struct Provider {
    calls: usize,
    reply: Result<Vec<u8>, IcMutationProviderError>,
}
impl IcMutationProvider for Provider {
    fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &crate::model::attempt_journal::AttemptJournalRecord,
    ) -> impl std::future::Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>>
    {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            self.calls += 1;
            let reply = self.reply.clone()?;
            Ok(
                IcMutationAcknowledgement::new(crate::test_support::update::input(
                    request.authority(),
                    request.mutation_attempt(),
                    request.plan().context(),
                    reply,
                ))
                .unwrap(),
            )
        })())
    }
}
fn receipt(
    request: &IcMutationRequest<'_>,
    reply: &IcMutationAcknowledgement,
) -> MutationReceiptRequest {
    crate::test_support::update::applied_receipt(
        request.mutation_attempt(),
        &request.payload().digest(),
        &reply.input().evidence,
    )
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "keep retained source, dispatch exclusion and reopen proof in one trace"
)]
fn load_and_start_qualify_under_exclusion_then_checkpoint_without_reissue() {
    for method in [Method::LoadCanisterSnapshot, Method::StartCanister] {
        let root = temp_dir("ic-backup-restore-step");
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
        let (reply, checkpoint) = ready(restore_snapshot(
            &stage,
            existing(&source_layout, &source),
            &safety,
            &mut provider,
            async |request, safety| {
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                Ok::<_, io::Error>(RestoreSafetyObservation::new(input(safety)).unwrap())
            },
            async |request, reply| {
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
        ))
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
            ready(restore_snapshot(
                &stage,
                existing(&source_layout, &source),
                &safety,
                &mut provider,
                async |_, _| -> Result<_, io::Error> { panic!("Applied never admits") },
                async |_, _| -> Result<_, io::Error> { panic!("Applied never settles") }
            ))
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
fn cancellation_during_qualification_keeps_durable_reply_and_pending_original() {
    let root = temp_dir("ic-backup-restore-cancel");
    fs::create_dir(root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let source = source();
    create_operation_plan(&source_layout, &source).unwrap();
    let plan = single(Method::LoadCanisterSnapshot);
    let sequence = plan.operations()[0].operation_sequence();
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    create_execution_workflow(&layout, &workflow).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
    let requirement = requirement(
        stage.plan(),
        &source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
    create_restore_safety_requirement(
        stage.layout().unwrap(),
        &source_layout,
        stage.plan(),
        &source,
        &requirement,
    )
    .unwrap();
    let payload = wire(Method::LoadCanisterSnapshot);
    let safety = RestoreSafetyRequest::new(
        stage.plan(),
        &source,
        &requirement,
        &payload,
        RestoreSafetyRequestInput {
            operation_sequence: sequence,
            challenge: ArtifactChecksumRecord::from_bytes(b"original cancellation challenge"),
            max_remote_observations: 1,
        },
    )
    .unwrap();
    let mut provider = Provider {
        calls: 0,
        reply: Ok(b"DIDL\0\0".to_vec()),
    };
    let reply_path = root.join("retained-reply.candid");
    let mut future = Box::pin(restore_snapshot(
        &stage,
        existing(&source_layout, &source),
        &safety,
        &mut provider,
        async |_, safety| Ok::<_, io::Error>(RestoreSafetyObservation::new(input(safety)).unwrap()),
        async |request, reply| {
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            let file = fs::File::create(&reply_path)?;
            (&file).write_all(&reply.input().reply)?;
            file.sync_all()?;
            fs::File::open(&root)?.sync_all()?;
            std::future::pending::<()>().await;
            Ok(receipt(request, reply))
        },
    ));
    crate::test_support::ready::assert_pending(future.as_mut());
    let authority = stage.plan().attempt_authority(sequence).unwrap();
    assert!(AttemptJournalGuard::open(stage.layout().unwrap(), &authority).is_err());
    let journal_path = stage
        .layout()
        .unwrap()
        .root()
        .join(format!("attempt-{sequence}.json"));
    let original = fs::read(&journal_path).unwrap();
    drop(future);
    assert_eq!(provider.calls, 1);
    let journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
    drop(journal);
    assert!(
        ready(restore_snapshot(
            &stage,
            existing(&source_layout, &source),
            &safety,
            &mut provider,
            async |_, _| -> Result<_, io::Error> { panic!("pending never readmits") },
            async |_, _| -> Result<_, io::Error> { panic!("pending never requalifies") },
        ))
        .is_err()
    );
    assert_eq!(provider.calls, 1);
    assert_eq!(fs::read(journal_path).unwrap(), original);
    assert_eq!(fs::read(reply_path).unwrap(), b"DIDL\0\0");
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RestoreFailure {
    Admission,
    RunningTarget,
    LostReply,
    MalformedReply,
    Qualification,
    WrongReceipt,
    ChangedRequirement,
    OccupiedCheckpoint,
    MissingAcceptance,
    InactiveFence,
    MissingControlledExecution,
    MissingRequirement,
}
impl RestoreFailure {
    const ALL: [Self; 12] = [
        Self::Admission,
        Self::RunningTarget,
        Self::LostReply,
        Self::MalformedReply,
        Self::Qualification,
        Self::WrongReceipt,
        Self::ChangedRequirement,
        Self::OccupiedCheckpoint,
        Self::MissingAcceptance,
        Self::InactiveFence,
        Self::MissingControlledExecution,
        Self::MissingRequirement,
    ];
    fn starts(self) -> bool {
        matches!(
            self,
            Self::MissingAcceptance
                | Self::InactiveFence
                | Self::MissingControlledExecution
                | Self::MissingRequirement
        )
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "each fault keeps its original reservation, returned evidence and no-reissue proof together"
)]
fn failure_boundaries_retain_pending_or_applied_and_never_repeat_provider() {
    for failure in RestoreFailure::ALL {
        let root = temp_dir("ic-backup-restore-rejection");
        fs::create_dir(root.join("source")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let source = source();
        create_operation_plan(&source_layout, &source).unwrap();
        let method = if failure.starts() {
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
            if failure.starts() {
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
                RestoreFailure::LostReply => Err(IcMutationProviderError::Indeterminate),
                RestoreFailure::MalformedReply => Ok(vec![]),
                _ => Ok(b"DIDL\0\0".to_vec()),
            },
        };
        let requirement_path = stage
            .layout()
            .unwrap()
            .root()
            .join("restore-safety-requirement.json");
        if failure == RestoreFailure::MissingRequirement {
            fs::rename(
                &requirement_path,
                requirement_path.with_extension("retained-original"),
            )
            .unwrap();
        }
        if failure == RestoreFailure::OccupiedCheckpoint {
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
        let result = ready(restore_snapshot(
            &stage,
            existing(&source_layout, &source),
            &safety,
            &mut provider,
            async |_, safety| {
                if failure == RestoreFailure::Admission {
                    return Err(io::Error::other("fresh admission refused"));
                }
                let mut actual = input(safety);
                if failure == RestoreFailure::RunningTarget {
                    actual.targets[0].state =
                        ic_platform::management_canister_types::CanisterStatusType::Running;
                }
                if failure == RestoreFailure::ChangedRequirement {
                    fs::write(&requirement_path, b"{}")?;
                }
                if failure == RestoreFailure::MissingAcceptance {
                    actual.targets[0].restored_acceptance = None;
                }
                if let crate::model::restore_safety::RestoreSafetyEvidence::ApplicationFenced(
                    fence,
                ) = &mut actual.safety
                {
                    if failure == RestoreFailure::InactiveFence {
                        fence.state = crate::model::consistency::ApplicationFenceState::Inactive;
                    }
                    if failure == RestoreFailure::MissingControlledExecution {
                        fence.controlled_execution = None;
                    }
                }
                Ok(RestoreSafetyObservation::new(actual).unwrap())
            },
            async |request, reply| {
                if failure == RestoreFailure::Qualification {
                    return Err(io::Error::other("durable qualification refused"));
                }
                let mut receipt = receipt(request, reply);
                if failure == RestoreFailure::WrongReceipt {
                    receipt.attempt += 1;
                }
                Ok(receipt)
            },
        ));
        match (failure, result) {
            (RestoreFailure::Admission, Err(IcRestoreExecutionError::Admission(_)))
            | (
                RestoreFailure::RunningTarget,
                Err(IcRestoreExecutionError::Safety(RestoreSafetyError::TargetNotStopped)),
            )
            | (
                RestoreFailure::LostReply,
                Err(IcRestoreExecutionError::Provider(IcMutationProviderError::Indeterminate)),
            )
            | (
                RestoreFailure::ChangedRequirement | RestoreFailure::MissingRequirement,
                Err(IcRestoreExecutionError::Retention(_)),
            )
            | (
                RestoreFailure::MissingAcceptance,
                Err(IcRestoreExecutionError::Safety(
                    RestoreSafetyError::RestoredAcceptanceRequired,
                )),
            )
            | (
                RestoreFailure::InactiveFence,
                Err(IcRestoreExecutionError::Safety(RestoreSafetyError::FenceNotActive)),
            )
            | (
                RestoreFailure::MissingControlledExecution,
                Err(IcRestoreExecutionError::Safety(
                    RestoreSafetyError::ControlledExecutionRequired,
                )),
            ) => {}
            (
                RestoreFailure::MalformedReply
                | RestoreFailure::Qualification
                | RestoreFailure::WrongReceipt
                | RestoreFailure::OccupiedCheckpoint,
                Err(IcRestoreExecutionError::AfterReply {
                    acknowledgement, ..
                }),
            ) => assert_eq!(
                acknowledgement.input().reply,
                if failure == RestoreFailure::MalformedReply {
                    &[][..]
                } else {
                    &b"DIDL\0\0"[..]
                }
            ),
            (_, result) => panic!("boundary {failure:?}: {result:?}"),
        }
        assert_eq!(
            provider.calls,
            usize::from(!matches!(
                failure,
                RestoreFailure::Admission
                    | RestoreFailure::RunningTarget
                    | RestoreFailure::ChangedRequirement
                    | RestoreFailure::MissingAcceptance
                    | RestoreFailure::InactiveFence
                    | RestoreFailure::MissingControlledExecution
                    | RestoreFailure::MissingRequirement
            ))
        );
        let progress =
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
        assert_eq!(
            progress.attempts.mutations_used,
            u32::from(failure != RestoreFailure::MissingRequirement)
        );
        assert_eq!(
            progress.applied_operations,
            usize::from(failure == RestoreFailure::OccupiedCheckpoint)
        );
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
            ready(restore_snapshot(
                &stage,
                existing(&source_layout, &source),
                &safety,
                &mut provider,
                async |_, _| -> Result<_, io::Error> { panic!("pending never admits") },
                async |_, _| -> Result<_, io::Error> { panic!("pending never settles") }
            ))
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

#[test]
fn independently_valid_load_plan_cannot_rebind_the_admitted_snapshot_source() {
    use crate::model::ic_request::{IcManagementRequest, IcManagementRequestRecord};
    let root = temp_dir("restore-different-valid-snapshot");
    fs::create_dir(root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let original_source = source();
    create_operation_plan(&source_layout, &original_source).unwrap();
    let original = wire(Method::LoadCanisterSnapshot);
    let other = IcManagementRequestRecord::new(IcManagementRequest {
        method: original.method(),
        target: original.target().into(),
        snapshot_id: Some(vec![88, 99]),
    })
    .unwrap();
    let mut fields = declaration(&single(Method::LoadCanisterSnapshot));
    let mut operation = operation_request(&fields.operations[0]);
    operation.request = other.digest().hash().into();
    fields.operations[0] = PlannedOperationRecord::new(operation).unwrap();
    let plan = OperationPlanRecord::new(fields).unwrap();
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    create_execution_workflow(&layout, &workflow).unwrap();
    let sequence = plan.operations()[0].operation_sequence();
    let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
    let requirement = requirement(
        stage.plan(),
        &original_source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
    create_restore_safety_requirement(
        stage.layout().unwrap(),
        &source_layout,
        stage.plan(),
        &original_source,
        &requirement,
    )
    .unwrap();
    // Both the new plan and its exact load request are internally valid.
    let safety = RestoreSafetyRequest::new(
        stage.plan(),
        &original_source,
        &requirement,
        &other,
        RestoreSafetyRequestInput {
            operation_sequence: sequence,
            challenge: ArtifactChecksumRecord::from_bytes(b"fresh"),
            max_remote_observations: 0,
        },
    )
    .unwrap();
    let mut provider = Provider {
        calls: 0,
        reply: Ok(crate::model::ic_lifecycle_reply::EMPTY_CANDID_REPLY.to_vec()),
    };
    let result = ready(restore_snapshot(
        &stage,
        existing(&source_layout, &original_source),
        &safety,
        &mut provider,
        async |_, _| -> Result<_, io::Error> { panic!("wrong source cannot admit") },
        async |_, _| -> Result<_, io::Error> { panic!("wrong source cannot settle") },
    ));
    assert!(matches!(
        result,
        Err(IcRestoreExecutionError::Retention(
            IcRestoreReplyError::Source(IcRestoreSourceError::Mismatch)
        ))
    ));
    assert_eq!(provider.calls, 0);
    assert_eq!(
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest())
            .unwrap()
            .attempts
            .mutations_used,
        0
    );
}
