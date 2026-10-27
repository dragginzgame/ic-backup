//! Native orchestration/custody fixtures; no application fence semantics or IC effects.

use super::*;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptJournalRecord, MutationOutcomeRecord, MutationReceiptRequest},
        consistency::{
            ApplicationFenceBinding, ConsistencyGuaranteeRecord, ConsistencyRequirementRecord,
        },
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionWorkflowRecord,
            tests::{child_request, workflow},
        },
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreFenceBindingRecord, RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest,
        },
    },
    ops::persistence::{
        BackupLayoutGuard, create_consistency_requirement, create_execution_workflow,
        create_fence_obligation, create_operation_plan, create_restore_safety_requirement,
    },
    test_support::{ready::ready, temp_dir},
};
use std::{convert::Infallible, fs, path::PathBuf};

mod reconciliation;

struct Fixture {
    layout: BackupLayoutGuard,
    source_layout: BackupLayoutGuard,
    binding: ExecutionStageBindingRecord,
    plan: OperationPlanRecord,
    source: OperationPlanRecord,
    payload: FenceAcquisitionPayload,
    consistency: ConsistencyRequirementRecord,
    fence: ApplicationFenceBinding,
    safety: RestoreSafetyRequirementRecord,
    obligation: FenceObligationRecord,
    restore: bool,
}
impl Fixture {
    fn new(restore: bool, dependency: bool) -> Self {
        let root = temp_dir("ic-backup-fence-step");
        fs::create_dir(root.join("source")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let payload = FenceAcquisitionPayload::new(
            workflow().stage(0).unwrap().target(),
            "native_acquire",
            b"exact original application bytes",
        )
        .unwrap();
        let (workflow, plan) = planned_stage(&payload, dependency);
        create_execution_workflow(&layout, &workflow).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan.clone()).unwrap();
        let consistency = ConsistencyRequirementRecord::new(
            &plan,
            ConsistencyGuaranteeRecord::ApplicationCoordinated,
        );
        let fence = ApplicationFenceBinding {
            identity: ArtifactChecksumRecord::from_bytes(b"original fence"),
            membership_revision: ArtifactChecksumRecord::from_bytes(b"original membership"),
        };
        let mut source = serde_json::to_value(&plan).unwrap();
        source["operations"][0]["request"] =
            serde_json::json!(ArtifactChecksumRecord::from_bytes(b"source request").hash());
        let source = serde_json::from_value(source).unwrap();
        create_operation_plan(&source_layout, &source).unwrap();
        let safety = RestoreSafetyRequirementRecord::new(
            &plan,
            &source,
            RestoreSafetyRequirementRequest {
                source_artifacts: ArtifactChecksumRecord::from_bytes(b"original source artifacts"),
                safety: RestoreSafetyLaneRecord::ApplicationFenced,
                expected_fence: Some(RestoreFenceBindingRecord {
                    identity: fence.identity.clone(),
                    membership_revision: fence.membership_revision.clone(),
                    external_obligations_revision: ArtifactChecksumRecord::from_bytes(
                        b"outside-snapshot obligations",
                    ),
                }),
            },
        )
        .unwrap();
        let obligation = if restore {
            source_layout
                .retain_restore(
                    &stage.layout().unwrap().root().join("attempt-42.json"),
                    plan.digest().hash(),
                )
                .unwrap();
            create_restore_safety_requirement(
                stage.layout().unwrap(),
                &source_layout,
                &plan,
                &source,
                &safety,
            )
            .unwrap();
            FenceObligationRecord::for_restore(&plan, &source, &safety, 42).unwrap()
        } else {
            create_consistency_requirement(stage.layout().unwrap(), &plan, &consistency).unwrap();
            FenceObligationRecord::for_capture(&plan, &consistency, 42, &fence).unwrap()
        };
        let requirement = if restore {
            FenceObligationRequirement::Restore {
                source_layout: &source_layout,
                source: &source,
                requirement: &safety,
            }
        } else {
            FenceObligationRequirement::Capture {
                requirement: &consistency,
                fence: &fence,
            }
        };
        create_fence_obligation(stage.layout().unwrap(), &plan, requirement, &obligation).unwrap();
        drop(stage);
        Self {
            layout,
            source_layout,
            binding,
            plan,
            source,
            payload,
            consistency,
            fence,
            safety,
            obligation,
            restore,
        }
    }
    fn requirement(&self) -> FenceObligationRequirement<'_> {
        if self.restore {
            FenceObligationRequirement::Restore {
                source_layout: &self.source_layout,
                source: &self.source,
                requirement: &self.safety,
            }
        } else {
            FenceObligationRequirement::Capture {
                requirement: &self.consistency,
                fence: &self.fence,
            }
        }
    }
    fn stage(&self) -> ExecutionStageGuard<'_> {
        ExecutionStageGuard::open(
            &self.layout,
            self.binding.workflow(),
            0,
            &self.binding.digest(),
        )
        .unwrap()
    }
}
fn admit(_: &FenceAcquisitionRequest<'_>) -> impl Future<Output = Result<(), Infallible>> {
    std::future::ready(Ok(()))
}
struct Provider {
    calls: usize,
    failure: Option<FenceAcquisitionProviderError>,
    change: Option<PathBuf>,
    mismatch: bool,
    replace_layout: Option<PathBuf>,
}
impl Provider {
    fn new(failure: Option<FenceAcquisitionProviderError>) -> Self {
        Self {
            calls: 0,
            failure,
            change: None,
            mismatch: false,
            replace_layout: None,
        }
    }
}
impl FenceAcquisitionProvider for Provider {
    fn acquire_fence(
        &mut self,
        request: &FenceAcquisitionRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> impl Future<Output = Result<FenceAcquisitionAcknowledgement, FenceAcquisitionProviderError>>
    {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            self.calls += 1;
            if let Some(path) = &self.replace_layout {
                fs::rename(path, path.with_extension("retained-original")).unwrap();
                fs::create_dir(path).unwrap();
            }
            if let Some(path) = &self.change {
                fs::write(path, b"{}").unwrap();
            }
            if let Some(error) = self.failure {
                return Err(error);
            }
            Ok(FenceAcquisitionAcknowledgement::new(
                if self.mismatch {
                    ArtifactChecksumRecord::from_bytes(b"other authority")
                } else {
                    request.authority().digest()
                },
                request.mutation_attempt(),
                ArtifactChecksumRecord::from_bytes(b"retained native reply"),
            )
            .unwrap())
        })())
    }
}
fn assert_pending(stage: &ExecutionStageGuard<'_>, plan: &OperationPlanRecord) -> Vec<u8> {
    let journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &plan.attempt_authority(42).unwrap(),
    )
    .unwrap();
    let view = journal.record().unwrap().view();
    assert_eq!(view.mutations_used, 1);
    assert_eq!(view.pending_mutation, Some(1));
    assert!(!view.applied);
    fs::read(journal.path()).unwrap()
}

