//! Explicit allocation qualification and retained original spending/checkpoint evidence.

use super::*;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_mutation::IcMutationAcknowledgementInput,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_upload::IcSnapshotUploadReply,
    },
    ops::persistence::{BackupLayoutGuard, create_execution_workflow, read_execution_progress},
    ports::ic_mutation::IcMutationProviderError,
    test_support::{
        ic_snapshot_upload::{
            DESTINATION_ID, SOURCE_ID, TARGET, raw, source_plan, unhex, upload_plan,
        },
        temp_dir,
    },
};
use ic_management_canister_types::SnapshotDataKind;
use std::{
    convert::Infallible,
    fs,
    io::{self, Write},
};

fn with_metadata(check: impl FnOnce(&IcSnapshotUploadRequest<'_>)) {
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let raw = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let payload = IcSnapshotUploadRequest::metadata(
        &plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"retained source tree"),
    )
    .unwrap();
    check(&payload);
}
fn prepare(
    payload: &IcSnapshotUploadRequest<'_>,
) -> (BackupLayoutGuard, ExecutionStageBindingRecord) {
    let root = temp_dir("ic-backup-allocation-stage");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = upload_plan(payload);
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    create_execution_workflow(&layout, &workflow).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![]).unwrap();
    drop(ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap());
    (layout, binding)
}
struct Provider {
    calls: usize,
    reply: Result<Vec<u8>, IcMutationProviderError>,
}
impl Provider {
    fn new() -> Self {
        Self {
            calls: 0,
            reply: Ok(unhex("4449444c026c01b6b897890f016d7b0100031500ff")),
        }
    }
}
impl IcSnapshotUploadProvider for Provider {
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
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
#[expect(
    clippy::unnecessary_wraps,
    reason = "mandatory fallible integration admission"
)]
fn admit(_: &IcSnapshotUploadAttempt<'_, '_>) -> Result<(), Infallible> {
    Ok(())
}
fn receipt(
    request: &IcSnapshotUploadAttempt<'_, '_>,
    acknowledgement: &IcMutationAcknowledgement,
) -> MutationReceiptRequest {
    MutationReceiptRequest {
        attempt: request.mutation_attempt(),
        request: request.payload().binding_digest().hash().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: acknowledgement.input().evidence.hash().into(),
    }
}
fn pending(stage: &ExecutionStageGuard<'_>) {
    let progress =
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
    assert_eq!(progress.attempts.mutations_used, 1);
    assert_eq!(progress.applied_operations, 0);
    assert!(
        stage
            .checkpoint(ArtifactChecksumRecord::from_bytes(b"not qualified"))
            .is_err()
    );
}

