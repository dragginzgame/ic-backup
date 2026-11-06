//! Exact original upload spending and one-call custody; no live backend or source proof.

use super::*;
use crate::model::attempt_journal::AttemptBudgetRecord;
use crate::model::operation_plan::{PlanBudgetRecord, PlannedOperationRecord};
use crate::tests::support::operation_plan::{declaration, graph, operation_request};
use crate::tests::support::provider::{Provider, SuspendedProvider};

use crate::tests::support::{ready::ready, response_fault::ContextFault};
use crate::workflow::ic_snapshot_upload::IcSnapshotUploadReplyError;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_lifecycle_reply::EMPTY_CANDID_REPLY,
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{BackupLayoutGuard, create_execution_workflow, read_execution_progress},
    tests::support::{
        ic_snapshot_upload::{DESTINATION_ID, unhex, upload_plan},
        temp_dir,
    },
};
use ic_platform::management_canister_types::SnapshotDataKind;
use std::{convert::Infallible, fs, io, path::PathBuf};

fn with_uploads(mut check: impl FnMut(&IcSnapshotUploadRequest<'_>, Vec<u8>)) {
    crate::tests::support::ic_snapshot_upload::with_metadata_upload(
        &ArtifactChecksumRecord::from_bytes(b"declared source tree"),
        |upload| {
            let data = IcSnapshotUploadRequest::data(
                upload,
                DESTINATION_ID,
                SnapshotDataKind::WasmModule { offset: 0, size: 3 },
                &[7, 8, 9],
            )
            .unwrap();
            check(upload, unhex("4449444c026c01b6b897890f016d7b0100031500ff"));
            check(&data, EMPTY_CANDID_REPLY.to_vec());
        },
    );
}
fn prepare(plan: OperationPlanRecord) -> (PathBuf, BackupLayoutGuard, ExecutionStageBindingRecord) {
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    crate::tests::support::execution_workflow::prepare_stage(
        "ic-backup-upload-step",
        &workflow,
        7,
        plan,
    )
}
fn admit(
    _: &IcSnapshotUploadAttempt<'_, '_>,
) -> impl std::future::Future<Output = Result<(), Infallible>> {
    std::future::ready(Ok(()))
}

#[test]
fn cancellation_retains_metadata_or_data_spending_and_denies_provider_reentry() {
    with_uploads(|payload, _| {
        for suspend_admission in [true, false] {
            let (_, layout, binding) = prepare(upload_plan(payload));
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            let mut provider = SuspendedProvider { calls: 0 };
            let future = dispatch_pending(&stage, 7, payload, &mut provider, async |_| {
                if suspend_admission {
                    std::future::pending::<()>().await;
                }
                Ok::<(), Infallible>(())
            });
            let original =
                crate::tests::support::execution_workflow::cancel_pending(&stage, 7, future);
            assert_eq!(provider.calls, usize::from(!suspend_admission));
            assert!(ready(dispatch_pending(&stage, 7, payload, &mut provider, admit)).is_err());
            assert_eq!(provider.calls, usize::from(!suspend_admission));
            assert_eq!(
                fs::read(stage.layout().unwrap().root().join("attempt-7.json")).unwrap(),
                original
            );
        }
    });
}

#[test]
fn exact_metadata_and_data_are_durably_reserved_locked_and_remain_pending_on_success() {
    with_uploads(|payload, raw| {
        let (_, layout, binding) = prepare(upload_plan(payload));
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let mut provider = Provider::new(Ok(raw.clone()));
        let response = ready(dispatch_pending(
            &stage,
            7,
            payload,
            &mut provider,
            async |request| {
                let record: crate::model::attempt_journal::AttemptJournalRecord =
                    serde_json::from_slice(
                        &fs::read(stage.layout().unwrap().root().join("attempt-7.json")).unwrap(),
                    )
                    .unwrap();
                assert_eq!(
                    record.view().pending_mutation,
                    Some(request.mutation_attempt())
                );
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                assert_eq!(request.payload().binding_digest(), payload.binding_digest());
                assert_eq!(request.payload().receiver(), "aaaaa-aa");
                Ok::<_, Infallible>(())
            },
        ))
        .unwrap();
        assert_eq!(response.input().reply, raw);
        assert_eq!(provider.calls, 1);
        crate::tests::support::execution_workflow::assert_pending(&stage, 7);
        assert!(
            ready(dispatch_pending(
                &stage,
                7,
                payload,
                &mut provider,
                async |_| -> Result<(), Infallible> { panic!("pending is not new custody") }
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 1);
    });
}

#[test]
fn failed_provider_preserves_original_journals_and_references_across_reopen() {
    with_uploads(|payload, raw| {
        for failure in [
            IcMutationProviderError::Unavailable,
            IcMutationProviderError::Unsupported,
            IcMutationProviderError::Indeterminate,
        ] {
            let (root, layout, binding) = prepare(upload_plan(payload));
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            stage
                .layout()
                .unwrap()
                .retain_restore(
                    &root.join("unfinished-reference.json"),
                    stage.plan().digest().hash(),
                )
                .unwrap();
            let references = stage.layout().unwrap().restore_references().unwrap();
            let mut provider = Provider::new(Ok(raw.clone()));
            provider.reply = Err(failure);
            assert!(matches!(
                ready(dispatch_pending(&stage, 7, payload, &mut provider, admit)),
                Err(IcSnapshotUploadExecutionError::Provider(_))
            ));
            crate::tests::support::execution_workflow::assert_pending(&stage, 7);
            let original = fs::read(stage.layout().unwrap().root().join("attempt-7.json")).unwrap();
            drop(stage);
            let (stage, progress) =
                ExecutionStageGuard::resume(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            assert_eq!(progress.attempts.mutations_used, 1);
            assert_eq!(
                stage.layout().unwrap().restore_references().unwrap(),
                references
            );
            assert_eq!(
                fs::read(stage.layout().unwrap().root().join("attempt-7.json")).unwrap(),
                original
            );
            assert!(
                ready(dispatch_pending(
                    &stage,
                    7,
                    payload,
                    &mut provider,
                    async |_| -> Result<(), Infallible> { panic!("never reissue") }
                ))
                .is_err()
            );
            assert_eq!(provider.calls, 1);
        }
    });
}

#[test]
fn malformed_or_mismatched_reply_retains_exact_returned_bytes_and_pending_spending() {
    with_uploads(|payload, raw| {
        for mismatched in [false, true] {
            let (_, layout, binding) = prepare(upload_plan(payload));
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            let expected = if mismatched { raw.clone() } else { vec![] };
            let mut provider = Provider::new(Ok(expected.clone()));
            if mismatched {
                provider.target = Some("aaaaa-aa");
            }
            let Err(IcSnapshotUploadExecutionError::AfterReply {
                source: IcSnapshotUploadReplyError::Association(_),
                acknowledgement,
            }) = ready(dispatch_pending(&stage, 7, payload, &mut provider, admit))
            else {
                panic!("passive association must reject")
            };
            assert_eq!(acknowledgement.input().reply, expected);
            crate::tests::support::execution_workflow::assert_pending(&stage, 7);
            assert_eq!(provider.calls, 1);
        }
    });
}

#[test]
fn fresh_admission_rejection_keeps_consumed_allowance_without_provider_call() {
    with_uploads(|payload, raw| {
        let (_, layout, binding) = prepare(upload_plan(payload));
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let mut provider = Provider::new(Ok(raw));
        assert!(matches!(
            ready(dispatch_pending(
                &stage,
                7,
                payload,
                &mut provider,
                async |_| Err(io::Error::other(
                    "no fresh controller/source/destination custody"
                ))
            )),
            Err(IcSnapshotUploadExecutionError::Admission(_))
        ));
        crate::tests::support::execution_workflow::assert_pending(&stage, 7);
        assert_eq!(provider.calls, 0);
    });
}

#[test]
fn changed_payload_source_network_or_release_reject_before_spending() {
    #[derive(Clone, Copy)]
    enum PlanFault {
        Request,
        Context(ContextFault),
    }
    with_uploads(|payload, raw| {
        for fault in [
            PlanFault::Request,
            PlanFault::Context(ContextFault::Network),
            PlanFault::Context(ContextFault::Release),
        ] {
            let mut plan = declaration(&upload_plan(payload));
            match fault {
                PlanFault::Request => {
                    let mut operation = operation_request(&plan.operations[0]);
                    operation.request = "00".repeat(32);
                    plan.operations[0] = PlannedOperationRecord::new(operation).unwrap();
                }
                PlanFault::Context(context) => context.apply(&mut plan.context, &"00".repeat(32)),
            }
            let (_, layout, binding) = prepare(OperationPlanRecord::new(plan).unwrap());
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            let mut provider = Provider::new(Ok(raw.clone()));
            assert!(matches!(
                ready(dispatch_pending(
                    &stage,
                    7,
                    payload,
                    &mut provider,
                    async |_| -> Result<(), Infallible> { panic!("wrong originals") }
                )),
                Err(IcSnapshotUploadExecutionError::Request(_))
            ));
            assert_eq!(
                read_execution_progress(stage.layout().unwrap(), &stage.plan().digest())
                    .unwrap()
                    .attempts
                    .mutations_used,
                0
            );
            assert_eq!(provider.calls, 0);
        }
    });
}

#[test]
fn held_missing_and_applied_journals_never_grant_dispatch() {
    with_uploads(|payload, raw| {
        let (_, layout, binding) = prepare(upload_plan(payload));
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let mut provider = Provider::new(Ok(raw));
        let mut journal = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &stage.plan().attempt_authority(7).unwrap(),
        )
        .unwrap();
        assert!(ready(dispatch_pending(&stage, 7, payload, &mut provider, admit)).is_err());
        journal
            .reserve_planned_mutation(&stage.plan().digest())
            .unwrap();
        journal
            .record_mutation(MutationReceiptRequest {
                attempt: 1,
                request: payload.binding_digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: ArtifactChecksumRecord::from_bytes(b"independent receipt")
                    .hash()
                    .into(),
            })
            .unwrap();
        let path = journal.path().clone();
        drop(journal);
        assert!(ready(dispatch_pending(&stage, 7, payload, &mut provider, admit)).is_err());
        fs::rename(&path, path.with_extension("retained-original")).unwrap();
        assert!(ready(dispatch_pending(&stage, 7, payload, &mut provider, admit)).is_err());
        assert_eq!(provider.calls, 0);
    });
}

#[test]
fn changed_stage_before_dispatch_or_after_reply_preserves_original_reservation() {
    with_uploads(|payload, raw| {
        for after_reply in [false, true] {
            let (_, layout, binding) = prepare(upload_plan(payload));
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            let path = stage.layout().unwrap().root().join("stage-binding.json");
            let mut provider = Provider::new(Ok(raw.clone()));
            if after_reply {
                provider.change_stage = Some(path.clone());
            }
            let result = ready(dispatch_pending(
                &stage,
                7,
                payload,
                &mut provider,
                async |_| {
                    if !after_reply {
                        fs::write(&path, b"{}").unwrap();
                    }
                    Ok::<_, Infallible>(())
                },
            ));
            if after_reply {
                let Err(IcSnapshotUploadExecutionError::AfterReply {
                    source: IcSnapshotUploadReplyError::Stage(_),
                    acknowledgement,
                }) = result
                else {
                    panic!("retain returned acknowledgement")
                };
                assert_eq!(acknowledgement.input().reply, raw);
                assert_eq!(provider.calls, 1);
            } else {
                assert!(matches!(
                    result,
                    Err(IcSnapshotUploadExecutionError::Stage(_))
                ));
                assert_eq!(provider.calls, 0);
            }
            let record: crate::model::attempt_journal::AttemptJournalRecord =
                serde_json::from_slice(
                    &fs::read(path.parent().unwrap().join("attempt-7.json")).unwrap(),
                )
                .unwrap();
            assert_eq!(record.view().pending_mutation, Some(1));
        }
    });
}

#[test]
fn dependent_upload_requires_retained_applied_prerequisite_not_pending_acknowledgement() {
    with_uploads(|payload, raw| {
        let mut child = declaration(&upload_plan(payload));
        let mut allocation = child.clone();
        let mut stage = operation_request(&allocation.operations[0]);
        stage.budget = AttemptBudgetRecord::new(2, stage.budget.observations()).unwrap();
        allocation.operations[0] = PlannedOperationRecord::new(stage).unwrap();
        allocation.budget = PlanBudgetRecord::new(2, allocation.budget.observations()).unwrap();
        let workflow = ExecutionWorkflowRecord::new(OperationPlanRecord::new(allocation).unwrap());
        child.budget = PlanBudgetRecord::new(2, child.budget.observations()).unwrap();
        child.graph = graph([(7, vec![]), (8, vec![7])]);
        let mut second = operation_request(&child.operations[0]);
        second.operation_sequence = 8;
        second.budget = AttemptBudgetRecord::new(second.budget.mutations(), 0).unwrap();
        child
            .operations
            .push(PlannedOperationRecord::new(second).unwrap());
        let plan = OperationPlanRecord::new(child).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![]).unwrap();
        let root = temp_dir("ic-backup-upload-prerequisite");
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        create_execution_workflow(&layout, &workflow).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
        let mut provider = Provider::new(Ok(raw));
        assert!(ready(dispatch_pending(&stage, 8, payload, &mut provider, admit)).is_err());
        ready(dispatch_pending(&stage, 7, payload, &mut provider, admit)).unwrap();
        assert!(ready(dispatch_pending(&stage, 8, payload, &mut provider, admit)).is_err());
        assert_eq!(provider.calls, 1);
        let second = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &stage.plan().attempt_authority(8).unwrap(),
        )
        .unwrap();
        assert_eq!(second.record().unwrap().view().mutations_used, 0);
        drop(second);
        let mut first = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &stage.plan().attempt_authority(7).unwrap(),
        )
        .unwrap();
        first
            .record_mutation(MutationReceiptRequest {
                attempt: 1,
                request: payload.binding_digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: ArtifactChecksumRecord::from_bytes(b"explicit independent attribution")
                    .hash()
                    .into(),
            })
            .unwrap();
        drop(first);
        ready(dispatch_pending(&stage, 8, payload, &mut provider, admit)).unwrap();
        assert_eq!(provider.calls, 2);
    });
}

// Exercise shared dispatch without a receipt; production stage coordinators supply qualification.
async fn dispatch_pending<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: &IcSnapshotUploadRequest<'_>,
    provider: &mut impl IcSnapshotUploadProvider,
    admit: impl AsyncFnOnce(&IcSnapshotUploadAttempt<'_, '_>) -> Result<(), E>,
) -> Result<IcMutationAcknowledgement, IcSnapshotUploadExecutionError<E>> {
    let (acknowledgement, ()) = upload_snapshot_with(
        stage,
        operation_sequence,
        payload,
        provider,
        admit,
        async |_, _, _| (),
    )
    .await?;
    Ok(acknowledgement)
}
