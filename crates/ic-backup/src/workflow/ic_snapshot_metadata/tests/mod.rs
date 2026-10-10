//! Original spending, explicit qualification and exact learned-stage admission.

use super::*;
use crate::test_support::ready::ready;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_snapshot_download::{
            IcSnapshotDownloadPlan,
            tests::{source, source_plan, values, workflow},
        },
        ic_snapshot_metadata::IcSnapshotMetadataReply,
        ic_snapshot_transfer_read::IcSnapshotTransferReadResponseInput,
    },
    ops::persistence::{BackupLayoutGuard, create_execution_workflow, read_execution_progress},
    ports::ic_observation::IcObservationProviderError,
    test_support::temp_dir,
};
use std::{
    convert::Infallible,
    fs,
    io::{self, Write},
    path::PathBuf,
};

fn prepare(
    workflow: &ExecutionWorkflowRecord,
    payload: &IcSnapshotMetadataRequest,
) -> (PathBuf, BackupLayoutGuard, ExecutionStageBindingRecord) {
    let root = temp_dir("ic-backup-metadata-stage");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, workflow).unwrap();
    let plan = source_plan(payload);
    let binding = ExecutionStageBindingRecord::new(workflow, 0, &plan, vec![]).unwrap();
    drop(ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap());
    (root, layout, binding)
}
struct Provider {
    calls: usize,
    reply: Result<Vec<u8>, IcObservationProviderError>,
}
impl Provider {
    fn new() -> Self {
        Self {
            calls: 0,
            reply: Ok(candid::encode_one(values()).unwrap()),
        }
    }
}
impl IcSnapshotTransferReadProvider for Provider {
    fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &crate::model::attempt_journal::AttemptJournalRecord,
    ) -> impl std::future::Future<
        Output = Result<IcSnapshotTransferReadResponse, IcObservationProviderError>,
    > {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            self.calls += 1;
            let raw = self.reply.clone()?;
            Ok(
                IcSnapshotTransferReadResponse::new(IcSnapshotTransferReadResponseInput {
                    authority: request.authority().digest(),
                    mutation_attempt: request.mutation_attempt(),
                    context: request.plan().context().clone(),
                    target: request.payload().target().into(),
                    evidence: ArtifactChecksumRecord::from_bytes(&raw),
                    reply: raw,
                })
                .unwrap(),
            )
        })())
    }
}
fn admit(
    _: &IcSnapshotTransferReadRequest<'_, '_>,
) -> impl std::future::Future<Output = Result<(), Infallible>> {
    std::future::ready(Ok(()))
}
fn receipt(
    request: &IcSnapshotTransferReadRequest<'_, '_>,
    response: &IcSnapshotTransferReadResponse,
) -> MutationReceiptRequest {
    MutationReceiptRequest {
        attempt: request.mutation_attempt(),
        request: request.payload().digest().hash().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: response.input().evidence.hash().into(),
    }
}
fn pending(stage: &ExecutionStageGuard<'_>) {
    let progress =
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
    assert_eq!(progress.attempts.mutations_used, 1);
    assert_eq!(progress.applied_operations, 0);
    let guard = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(42).unwrap(),
    )
    .unwrap();
    assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
}