#[test]
fn both_purposes_preserve_pending_originals_on_acknowledgement_and_provider_failure() {
    for restore in [false, true] {
        for failure in [
            None,
            Some(FenceAcquisitionProviderError::Unavailable),
            Some(FenceAcquisitionProviderError::Unsupported),
            Some(FenceAcquisitionProviderError::Indeterminate),
        ] {
            let fixture = Fixture::new(restore, false);
            let stage = fixture.stage();
            let original =
                fs::read(stage.layout().unwrap().root().join("fence-obligation.json")).unwrap();
            let references = fixture.source_layout.restore_references().unwrap();
            let mut provider = Provider::new(failure);
            let result = ready(acquire_fence(
                &stage,
                fixture.requirement(),
                &fixture.obligation,
                &fixture.payload,
                &mut provider,
                async |request| {
                    assert!(
                        AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                            .is_err()
                    );
                    assert_eq!(request.obligation(), &fixture.obligation);
                    assert_eq!(request.payload().arguments(), fixture.payload.arguments());
                    Ok::<(), Infallible>(())
                },
            ));
            match (failure, result) {
                (None, Ok(reply)) => assert_eq!(
                    reply.evidence,
                    ArtifactChecksumRecord::from_bytes(b"retained native reply")
                ),
                (Some(expected), Err(FenceAcquisitionExecutionError::Provider(actual))) => {
                    assert_eq!(actual, expected);
                }
                other => panic!("unexpected result: {other:?}"),
            }
            let pending = assert_pending(&stage, &fixture.plan);
            assert!(
                ready(acquire_fence(
                    &stage,
                    fixture.requirement(),
                    &fixture.obligation,
                    &fixture.payload,
                    &mut provider,
                    admit
                ))
                .is_err()
            );
            assert_eq!(provider.calls, 1);
            assert_eq!(assert_pending(&stage, &fixture.plan), pending);
            assert_eq!(
                fixture.source_layout.restore_references().unwrap(),
                references
            );
            assert_eq!(
                fs::read(stage.layout().unwrap().root().join("fence-obligation.json")).unwrap(),
                original
            );
        }
    }
}

