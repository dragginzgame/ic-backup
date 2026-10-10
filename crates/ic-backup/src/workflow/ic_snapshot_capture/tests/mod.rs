//! Exact spending, locked admission, bounded evidence and no recapture on failure.

use super::*;
use crate::test_support::ready::ready;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptJournalRecord, MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{
            ExecutionStageBindingRecord,
            tests::{child_request, workflow},
        },
        ic_mutation::IcMutationAcknowledgementInput,
        ic_request::IcManagementRequest,
        operation_plan::{OperationPlanRecord, PlannedOperationRecord, PlannedOperationRequest},
    },
    ops::persistence::{BackupLayoutGuard, create_execution_workflow},
    test_support::{ic_mutation::reply, temp_dir},
};
use std::{convert::Infallible, fs, path::PathBuf};

struct SuspendedProvider {
    calls: usize,
}
impl IcMutationProvider for SuspendedProvider {
    async fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        request.validate_journal(journal).unwrap();
        self.calls += 1;
        std::future::pending().await
    }
}

#[test]
fn cancellation_during_admission_or_submission_releases_lock_without_reset_or_reissue() {
    use std::task::{Context, Poll, Waker};
    for suspend_admission in [true, false] {
        let payload = payload(IcManagementMethodRecord::TakeCanisterSnapshot);
        let (_root, layout, binding, plan) = prepare(&payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let mut provider = SuspendedProvider { calls: 0 };
        let mut future = Box::pin(capture_snapshot(
            &stage,
            42,
            &payload,
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
                &plan.attempt_authority(42).unwrap()
            )
            .is_err()
        );
        let before = fs::read(stage.layout().unwrap().root().join("attempt-42.json")).unwrap();
        drop(future);
        assert_eq!(provider.calls, usize::from(!suspend_admission));
        assert_pending(&stage, &plan);
        assert!(ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)).is_err());
        assert_eq!(provider.calls, usize::from(!suspend_admission));
        assert_eq!(
            fs::read(stage.layout().unwrap().root().join("attempt-42.json")).unwrap(),
            before
        );
    }
}

fn payload(method: IcManagementMethodRecord) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: workflow().stage(0).unwrap().target().into(),
        snapshot_id: (method == IcManagementMethodRecord::LoadCanisterSnapshot)
            .then(|| vec![0, 255, 128]),
    })
    .unwrap()
}
fn prepare(
    payload: &IcManagementRequestRecord,
) -> (
    PathBuf,
    BackupLayoutGuard,
    ExecutionStageBindingRecord,
    OperationPlanRecord,
) {
    let root = temp_dir("ic-backup-capture-step");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let workflow = workflow();
    create_execution_workflow(&layout, &workflow).unwrap();
    let mut child = child_request(0);
    child.operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 42,
            target: payload.target().into(),
            request: payload.digest().hash().into(),
            budget: workflow.stage(0).unwrap().budget().clone(),
        })
        .unwrap(),
    ];
    let plan = OperationPlanRecord::new(child).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan.clone()).unwrap();
    drop(stage);
    (root, layout, binding, plan)
}
fn admit(_: &IcMutationRequest<'_>) -> impl std::future::Future<Output = Result<(), Infallible>> {
    std::future::ready(Ok(()))
}
struct Provider {
    calls: usize,
    reply: Result<Vec<u8>, IcMutationProviderError>,
    change_stage: Option<PathBuf>,
    replace_layout: Option<PathBuf>,
    target: Option<&'static str>,
}
impl Provider {
    fn new(reply: Result<Vec<u8>, IcMutationProviderError>) -> Self {
        Self {
            calls: 0,
            reply,
            change_stage: None,
            replace_layout: None,
            target: None,
        }
    }
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
            assert_eq!(
                request.payload().method(),
                IcManagementMethodRecord::TakeCanisterSnapshot
            );
            if let Some(path) = &self.change_stage {
                fs::write(path, b"{}").unwrap();
            }
            if let Some(path) = &self.replace_layout {
                fs::rename(path, path.with_extension("retained-original")).unwrap();
                fs::create_dir(path).unwrap();
            }
            IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                context: request.plan().context().clone(),
                target: self.target.unwrap_or(request.payload().target()).into(),
                reply: self.reply.clone()?,
                evidence: ArtifactChecksumRecord::from_bytes(b"passive fixture association"),
            })
            .map_err(|_| IcMutationProviderError::Indeterminate)
        })())
    }
}
fn assert_pending(stage: &ExecutionStageGuard<'_>, plan: &OperationPlanRecord) {
    let journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &plan.attempt_authority(42).unwrap(),
    )
    .unwrap();
    let view = journal.record().unwrap().view();
    assert_eq!(view.pending_mutation, Some(1));
    assert_eq!(view.mutations_used, 1);
    assert!(!view.applied);
}