#[test]
fn cancelled_qualification_retains_reply_and_original_pending_spending() {
    use std::task::{Context, Poll, Waker};
    let workflow = workflow(12);
    let payload = source();
    let (root, layout, binding) = prepare(&workflow, &payload);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new();
    let retained = root.join("retained-metadata.candid");
    let mut future = Box::pin(read_snapshot_metadata(
        &stage,
        42,
        &payload,
        &mut provider,
        admit,
        async |_, response| {
            let file = fs::File::create(&retained).unwrap();
            (&file).write_all(&response.input().reply).unwrap();
            file.sync_all().unwrap();
            fs::File::open(&root).unwrap().sync_all().unwrap();
            std::future::pending::<Result<MutationReceiptRequest, Infallible>>().await
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
            &stage.plan().attempt_authority(42).unwrap()
        )
        .is_err()
    );
    let path = stage.layout().unwrap().root().join("attempt-42.json");
    let before = fs::read(&path).unwrap();
    drop(future);
    pending(&stage);
    assert_eq!(provider.calls, 1);
    assert_eq!(
        fs::read(&retained).unwrap(),
        candid::encode_one(values()).unwrap()
    );
    assert!(
        ready(read_snapshot_metadata(
            &stage,
            42,
            &payload,
            &mut provider,
            admit,
            async |_, _| -> Result<_, Infallible> { panic!("no requalification") },
        ))
        .is_err()
    );
    assert_eq!(provider.calls, 1);
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn explicit_receipt_checkpoints_exact_metadata_and_admits_original_data_stage() {
    let workflow = workflow(12);
    let payload = source();
    let (root, layout, binding) = prepare(&workflow, &payload);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new();
    let (response, predecessor) = ready(read_snapshot_metadata(
        &stage,
        42,
        &payload,
        &mut provider,
        admit,
        async |request, response| {
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            // Simulate integration-owned durable original reply retention before receipt.
            let file = fs::File::create(root.join("retained-metadata.candid")).unwrap();
            (&file).write_all(&response.input().reply).unwrap();
            file.sync_all().unwrap();
            Ok::<_, Infallible>(receipt(request, response))
        },
    ))
    .unwrap();
    assert_eq!(provider.calls, 1);
    let metadata = IcSnapshotMetadataReply::decode(&payload, &response.input().reply).unwrap();
    assert_eq!(predecessor.learned_evidence(), &metadata.digest());
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 32).unwrap();
    let bound = download
        .bind(&binding, stage.plan(), vec![predecessor])
        .unwrap();
    drop(stage);
    let data_stage =
        ExecutionStageGuard::prepare(&layout, bound, download.plan().unwrap().clone()).unwrap();
    assert_eq!(
        read_execution_progress(data_stage.layout().unwrap(), &data_stage.plan().digest())
            .unwrap()
            .attempts
            .mutations_used,
        0
    );
    drop(data_stage);
    let (reopened, progress) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    assert_eq!(progress.applied_operations, 1);
    assert!(
        ready(read_snapshot_metadata(
            &reopened,
            42,
            &payload,
            &mut provider,
            admit,
            async |_, _| -> Result<_, Infallible> { panic!("no reissue") }
        ))
        .is_err()
    );
    assert_eq!(provider.calls, 1);
}

#[test]
fn lost_and_malformed_metadata_stop_pending_without_checkpoint_or_data_stage() {
    for raw in [Err(IcObservationProviderError::Indeterminate), Ok(vec![])] {
        let workflow = workflow(12);
        let payload = source();
        let (_, layout, binding) = prepare(&workflow, &payload);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
        let mut provider = Provider {
            calls: 0,
            reply: raw,
        };
        assert!(
            ready(read_snapshot_metadata(
                &stage,
                42,
                &payload,
                &mut provider,
                admit,
                async |_, _| -> Result<_, Infallible> { panic!("unadmitted reply") }
            ))
            .is_err()
        );
        pending(&stage);
        assert_eq!(provider.calls, 1);
        assert!(!layout.root().join("execution-stage-7").exists());
        assert!(
            stage
                .checkpoint(ArtifactChecksumRecord::from_bytes(b"no outcome"))
                .is_err()
        );
    }
}

#[test]
fn qualification_rejection_retains_returned_bytes_and_pending_original() {
    let workflow = workflow(12);
    let payload = source();
    let (_, layout, binding) = prepare(&workflow, &payload);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new();
    let expected = provider.reply.clone().unwrap();
    let Err(IcSnapshotMetadataExecutionError::AfterReply {
        source: IcSnapshotMetadataSettlementError::Qualification(_),
        response,
    }) = ready(read_snapshot_metadata(
        &stage,
        42,
        &payload,
        &mut provider,
        async |_| Ok::<_, io::Error>(()),
        async |_, _| Err(io::Error::other("no durable authenticated originals")),
    ))
    else {
        panic!("qualification must reject")
    };
    assert_eq!(response.input().reply, expected);
    pending(&stage);
}

#[test]
fn non_applied_wrong_attempt_and_wrong_request_receipts_cannot_checkpoint() {
    for field in 0..3 {
        let workflow = workflow(12);
        let payload = source();
        let (_, layout, binding) = prepare(&workflow, &payload);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
        let mut provider = Provider::new();
        let result = ready(read_snapshot_metadata(
            &stage,
            42,
            &payload,
            &mut provider,
            admit,
            async |request, response| {
                let mut receipt = receipt(request, response);
                match field {
                    0 => receipt.outcome = MutationOutcomeRecord::NotApplied,
                    1 => receipt.attempt += 1,
                    _ => receipt.request = "00".repeat(32),
                }
                Ok::<_, Infallible>(receipt)
            },
        ));
        assert!(matches!(
            result,
            Err(IcSnapshotMetadataExecutionError::AfterReply {
                source: IcSnapshotMetadataSettlementError::ReceiptRequired,
                ..
            })
        ));
        pending(&stage);
    }
}

#[test]
fn occupied_checkpoint_retains_applied_receipt_and_reply_without_reissue() {
    let workflow = workflow(12);
    let payload = source();
    let (_, layout, binding) = prepare(&workflow, &payload);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new();
    let result = ready(read_snapshot_metadata(
        &stage,
        42,
        &payload,
        &mut provider,
        admit,
        async |request, response| {
            fs::write(
                stage
                    .layout()
                    .unwrap()
                    .root()
                    .join("execution-settlement.json"),
                b"occupied original evidence",
            )
            .unwrap();
            Ok::<_, Infallible>(receipt(request, response))
        },
    ));
    assert!(matches!(
        result,
        Err(IcSnapshotMetadataExecutionError::AfterReply {
            source: IcSnapshotMetadataSettlementError::Checkpoint(_),
            ..
        })
    ));
    assert_eq!(
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest())
            .unwrap()
            .applied_operations,
        1
    );
    assert!(
        ready(read_snapshot_metadata(
            &stage,
            42,
            &payload,
            &mut provider,
            admit,
            async |_, _| -> Result<_, Infallible> { panic!("no repeat") }
        ))
        .is_err()
    );
    assert_eq!(provider.calls, 1);
}