#[test]
fn original_binding_missing_obligation_and_missing_journal_reject_before_spending() {
    for boundary in 0..3 {
        let fixture = Fixture::new(false, false);
        let stage = fixture.stage();
        let layout = stage.layout().unwrap();
        let before = fs::read(layout.root().join("attempt-42.json")).unwrap();
        let changed = FenceAcquisitionPayload::new(
            fixture.payload.target(),
            "changed_method",
            fixture.payload.arguments(),
        )
        .unwrap();
        if boundary != 0 {
            let name = if boundary == 1 {
                "fence-obligation.json"
            } else {
                "attempt-42.json"
            };
            fs::rename(
                layout.root().join(name),
                layout.root().join(format!("retained-{name}")),
            )
            .unwrap();
        }
        let mut provider = Provider::new(None);
        let result = ready(acquire_fence(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            if boundary == 0 {
                &changed
            } else {
                &fixture.payload
            },
            &mut provider,
            async |_| -> Result<(), Infallible> {
                panic!("invalid originals reached fresh admission")
            },
        ));
        assert!(result.is_err());
        assert_eq!(provider.calls, 0);
        let journal = if boundary == 2 {
            "retained-attempt-42.json"
        } else {
            "attempt-42.json"
        };
        assert_eq!(fs::read(layout.root().join(journal)).unwrap(), before);
    }
}

#[test]
fn complete_journal_coverage_and_applied_prerequisites_are_required() {
    for missing in [false, true] {
        let fixture = Fixture::new(false, true);
        let stage = fixture.stage();
        let layout = stage.layout().unwrap();
        let before = fs::read(layout.root().join("attempt-42.json")).unwrap();
        if missing {
            fs::rename(
                layout.root().join("attempt-43.json"),
                layout.root().join("retained-attempt-43.json"),
            )
            .unwrap();
        }
        let mut provider = Provider::new(None);
        let result = ready(acquire_fence(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            &mut provider,
            admit,
        ));
        assert!(matches!(
            result,
            Err(FenceAcquisitionExecutionError::Progress(_))
        ));
        assert_eq!(provider.calls, 0);
        assert_eq!(
            fs::read(layout.root().join("attempt-42.json")).unwrap(),
            before
        );
        if missing {
            continue;
        }
        let authority = fixture.plan.attempt_authority(43).unwrap();
        let mut prerequisite = AttemptJournalGuard::open(layout, &authority).unwrap();
        let mutation = prerequisite
            .reserve_planned_mutation(&fixture.plan.digest())
            .unwrap();
        prerequisite
            .record_mutation(MutationReceiptRequest {
                request: authority.binding().request().into(),
                attempt: mutation,
                outcome: MutationOutcomeRecord::Applied,
                evidence: ArtifactChecksumRecord::from_bytes(b"independent prerequisite")
                    .hash()
                    .into(),
            })
            .unwrap();
        drop(prerequisite);
        ready(acquire_fence(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            &mut provider,
            admit,
        ))
        .unwrap();
        assert_eq!(provider.calls, 1);
        assert_pending(&stage, &fixture.plan);
    }
}