#[test]
fn one_durably_reserved_capture_holds_exclusion_and_leaves_receipt_qualification_explicit() {
    let payload = payload(IcManagementMethodRecord::TakeCanisterSnapshot);
    let raw = reply(payload.method());
    let (root, layout, binding, plan) = prepare(&payload);
    let stage =
        ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new(Ok(raw.clone()));
    let acknowledgement = ready(capture_snapshot(
        &stage,
        42,
        &payload,
        &mut provider,
        async |request| {
            let retained: AttemptJournalRecord = serde_json::from_slice(
                &fs::read(stage.layout().unwrap().root().join("attempt-42.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(
                retained.view().pending_mutation,
                Some(request.mutation_attempt())
            );
            assert_eq!(request.payload().digest(), payload.digest());
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            Ok::<(), Infallible>(())
        },
    ))
    .unwrap();
    assert_eq!(acknowledgement.input().reply, raw);
    assert_pending(&stage, &plan);
    assert!(
        ready(capture_snapshot(
            &stage,
            42,
            &payload,
            &mut provider,
            async |_| -> Result<(), Infallible> {
                panic!("pending original cannot re-enter admission")
            }
        ))
        .is_err()
    );
    let mut journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &plan.attempt_authority(42).unwrap(),
    )
    .unwrap();
    let request = IcMutationRequest::new(&plan, 42, journal.record().unwrap(), &payload).unwrap();
    let view =
        validate_acknowledgement(&request, journal.record().unwrap(), &acknowledgement).unwrap();
    let crate::policy::ic_mutation::IcMutationReplyView::Capture(reply) = view.reply() else {
        panic!("capture wire")
    };
    assert_eq!(reply.snapshots()[0].id(), &[0, 255, 128]);
    // Explicit fixture receipt, never an outcome inferred by the coordinator.
    journal
        .record_mutation(MutationReceiptRequest {
            attempt: 1,
            request: payload.digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: acknowledgement.input().evidence.hash().into(),
        })
        .unwrap();
    drop(journal);
    assert!(ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)).is_err());
    assert_eq!(provider.calls, 1);
    stage
        .checkpoint(ArtifactChecksumRecord::from_bytes(&raw))
        .unwrap();
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn every_other_management_method_rejects_before_spending_callback_or_provider() {
    for method in [
        IcManagementMethodRecord::StopCanister,
        IcManagementMethodRecord::StartCanister,
        IcManagementMethodRecord::LoadCanisterSnapshot,
        IcManagementMethodRecord::CanisterStatus,
        IcManagementMethodRecord::ListCanisterSnapshots,
    ] {
        let payload = payload(method);
        let (root, layout, binding, _) = prepare(&payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let path = stage.layout().unwrap().root().join("attempt-42.json");
        let original = fs::read(&path).unwrap();
        let mut provider = Provider::new(Ok(reply(method)));
        assert!(matches!(
            ready(capture_snapshot(
                &stage,
                42,
                &payload,
                &mut provider,
                async |_| -> Result<(), Infallible> {
                    panic!("wrong method never reaches admission")
                }
            )),
            Err(IcSnapshotCaptureExecutionError::CaptureRequired)
        ));
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(provider.calls, 0);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn changed_payload_unknown_operation_missing_and_held_originals_refuse_without_consumption() {
    let payload = payload(IcManagementMethodRecord::TakeCanisterSnapshot);
    let (root, layout, binding, plan) = prepare(&payload);
    let stage =
        ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
    let path = stage.layout().unwrap().root().join("attempt-42.json");
    let original = fs::read(&path).unwrap();
    let mut provider = Provider::new(Ok(reply(payload.method())));
    let other = IcManagementRequestRecord::new(IcManagementRequest {
        method: payload.method(),
        target: "aaaaa-aa".into(),
        snapshot_id: None,
    })
    .unwrap();
    assert!(matches!(
        ready(capture_snapshot(&stage, 42, &other, &mut provider, admit)),
        Err(IcSnapshotCaptureExecutionError::Request(_))
    ));
    assert!(matches!(
        ready(capture_snapshot(&stage, 99, &payload, &mut provider, admit)),
        Err(IcSnapshotCaptureExecutionError::Plan(_))
    ));
    let held = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &plan.attempt_authority(42).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)),
        Err(IcSnapshotCaptureExecutionError::Journal(_))
    ));
    drop(held);
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(&path).unwrap();
    assert!(matches!(
        ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)),
        Err(IcSnapshotCaptureExecutionError::Journal(_))
    ));
    assert!(!path.exists());
    assert_eq!(provider.calls, 0);
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fresh_consistency_rejection_consumes_original_reservation_without_dispatch_or_recapture() {
    let payload = payload(IcManagementMethodRecord::TakeCanisterSnapshot);
    let (root, layout, binding, plan) = prepare(&payload);
    let stage =
        ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new(Ok(reply(payload.method())));
    assert!(matches!(
        ready(capture_snapshot(
            &stage,
            42,
            &payload,
            &mut provider,
            async |_| Err(std::io::Error::other("fixture drain refused"))
        )),
        Err(IcSnapshotCaptureExecutionError::Admission(_))
    ));
    assert_pending(&stage, &plan);
    assert!(ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)).is_err());
    assert_eq!(provider.calls, 0);
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn provider_failures_retain_original_pending_spending_and_references_across_resume() {
    for failure in [
        IcMutationProviderError::Unavailable,
        IcMutationProviderError::Unsupported,
        IcMutationProviderError::Indeterminate,
    ] {
        let payload = payload(IcManagementMethodRecord::TakeCanisterSnapshot);
        let (root, layout, binding, plan) = prepare(&payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let stage_layout = stage.layout().unwrap();
        stage_layout
            .retain_restore(&root.join("unfinished-restore.json"), plan.digest().hash())
            .unwrap();
        let references = stage_layout.restore_references().unwrap();
        let mut provider = Provider::new(Err(failure));
        assert!(
            matches!(ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)), Err(IcSnapshotCaptureExecutionError::Provider(actual)) if actual == failure)
        );
        assert_pending(&stage, &plan);
        let path = stage_layout.root().join("attempt-42.json");
        let original = fs::read(&path).unwrap();
        drop(stage);
        let (stage, progress) =
            ExecutionStageGuard::resume(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        assert_eq!(progress.attempts.mutations_used, 1);
        assert!(ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(
            stage.layout().unwrap().restore_references().unwrap(),
            references
        );
        assert_eq!(provider.calls, 1);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn malformed_and_wrong_target_acknowledgements_retain_original_bytes_without_receipts() {
    for malformed in [true, false] {
        let payload = payload(IcManagementMethodRecord::TakeCanisterSnapshot);
        let (root, layout, binding, plan) = prepare(&payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let raw = if malformed {
            b"bounded malformed reply".to_vec()
        } else {
            reply(payload.method())
        };
        let mut provider = Provider::new(Ok(raw.clone()));
        if !malformed {
            provider.target = Some("aaaaa-aa");
        }
        let error =
            ready(capture_snapshot(&stage, 42, &payload, &mut provider, admit)).unwrap_err();
        let IcSnapshotCaptureExecutionError::Association {
            acknowledgement, ..
        } = error
        else {
            panic!("passive rejection retains reply")
        };
        assert_eq!(acknowledgement.input().reply, raw);
        assert_eq!(provider.calls, 1);
        assert_pending(&stage, &plan);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn drift_at_admission_stops_dispatch_and_post_reply_stage_or_layout_drift_retains_acknowledgement()
{
    for boundary in 0..3 {
        let payload = payload(IcManagementMethodRecord::TakeCanisterSnapshot);
        let (root, layout, binding, _) = prepare(&payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let stage_root = stage.layout().unwrap().root().to_path_buf();
        let path = stage_root.join("stage-binding.json");
        let raw = reply(payload.method());
        let mut provider = Provider::new(Ok(raw.clone()));
        match boundary {
            1 => provider.change_stage = Some(path.clone()),
            2 => provider.replace_layout = Some(stage_root.clone()),
            _ => {}
        }
        let error = ready(capture_snapshot(
            &stage,
            42,
            &payload,
            &mut provider,
            async |_| {
                if boundary == 0 {
                    fs::write(&path, b"{}").unwrap();
                }
                Ok::<(), Infallible>(())
            },
        ))
        .unwrap_err();
        match error {
            IcSnapshotCaptureExecutionError::Stage(_) if boundary == 0 => {}
            IcSnapshotCaptureExecutionError::AfterReplyStage {
                acknowledgement, ..
            } if boundary == 1 => assert_eq!(acknowledgement.input().reply, raw),
            IcSnapshotCaptureExecutionError::AfterReplyJournal {
                acknowledgement, ..
            } if boundary == 2 => assert_eq!(acknowledgement.input().reply, raw),
            other => panic!("unexpected boundary failure: {other:?}"),
        }
        assert_eq!(provider.calls, usize::from(boundary != 0));
        let retained = if boundary == 2 {
            stage_root.with_extension("retained-original")
        } else {
            stage_root
        };
        let journal: AttemptJournalRecord =
            serde_json::from_slice(&fs::read(retained.join("attempt-42.json")).unwrap()).unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert_eq!(journal.view().mutations_used, 1);
        assert!(!journal.view().applied);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