#[test]
fn qualified_retained_reply_checkpoints_exact_allocation_without_reissue_on_reopen() {
    with_metadata(|payload| {
        let (layout, binding) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let mut provider = Provider::new();
        let (acknowledgement, predecessor) = allocate_snapshot(
            &stage,
            7,
            payload,
            &mut provider,
            admit,
            |request, acknowledgement| {
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                let file = fs::File::create(
                    stage
                        .layout()
                        .unwrap()
                        .root()
                        .join("retained-allocation.candid"),
                )
                .unwrap();
                (&file).write_all(&acknowledgement.input().reply).unwrap();
                file.sync_all().unwrap();
                fs::File::open(stage.layout().unwrap().root())
                    .unwrap()
                    .sync_all()
                    .unwrap();
                Ok::<_, Infallible>(receipt(request, acknowledgement))
            },
        )
        .unwrap();
        assert_eq!(
            predecessor.learned_evidence(),
            &IcSnapshotUploadReply::decode(payload, &acknowledgement.input().reply)
                .unwrap()
                .digest()
        );
        assert_eq!(provider.calls, 1);
        drop(stage);
        let (stage, view) =
            ExecutionStageGuard::resume(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        assert_eq!(view.applied_operations, 1);
        let settlement = crate::ops::persistence::read_execution_settlement(
            stage.layout().unwrap(),
            &stage.plan().digest(),
            predecessor.settlement(),
        )
        .unwrap();
        assert_eq!(&settlement.digest(), predecessor.settlement());
        assert!(
            allocate_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                admit,
                |_, _| -> Result<_, Infallible> { panic!("Applied never reissues") }
            )
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
        let mut provider = Provider::new();
        for (sequence, request) in [(6, payload), (7, &data)] {
            assert!(matches!(
                allocate_snapshot(
                    &stage,
                    sequence,
                    request,
                    &mut provider,
                    admit,
                    |_, _| -> Result<_, Infallible> { panic!("not allocation") }
                ),
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
        fs::create_dir(&root).unwrap();
        let multi_layout = BackupLayoutGuard::acquire(&root).unwrap();
        let mut value = serde_json::to_value(upload_plan(payload)).unwrap();
        let mut op = value["operations"][0].clone();
        op["operation_sequence"] = serde_json::json!(8);
        value["operations"].as_array_mut().unwrap().push(op);
        value["graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"operation_sequence":8,"depends_on":[7]}));
        value["budget"] = serde_json::json!({"mutations":4,"observations":2});
        let plan = serde_json::from_value(value).unwrap();
        let mut allocation = serde_json::to_value(upload_plan(payload)).unwrap();
        allocation["operations"][0]["budget"] = serde_json::json!({"mutations":4,"observations":2});
        allocation["budget"] = serde_json::json!({"mutations":4,"observations":2});
        let workflow = ExecutionWorkflowRecord::new(serde_json::from_value(allocation).unwrap());
        create_execution_workflow(&multi_layout, &workflow).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![]).unwrap();
        let multi = ExecutionStageGuard::prepare(&multi_layout, binding, plan).unwrap();
        assert!(matches!(
            allocate_snapshot(
                &multi,
                7,
                payload,
                &mut provider,
                admit,
                |_, _| -> Result<_, Infallible> { panic!("not singleton") }
            ),
            Err(IcSnapshotAllocationExecutionError::OriginalMismatch)
        ));
        assert_eq!(provider.calls, 0);
    });
}

#[test]
fn lost_malformed_or_unqualified_allocation_cannot_checkpoint_or_reissue() {
    with_metadata(|payload| {
        for failure in 0..3 {
            let (layout, binding) = prepare(payload);
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            let mut provider = Provider::new();
            if failure == 0 {
                provider.reply = Err(IcMutationProviderError::Indeterminate);
            }
            if failure == 1 {
                provider.reply = Ok(vec![]);
            }
            let result = allocate_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                |_| Ok::<_, io::Error>(()),
                |_, _| {
                    assert_eq!(failure, 2, "unadmitted replies never reach qualification");
                    Err(io::Error::other("no authentic durable originals"))
                },
            );
            if failure == 2 {
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
            let (stage, view) =
                ExecutionStageGuard::resume(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            assert_eq!(view.attempts.mutations_used, 1);
            assert!(
                allocate_snapshot(
                    &stage,
                    7,
                    payload,
                    &mut provider,
                    admit,
                    |_, _| -> Result<_, Infallible> { panic!("pending never reissues") }
                )
                .is_err()
            );
            assert_eq!(provider.calls, 1);
        }
    });
}

#[test]
fn wrong_or_invalid_receipts_retain_reply_and_original_pending_spending() {
    with_metadata(|payload| {
        for field in 0..4 {
            let (layout, binding) = prepare(payload);
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            let mut provider = Provider::new();
            let result = allocate_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                admit,
                |request, acknowledgement| {
                    let mut receipt = receipt(request, acknowledgement);
                    match field {
                        0 => receipt.outcome = MutationOutcomeRecord::NotApplied,
                        1 => receipt.attempt += 1,
                        2 => receipt.request = "00".repeat(32),
                        _ => receipt.evidence.clear(),
                    }
                    Ok::<_, Infallible>(receipt)
                },
            );
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
            if field == 3 {
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
        }
    });
}

#[test]
fn changed_stage_during_qualification_keeps_original_pending_and_returned_reply() {
    with_metadata(|payload| {
        let (layout, binding) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let root = stage.layout().unwrap().root().to_path_buf();
        let mut provider = Provider::new();
        let result = allocate_snapshot(
            &stage,
            7,
            payload,
            &mut provider,
            admit,
            |request, acknowledgement| {
                fs::write(root.join("stage-binding.json"), b"{}").unwrap();
                Ok::<_, Infallible>(receipt(request, acknowledgement))
            },
        );
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
    with_metadata(|payload| {
        let (layout, binding) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let path = stage
            .layout()
            .unwrap()
            .root()
            .join("execution-settlement.json");
        let mut provider = Provider::new();
        let result = allocate_snapshot(
            &stage,
            7,
            payload,
            &mut provider,
            admit,
            |request, acknowledgement| {
                fs::write(&path, b"occupied original evidence").unwrap();
                Ok::<_, Infallible>(receipt(request, acknowledgement))
            },
        );
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
        let (stage, view) =
            ExecutionStageGuard::resume(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        assert_eq!(view.applied_operations, 1);
        assert!(
            allocate_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                admit,
                |_, _| -> Result<_, Infallible> { panic!("Applied never reissues") }
            )
            .is_err()
        );
        assert_eq!(provider.calls, 1);
        assert_eq!(fs::read(&path).unwrap(), b"occupied original evidence");
    });
}