#[test]
fn admission_failure_consumes_original_without_dispatch_or_reentry() {
    let fixture = Fixture::new(true, false);
    let stage = fixture.stage();
    let mut provider = Provider::new(None);
    let result = ready(acquire_fence(
        &stage,
        fixture.requirement(),
        &fixture.obligation,
        &fixture.payload,
        &mut provider,
        async |_| Err(std::io::Error::other("fixture admission rejected")),
    ));
    assert!(matches!(
        result,
        Err(FenceAcquisitionExecutionError::Admission(_))
    ));
    let before = assert_pending(&stage, &fixture.plan);
    assert!(
        ready(acquire_fence(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            &mut provider,
            admit
        ))
        .is_err()
    );
    assert_eq!(provider.calls, 0);
    assert_eq!(assert_pending(&stage, &fixture.plan), before);
}

struct SuspendedProvider {
    calls: usize,
}
impl FenceAcquisitionProvider for SuspendedProvider {
    async fn acquire_fence(
        &mut self,
        request: &FenceAcquisitionRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<FenceAcquisitionAcknowledgement, FenceAcquisitionProviderError> {
        request.validate_journal(journal).unwrap();
        self.calls += 1;
        std::future::pending().await
    }
}
#[test]
fn cancellation_holds_exclusion_and_retains_pending_spending_and_source_references() {
    use std::task::{Context, Poll, Waker};
    for suspend_admission in [true, false] {
        let fixture = Fixture::new(true, false);
        let stage = fixture.stage();
        let references = fixture.source_layout.restore_references().unwrap();
        let mut provider = SuspendedProvider { calls: 0 };
        let mut future = Box::pin(acquire_fence(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            &mut provider,
            async |_| {
                if suspend_admission {
                    std::future::pending::<()>().await;
                }
                Ok::<(), Infallible>(())
            },
        ));
        assert!(matches!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop())),
            Poll::Pending
        ));
        assert!(
            AttemptJournalGuard::open(
                stage.layout().unwrap(),
                &fixture.plan.attempt_authority(42).unwrap()
            )
            .is_err()
        );
        let before = fs::read(stage.layout().unwrap().root().join("attempt-42.json")).unwrap();
        drop(future);
        assert_eq!(provider.calls, usize::from(!suspend_admission));
        assert_eq!(assert_pending(&stage, &fixture.plan), before);
        assert!(
            ready(acquire_fence(
                &stage,
                fixture.requirement(),
                &fixture.obligation,
                &fixture.payload,
                &mut Provider::new(None),
                admit
            ))
            .is_err()
        );
        assert_eq!(
            fixture.source_layout.restore_references().unwrap(),
            references
        );
    }
}

#[test]
fn changed_requirement_before_dispatch_preserves_reservation_without_provider_call() {
    let fixture = Fixture::new(true, false);
    let stage = fixture.stage();
    let path = fixture.source_layout.root().join("operation-plan.json");
    let mut provider = Provider::new(None);
    let result = ready(acquire_fence(
        &stage,
        fixture.requirement(),
        &fixture.obligation,
        &fixture.payload,
        &mut provider,
        async |_| {
            fs::write(path, b"{}").unwrap();
            Ok::<(), Infallible>(())
        },
    ));
    assert!(matches!(
        result,
        Err(FenceAcquisitionExecutionError::Obligation(_))
    ));
    assert_eq!(provider.calls, 0);
    assert_pending(&stage, &fixture.plan);
}

