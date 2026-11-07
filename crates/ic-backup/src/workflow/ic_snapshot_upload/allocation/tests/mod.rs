//! Explicit allocation qualification and retained original spending/checkpoint evidence.

use super::*;
use crate::model::attempt_journal::AttemptBudgetRecord;
use crate::model::operation_plan::{OperationPlanRecord, PlanBudgetRecord, PlannedOperationRecord};
use crate::tests::support::operation_plan::{declaration, graph, operation_request};
use crate::tests::support::provider::Provider;

use crate::tests::support::{
    execution_workflow::cancel_pending, ready::ready, response_fault::ReceiptFault,
    retained_reply::write_synced,
};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_snapshot_upload::IcSnapshotUploadReply,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, create_execution_workflow, read_execution_progress,
    },
    ports::ic_mutation::IcMutationProviderError,
    tests::support::{
        ic_snapshot_upload::{DESTINATION_ID, unhex, upload_plan},
        temp_dir,
    },
};
use ic_platform::management_canister_types::SnapshotDataKind;
use std::{
    convert::Infallible,
    fs,
    io::{self, Write},
};

fn with_metadata(check: impl FnOnce(&IcSnapshotUploadRequest<'_>)) {
    crate::tests::support::ic_snapshot_upload::with_metadata_upload(
        &ArtifactChecksumRecord::from_bytes(b"retained source tree"),
        check,
    );
}
fn prepare(
    payload: &IcSnapshotUploadRequest<'_>,
) -> (BackupLayoutGuard, ExecutionStageBindingRecord) {
    let plan = upload_plan(payload);
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    let (_, layout, binding) = crate::tests::support::execution_workflow::prepare_stage(
        "ic-backup-allocation-stage",
        &workflow,
        7,
        plan,
    );
    (layout, binding)
}
fn provider() -> Provider<IcMutationProviderError> {
    Provider::new(Ok(unhex("4449444c026c01b6b897890f016d7b0100031500ff")))
}

fn with_allocation_stage(
    check: impl FnOnce(
        &IcSnapshotUploadRequest<'_>,
        &BackupLayoutGuard,
        &ExecutionStageBindingRecord,
        ExecutionStageGuard<'_>,
        Provider<IcMutationProviderError>,
    ),
) {
    with_metadata(|payload| {
        let (layout, binding) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        check(payload, &layout, &binding, stage, provider());
    });
}

fn admit(
    _: &IcSnapshotUploadAttempt<'_, '_>,
) -> impl std::future::Future<Output = Result<(), Infallible>> {
    std::future::ready(Ok(()))
}
fn receipt(
    request: &IcSnapshotUploadAttempt<'_, '_>,
    acknowledgement: &IcMutationAcknowledgement,
) -> MutationReceiptRequest {
    crate::tests::support::update::applied_receipt(
        request.mutation_attempt(),
        &request.payload().binding_digest(),
        &acknowledgement.input().evidence,
    )
}
fn pending(stage: &ExecutionStageGuard<'_>) {
    crate::tests::support::execution_workflow::assert_pending(stage, 7);
    assert!(
        stage
            .reconcile_checkpoint(ArtifactChecksumRecord::from_bytes(b"not qualified"))
            .is_err()
    );
}

#[test]
fn cancelled_allocation_qualification_retains_exact_reply_and_pending_original() {
    with_allocation_stage(|payload, _layout, _binding, stage, mut provider| {
        let root = stage.layout().unwrap().root().to_path_buf();
        let reply = provider.reply.clone().unwrap();
        let future = allocate_snapshot(
            &stage,
            7,
            payload,
            &mut provider,
            admit,
            async |request, acknowledgement| {
                for (name, bytes) in [
                    (
                        "retained-allocation.candid",
                        acknowledgement.input().reply.as_slice(),
                    ),
                    (
                        "retained-allocation-request.candid",
                        request.payload().arguments(),
                    ),
                ] {
                    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(root.join(name))
                        .unwrap();
                    file.write_all(bytes).unwrap();
                    file.sync_all().unwrap();
                }
                fs::File::open(&root).unwrap().sync_all().unwrap();
                assert_eq!(
                    fs::read(root.join("retained-allocation.candid")).unwrap(),
                    acknowledgement.input().reply
                );
                std::future::pending::<()>().await;
                Ok::<_, Infallible>(receipt(request, acknowledgement))
            },
        );
        let journal_path = root.join("attempt-7.json");
        let original = cancel_pending(&stage, 7, future);
        pending(&stage);
        assert_eq!(provider.calls, 1);
        assert!(
            ready(allocate_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                admit,
                async |request, acknowledgement| Ok::<_, Infallible>(receipt(
                    request,
                    acknowledgement
                ))
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 1);
        assert_eq!(fs::read(journal_path).unwrap(), original);
        assert_eq!(
            fs::read(root.join("retained-allocation.candid")).unwrap(),
            reply
        );
        assert_eq!(
            fs::read(root.join("retained-allocation-request.candid")).unwrap(),
            payload.arguments()
        );
    });
}

#[test]
fn qualified_retained_reply_checkpoints_exact_allocation_without_reissue_on_reopen() {
    with_allocation_stage(|payload, layout, binding, stage, mut provider| {
        let (acknowledgement, predecessor) = ready(allocate_snapshot(
            &stage,
            7,
            payload,
            &mut provider,
            admit,
            async |request, acknowledgement| {
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                let _file = write_synced(
                    &stage
                        .layout()
                        .unwrap()
                        .root()
                        .join("retained-allocation.candid"),
                    &acknowledgement.input().reply,
                )
                .unwrap();
                Ok::<_, Infallible>(receipt(request, acknowledgement))
            },
        ))
        .unwrap();
        assert_eq!(
            predecessor.learned_evidence(),
            &IcSnapshotUploadReply::decode(payload, &acknowledgement.input().reply)
                .unwrap()
                .digest()
        );
        assert_eq!(provider.calls, 1);
        drop(stage);
        let (reopened, view) =
            ExecutionStageGuard::resume(layout, binding.workflow(), 7, &binding.digest()).unwrap();
        assert_eq!(view.applied_operations, 1);
        let settlement = crate::ops::persistence::read_execution_settlement(
            reopened.layout().unwrap(),
            &reopened.plan().digest(),
            predecessor.settlement(),
        )
        .unwrap();
        assert_eq!(&settlement.digest(), predecessor.settlement());
        assert!(
            ready(allocate_snapshot(
                &reopened,
                7,
                payload,
                &mut provider,
                admit,
                async |_, _| -> Result<_, Infallible> { panic!("Applied never reissues") }
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 1);
    });
}

#[test]
fn data_sequence_and_multi_operation_stages_refuse_before_spending_or_provider() {
    with_metadata(|payload| {
        let data = IcSnapshotUploadRequest::data(
            payload,
            DESTINATION_ID,
            SnapshotDataKind::WasmModule { offset: 0, size: 3 },
            &[7, 8, 9],
        )
        .unwrap();
        let (layout, binding) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let mut provider = provider();
        for (sequence, request) in [(6, payload), (7, &data)] {
            assert!(matches!(
                ready(allocate_snapshot(
                    &stage,
                    sequence,
                    request,
                    &mut provider,
                    admit,
                    async |_, _| -> Result<_, Infallible> { panic!("not allocation") }
                )),
                Err(IcSnapshotAllocationExecutionError::OriginalMismatch)
            ));
        }
        assert_eq!(
            read_execution_progress(stage.layout().unwrap(), &stage.plan().digest())
                .unwrap()
                .attempts
                .mutations_used,
            0
        );
        let root = temp_dir("ic-backup-multi-allocation");
        let multi_layout = BackupLayoutGuard::acquire(&root).unwrap();
        let mut value = declaration(&upload_plan(payload));
        let mut operation = operation_request(&value.operations[0]);
        operation.operation_sequence = 8;
        value
            .operations
            .push(PlannedOperationRecord::new(operation).unwrap());
        value.graph = graph([(7, vec![]), (8, vec![7])]);
        value.budget = PlanBudgetRecord::new(4, 2).unwrap();
        let plan = OperationPlanRecord::new(value).unwrap();
        let mut allocation = declaration(&upload_plan(payload));
        let mut stage = operation_request(&allocation.operations[0]);
        stage.budget = AttemptBudgetRecord::new(4, 2).unwrap();
        allocation.operations[0] = PlannedOperationRecord::new(stage).unwrap();
        allocation.budget = PlanBudgetRecord::new(4, 2).unwrap();
        let workflow = ExecutionWorkflowRecord::new(OperationPlanRecord::new(allocation).unwrap());
        create_execution_workflow(&multi_layout, &workflow).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![]).unwrap();
        let multi = ExecutionStageGuard::prepare(&multi_layout, binding, plan).unwrap();
        assert!(matches!(
            ready(allocate_snapshot(
                &multi,
                7,
                payload,
                &mut provider,
                admit,
                async |_, _| -> Result<_, Infallible> { panic!("not singleton") }
            )),
            Err(IcSnapshotAllocationExecutionError::OriginalMismatch)
        ));
        assert_eq!(provider.calls, 0);
    });
}

#[derive(Debug, Eq, PartialEq)]
enum AllocationFailure {
    LostReply,
    MalformedReply,
    Unqualified,
}

#[test]
fn lost_malformed_or_unqualified_allocation_cannot_checkpoint_or_reissue() {
    for failure in [
        AllocationFailure::LostReply,
        AllocationFailure::MalformedReply,
        AllocationFailure::Unqualified,
    ] {
        with_allocation_stage(|payload, layout, binding, stage, mut provider| {
            if failure == AllocationFailure::LostReply {
                provider.reply = Err(IcMutationProviderError::Indeterminate);
            }
            if failure == AllocationFailure::MalformedReply {
                provider.reply = Ok(vec![]);
            }
            let result = ready(allocate_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                async |_| Ok::<_, io::Error>(()),
                async |_, _| {
                    assert_eq!(
                        failure,
                        AllocationFailure::Unqualified,
                        "unadmitted replies never reach qualification"
                    );
                    Err(io::Error::other("no authentic durable originals"))
                },
            ));
            if failure == AllocationFailure::Unqualified {
                let Err(IcSnapshotAllocationExecutionError::AfterReply {
                    source: IcSnapshotAllocationSettlementError::Qualification(_),
                    acknowledgement,
                }) = result
                else {
                    panic!("qualification rejection");
                };
                assert_eq!(
                    acknowledgement.input().reply,
                    provider.reply.clone().unwrap()
                );
            } else {
                assert!(matches!(
                    result,
                    Err(IcSnapshotAllocationExecutionError::Upload(_))
                ));
            }
            pending(&stage);
            drop(stage);
            let (reopened, view) =
                ExecutionStageGuard::resume(layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            assert_eq!(view.attempts.mutations_used, 1);
            assert!(
                ready(allocate_snapshot(
                    &reopened,
                    7,
                    payload,
                    &mut provider,
                    admit,
                    async |_, _| -> Result<_, Infallible> { panic!("pending never reissues") }
                ))
                .is_err()
            );
            assert_eq!(provider.calls, 1);
        });
    }
}

#[test]
fn wrong_or_invalid_receipts_retain_reply_and_original_pending_spending() {
    #[derive(Clone, Copy)]
    enum AllocationReceiptFault {
        Receipt(ReceiptFault),
        InvalidEvidence,
    }
    for fault in [
        AllocationReceiptFault::Receipt(ReceiptFault::NotApplied),
        AllocationReceiptFault::Receipt(ReceiptFault::Attempt),
        AllocationReceiptFault::Receipt(ReceiptFault::Request),
        AllocationReceiptFault::InvalidEvidence,
    ] {
        with_allocation_stage(|payload, _layout, _binding, stage, mut provider| {
            let result = ready(allocate_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                admit,
                async |request, acknowledgement| {
                    let mut receipt = receipt(request, acknowledgement);
                    match fault {
                        AllocationReceiptFault::Receipt(fault) => {
                            fault.apply(&mut receipt, &"00".repeat(32));
                        }
                        AllocationReceiptFault::InvalidEvidence => receipt.evidence.clear(),
                    }
                    Ok::<_, Infallible>(receipt)
                },
            ));
            let Err(IcSnapshotAllocationExecutionError::AfterReply {
                source,
                acknowledgement,
            }) = result
            else {
                panic!("receipt rejection retains reply");
            };
            assert_eq!(
                acknowledgement.input().reply,
                provider.reply.clone().unwrap()
            );
            if matches!(fault, AllocationReceiptFault::InvalidEvidence) {
                assert!(matches!(
                    source,
                    IcSnapshotAllocationSettlementError::Journal(_)
                ));
            } else {
                assert!(matches!(
                    source,
                    IcSnapshotAllocationSettlementError::ReceiptRequired
                ));
            }
            pending(&stage);
        });
    }
}

#[test]
fn changed_stage_during_qualification_keeps_original_pending_and_returned_reply() {
    with_allocation_stage(|payload, _layout, _binding, stage, mut provider| {
        let root = stage.layout().unwrap().root().to_path_buf();
        let result = ready(allocate_snapshot(
            &stage,
            7,
            payload,
            &mut provider,
            admit,
            async |request, acknowledgement| {
                fs::write(root.join("stage-binding.json"), b"{}").unwrap();
                Ok::<_, Infallible>(receipt(request, acknowledgement))
            },
        ));
        let Err(IcSnapshotAllocationExecutionError::AfterReply {
            source: IcSnapshotAllocationSettlementError::Stage(_),
            acknowledgement,
        }) = result
        else {
            panic!("stage rejection");
        };
        assert_eq!(
            acknowledgement.input().reply,
            provider.reply.clone().unwrap()
        );
        let journal: crate::model::attempt_journal::AttemptJournalRecord =
            serde_json::from_slice(&fs::read(root.join("attempt-7.json")).unwrap()).unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
    });
}

#[test]
fn occupied_checkpoint_preserves_applied_receipt_reply_and_original_evidence() {
    with_allocation_stage(|payload, layout, binding, stage, mut provider| {
        let path = stage
            .layout()
            .unwrap()
            .root()
            .join("execution-settlement.json");
        let result = ready(allocate_snapshot(
            &stage,
            7,
            payload,
            &mut provider,
            admit,
            async |request, acknowledgement| {
                fs::write(&path, b"occupied original evidence").unwrap();
                Ok::<_, Infallible>(receipt(request, acknowledgement))
            },
        ));
        let Err(IcSnapshotAllocationExecutionError::AfterReply {
            source: IcSnapshotAllocationSettlementError::Checkpoint(_),
            acknowledgement,
        }) = result
        else {
            panic!("checkpoint rejection");
        };
        assert_eq!(
            acknowledgement.input().reply,
            provider.reply.clone().unwrap()
        );
        assert_eq!(fs::read(&path).unwrap(), b"occupied original evidence");
        drop(stage);
        let (reopened, view) =
            ExecutionStageGuard::resume(layout, binding.workflow(), 7, &binding.digest()).unwrap();
        assert_eq!(view.applied_operations, 1);
        assert!(
            ready(allocate_snapshot(
                &reopened,
                7,
                payload,
                &mut provider,
                admit,
                async |_, _| -> Result<_, Infallible> { panic!("Applied never reissues") }
            ))
            .is_err()
        );
        assert_eq!(provider.calls, 1);
        assert_eq!(fs::read(&path).unwrap(), b"occupied original evidence");
    });
}