#[test]
fn changed_stage_during_qualification_retains_reply_and_pending_spending() {
    let workflow = workflow(12);
    let payload = source();
    let (_, layout, binding) = prepare(&workflow, &payload);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let path = stage.layout().unwrap().root().join("stage-binding.json");
    let mut provider = Provider::new();
    let result = ready(read_snapshot_metadata(
        &stage,
        42,
        &payload,
        &mut provider,
        admit,
        async |request, response| {
            fs::write(&path, b"{}").unwrap();
            Ok::<_, Infallible>(receipt(request, response))
        },
    ));
    assert!(matches!(
        result,
        Err(IcSnapshotMetadataExecutionError::AfterReply {
            source: IcSnapshotMetadataSettlementError::Stage(_),
            ..
        })
    ));
    let journal: crate::model::attempt_journal::AttemptJournalRecord =
        serde_json::from_slice(&fs::read(path.parent().unwrap().join("attempt-42.json")).unwrap())
            .unwrap();
    assert_eq!(journal.view().pending_mutation, Some(1));
}

#[test]
fn wrong_payload_sequence_missing_or_held_original_refuse_before_effects() {
    let workflow = workflow(12);
    let payload = source();
    let (_, layout, binding) = prepare(&workflow, &payload);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new();
    let wrong = IcSnapshotMetadataRequest::new(payload.target(), &[1]).unwrap();
    for (sequence, request) in [(41, &payload), (42, &wrong)] {
        assert!(matches!(
            ready(read_snapshot_metadata(
                &stage,
                sequence,
                request,
                &mut provider,
                admit,
                async |_, _| -> Result<_, Infallible> { panic!("no qualification") }
            )),
            Err(IcSnapshotMetadataExecutionError::OriginalMismatch)
        ));
    }
    let journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(42).unwrap(),
    )
    .unwrap();
    assert!(
        ready(read_snapshot_metadata(
            &stage,
            42,
            &payload,
            &mut provider,
            admit,
            async |_, _| -> Result<_, Infallible> { panic!("held original") }
        ))
        .is_err()
    );
    let path = journal.path().clone();
    drop(journal);
    fs::rename(&path, path.with_extension("retained-original")).unwrap();
    assert!(
        ready(read_snapshot_metadata(
            &stage,
            42,
            &payload,
            &mut provider,
            admit,
            async |_, _| -> Result<_, Infallible> { panic!("missing original") }
        ))
        .is_err()
    );
    assert_eq!(provider.calls, 0);
}

#[test]
fn fresh_admission_and_invalid_receipt_evidence_keep_original_pending() {
    for rejected_admission in [true, false] {
        let workflow = workflow(12);
        let payload = source();
        let (_, layout, binding) = prepare(&workflow, &payload);
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
        let mut provider = Provider::new();
        let result = ready(read_snapshot_metadata(
            &stage,
            42,
            &payload,
            &mut provider,
            async |_| {
                if rejected_admission {
                    Err(io::Error::other("not freshly admitted"))
                } else {
                    Ok(())
                }
            },
            async |request, response| {
                let mut receipt = receipt(request, response);
                receipt.evidence.clear();
                Ok(receipt)
            },
        ));
        if rejected_admission {
            assert!(matches!(
                result,
                Err(IcSnapshotMetadataExecutionError::Read(
                    IcSnapshotTransferReadExecutionError::Admission(_)
                ))
            ));
            assert_eq!(provider.calls, 0);
        } else {
            assert!(matches!(
                result,
                Err(IcSnapshotMetadataExecutionError::AfterReply {
                    source: IcSnapshotMetadataSettlementError::Journal(_),
                    ..
                })
            ));
            assert_eq!(provider.calls, 1);
        }
        pending(&stage);
    }
}

#[test]
fn multi_operation_stage_rejects_before_any_original_spending() {
    let mut allocation = serde_json::to_value(workflow(12).allocation()).unwrap();
    allocation["operations"][0]["budget"]["mutations"] = serde_json::json!(2);
    let workflow = ExecutionWorkflowRecord::new(serde_json::from_value(allocation).unwrap());
    let payload = source();
    let mut child = serde_json::to_value(source_plan(&payload)).unwrap();
    child["graph"]["nodes"] = serde_json::json!([
        {"operation_sequence":42,"depends_on":[]},
        {"operation_sequence":43,"depends_on":[]}
    ]);
    let mut second = child["operations"][0].clone();
    second["operation_sequence"] = serde_json::json!(43);
    child["operations"].as_array_mut().unwrap().push(second);
    child["budget"]["mutations"] = serde_json::json!(2);
    let plan = serde_json::from_value(child).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap();
    let root = temp_dir("ic-backup-singleton-metadata");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, &workflow).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
    let mut provider = Provider::new();
    assert!(matches!(
        ready(read_snapshot_metadata(
            &stage,
            42,
            &payload,
            &mut provider,
            admit,
            async |_, _| -> Result<_, Infallible> { panic!("not a metadata stage") }
        )),
        Err(IcSnapshotMetadataExecutionError::OriginalMismatch)
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