#[test]
fn after_reply_association_stage_and_obligation_rejections_retain_acknowledgement() {
    for boundary in 0..3 {
        let fixture = Fixture::new(false, false);
        let stage = fixture.stage();
        let root = stage.layout().unwrap().root().to_path_buf();
        let mut provider = Provider::new(None);
        provider.mismatch = boundary == 0;
        if boundary != 0 {
            provider.change = Some(root.join(if boundary == 1 {
                "stage-binding.json"
            } else {
                "fence-obligation.json"
            }));
        }
        let result = ready(acquire_fence(
            &stage,
            fixture.requirement(),
            &fixture.obligation,
            &fixture.payload,
            &mut provider,
            admit,
        ))
        .unwrap_err();
        let acknowledgement = match result {
            FenceAcquisitionExecutionError::AfterReply {
                source: FenceAcquisitionReplyError::Association(_),
                acknowledgement,
                ..
            } if boundary == 0 => acknowledgement,
            FenceAcquisitionExecutionError::AfterReply {
                source: FenceAcquisitionReplyError::Stage(_),
                acknowledgement,
                ..
            } if boundary == 1 => acknowledgement,
            FenceAcquisitionExecutionError::AfterReply {
                source: FenceAcquisitionReplyError::Obligation(_),
                acknowledgement,
                ..
            } if boundary == 2 => acknowledgement,
            other => panic!("unexpected post-reply error: {other:?}"),
        };
        assert_eq!(
            acknowledgement.evidence,
            ArtifactChecksumRecord::from_bytes(b"retained native reply")
        );
        assert_eq!(provider.calls, 1);
        let journal: AttemptJournalRecord =
            serde_json::from_slice(&fs::read(root.join("attempt-42.json")).unwrap()).unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert_eq!(journal.view().mutations_used, 1);
    }
}

fn planned_stage(
    payload: &FenceAcquisitionPayload,
    dependency: bool,
) -> (ExecutionWorkflowRecord, OperationPlanRecord) {
    let mut allocation = serde_json::to_value(workflow().allocation()).unwrap();
    let mut child =
        serde_json::to_value(OperationPlanRecord::new(child_request(0)).unwrap()).unwrap();
    child["operations"][0]["request"] = serde_json::json!(payload.digest().hash());
    if dependency {
        allocation["operations"][0]["budget"]["mutations"] = serde_json::json!(2);
        allocation["budget"]["mutations"] = serde_json::json!(4);
        child["graph"]["nodes"][0]["depends_on"] = serde_json::json!([43]);
        child["graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"operation_sequence":43,"depends_on":[]}));
        let mut prerequisite = child["operations"][0].clone();
        prerequisite["operation_sequence"] = serde_json::json!(43);
        prerequisite["budget"]["observations"] = serde_json::json!(0);
        child["operations"]
            .as_array_mut()
            .unwrap()
            .push(prerequisite);
        child["budget"]["mutations"] = serde_json::json!(2);
    }
    let workflow = ExecutionWorkflowRecord::new(serde_json::from_value(allocation).unwrap());
    let plan: OperationPlanRecord = serde_json::from_value(child).unwrap();
    (workflow, plan)
}

#[test]
fn replaced_layout_after_reply_retains_journal_and_acknowledgement() {
    let fixture = Fixture::new(true, false);
    let stage = fixture.stage();
    let root = stage.layout().unwrap().root().to_path_buf();
    let references = fixture.source_layout.restore_references().unwrap();
    let mut provider = Provider::new(None);
    provider.replace_layout = Some(root.clone());
    let error = ready(acquire_fence(
        &stage,
        fixture.requirement(),
        &fixture.obligation,
        &fixture.payload,
        &mut provider,
        admit,
    ))
    .unwrap_err();
    let FenceAcquisitionExecutionError::AfterReply {
        source: FenceAcquisitionReplyError::Journal(_),
        acknowledgement,
        ..
    } = error
    else {
        panic!("unexpected replaced-layout rejection")
    };
    assert_eq!(
        acknowledgement.evidence,
        ArtifactChecksumRecord::from_bytes(b"retained native reply")
    );
    assert_eq!(provider.calls, 1);
    let original: AttemptJournalRecord = serde_json::from_slice(
        &fs::read(
            root.with_extension("retained-original")
                .join("attempt-42.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(original.view().pending_mutation, Some(1));
    assert_eq!(original.view().mutations_used, 1);
    assert_eq!(
        fixture.source_layout.restore_references().unwrap(),
        references
    );
}
