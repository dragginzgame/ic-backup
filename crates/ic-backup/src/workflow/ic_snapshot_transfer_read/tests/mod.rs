//! Spending, exclusive dispatch and retained failures through the public step.

use super::*;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{
            ExecutionStageBindingRecord,
            tests::{child_request, workflow},
        },
        ic_snapshot_data::IcSnapshotDataRequest,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_transfer_read::IcSnapshotTransferReadResponseInput,
        operation_plan::{OperationPlanRecord, PlannedOperationRecord, PlannedOperationRequest},
    },
    ops::persistence::{BackupLayoutGuard, create_execution_workflow},
    test_support::temp_dir,
};
use ic_management_canister_types::{ReadCanisterSnapshotDataResult, SnapshotDataKind};
use std::{convert::Infallible, fs, path::PathBuf};

fn metadata_wire() -> Vec<u8> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    cases
        .iter()
        .find(|row| row["name"] == "data-source")
        .unwrap()["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn metadata_request() -> IcSnapshotMetadataRequest {
    IcSnapshotMetadataRequest::new(workflow().stage(0).unwrap().target(), &[0, 255, 128]).unwrap()
}
fn prepare(
    payload: IcSnapshotTransferReadPayload<'_, '_>,
) -> (
    PathBuf,
    BackupLayoutGuard,
    ExecutionStageBindingRecord,
    OperationPlanRecord,
) {
    let root = temp_dir("ic-backup-transfer-step");
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
#[expect(
    clippy::unnecessary_wraps,
    reason = "implements the mandatory fallible admission callback in this isolated fixture"
)]
fn admit(_: &IcSnapshotTransferReadRequest<'_, '_>) -> Result<(), Infallible> {
    Ok(())
}
struct Provider {
    calls: usize,
    reply: Result<Vec<u8>, IcObservationProviderError>,
    change_stage: Option<PathBuf>,
    replace_layout: Option<PathBuf>,
    target: Option<&'static str>,
}
impl Provider {
    fn new(reply: Result<Vec<u8>, IcObservationProviderError>) -> Self {
        Self {
            calls: 0,
            reply,
            change_stage: None,
            replace_layout: None,
            target: None,
        }
    }
}
impl IcSnapshotTransferReadProvider for Provider {
    fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
    ) -> Result<IcSnapshotTransferReadResponse, IcObservationProviderError> {
        self.calls += 1;
        if let Some(path) = &self.change_stage {
            fs::write(path, b"{}").unwrap();
        }
        if let Some(path) = &self.replace_layout {
            fs::rename(path, path.with_extension("retained-original")).unwrap();
            fs::create_dir(path).unwrap();
        }
        IcSnapshotTransferReadResponse::new(IcSnapshotTransferReadResponseInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: request.plan().context().clone(),
            target: self.target.unwrap_or(request.payload().target()).into(),
            reply: self.reply.clone()?,
            evidence: ArtifactChecksumRecord::from_bytes(b"fixture association only"),
        })
        .map_err(|_| IcObservationProviderError::Indeterminate)
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
    assert!(!view.applied);
    assert_eq!(view.mutations_used, 1);
}

#[test]
fn one_locked_durable_read_leaves_pending_until_explicit_integration_receipt() {
    let metadata_request = metadata_request();
    let wire = metadata_wire();
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &wire).unwrap();
    let data = IcSnapshotDataRequest::new(
        &metadata,
        SnapshotDataKind::WasmModule { offset: 0, size: 3 },
    )
    .unwrap();
    for (payload, raw) in [
        (
            IcSnapshotTransferReadPayload::Metadata(&metadata_request),
            wire.clone(),
        ),
        (
            IcSnapshotTransferReadPayload::Data(&data),
            candid::encode_one(ReadCanisterSnapshotDataResult {
                chunk: vec![7, 8, 9],
            })
            .unwrap(),
        ),
    ] {
        let (root, layout, binding, plan) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let mut provider = Provider::new(Ok(raw.clone()));
        let response = read_snapshot(&stage, 42, payload, &mut provider, |request| {
            let retained: crate::model::attempt_journal::AttemptJournalRecord =
                serde_json::from_slice(
                    &fs::read(stage.layout().unwrap().root().join("attempt-42.json")).unwrap(),
                )
                .unwrap();
            assert_eq!(
                retained.view().pending_mutation,
                Some(request.mutation_attempt())
            );
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            Ok::<(), Infallible>(())
        })
        .unwrap();
        assert_eq!(response.input().reply, raw);
        assert_eq!(provider.calls, 1);
        assert_pending(&stage, &plan);
        assert!(
            read_snapshot(
                &stage,
                42,
                payload,
                &mut provider,
                |_| -> Result<(), Infallible> {
                    panic!("pending attempt must reject before fresh admission")
                }
            )
            .is_err()
        );
        assert_eq!(provider.calls, 1);
        let mut journal = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &plan.attempt_authority(42).unwrap(),
        )
        .unwrap();
        // Explicit fixture-owned qualification; the workflow never creates this receipt.
        journal
            .record_mutation(MutationReceiptRequest {
                attempt: 1,
                request: payload.digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: response.input().evidence.hash().into(),
            })
            .unwrap();
        drop(journal);
        assert!(read_snapshot(&stage, 42, payload, &mut provider, admit).is_err());
        assert_eq!(provider.calls, 1);
        stage
            .checkpoint(ArtifactChecksumRecord::from_bytes(&raw))
            .unwrap();
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn wrong_payload_unknown_operation_missing_and_locked_originals_do_not_spend_or_dispatch() {
    let request = metadata_request();
    let payload = IcSnapshotTransferReadPayload::Metadata(&request);
    let (root, layout, binding, plan) = prepare(payload);
    let stage =
        ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
    let path = stage.layout().unwrap().root().join("attempt-42.json");
    let original = fs::read(&path).unwrap();
    let mut provider = Provider::new(Ok(metadata_wire()));
    let other = IcSnapshotMetadataRequest::new(request.target(), &[1]).unwrap();
    assert!(matches!(
        read_snapshot(
            &stage,
            42,
            IcSnapshotTransferReadPayload::Metadata(&other),
            &mut provider,
            admit
        ),
        Err(IcSnapshotTransferReadExecutionError::Request(
            IcSnapshotTransferReadError::PayloadMismatch
        ))
    ));
    assert!(matches!(
        read_snapshot(&stage, 99, payload, &mut provider, admit),
        Err(IcSnapshotTransferReadExecutionError::Plan(_))
    ));
    let held = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &plan.attempt_authority(42).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        read_snapshot(&stage, 42, payload, &mut provider, admit),
        Err(IcSnapshotTransferReadExecutionError::Journal(_))
    ));
    drop(held);
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(&path).unwrap();
    assert!(matches!(
        read_snapshot(&stage, 42, payload, &mut provider, admit),
        Err(IcSnapshotTransferReadExecutionError::Journal(_))
    ));
    assert!(!path.exists());
    assert_eq!(provider.calls, 0);
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fresh_admission_rejection_retains_spending_without_provider_or_redispatch() {
    let request = metadata_request();
    let payload = IcSnapshotTransferReadPayload::Metadata(&request);
    let (root, layout, binding, plan) = prepare(payload);
    let stage =
        ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
    let mut provider = Provider::new(Ok(metadata_wire()));
    assert!(matches!(
        read_snapshot(&stage, 42, payload, &mut provider, |_| Err(
            std::io::Error::other("fixture access refused")
        )),
        Err(IcSnapshotTransferReadExecutionError::Admission(_))
    ));
    assert_pending(&stage, &plan);
    assert!(read_snapshot(&stage, 42, payload, &mut provider, admit).is_err());
    assert_eq!(provider.calls, 0);
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn each_provider_failure_retains_pending_original_across_resume_without_reissue() {
    for failure in [
        IcObservationProviderError::Unavailable,
        IcObservationProviderError::Unsupported,
        IcObservationProviderError::Indeterminate,
    ] {
        let request = metadata_request();
        let payload = IcSnapshotTransferReadPayload::Metadata(&request);
        let (root, layout, binding, plan) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let mut provider = Provider::new(Err(failure));
        assert!(
            matches!(read_snapshot(&stage, 42, payload, &mut provider, admit), Err(IcSnapshotTransferReadExecutionError::Provider(actual)) if actual == failure)
        );
        assert_pending(&stage, &plan);
        let path = stage.layout().unwrap().root().join("attempt-42.json");
        let original = fs::read(&path).unwrap();
        drop(stage);
        let (stage, progress) =
            ExecutionStageGuard::resume(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        assert_eq!(progress.attempts.mutations_used, 1);
        assert!(read_snapshot(&stage, 42, payload, &mut provider, admit).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(provider.calls, 1);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn malformed_reply_retains_returned_bytes_and_original_pending_reservation() {
    let request = metadata_request();
    let payload = IcSnapshotTransferReadPayload::Metadata(&request);
    let (root, layout, binding, plan) = prepare(payload);
    let stage =
        ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
    let raw = b"invalid bounded candid".to_vec();
    let mut provider = Provider::new(Ok(raw.clone()));
    let error = read_snapshot(&stage, 42, payload, &mut provider, admit).unwrap_err();
    let IcSnapshotTransferReadExecutionError::Association { response, .. } = error else {
        panic!("retained association failure")
    };
    assert_eq!(response.input().reply, raw);
    assert_eq!(provider.calls, 1);
    assert_pending(&stage, &plan);
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_stage_at_admission_stops_dispatch_and_changed_stage_after_reply_retains_bytes() {
    for after_reply in [false, true] {
        let request = metadata_request();
        let payload = IcSnapshotTransferReadPayload::Metadata(&request);
        let (root, layout, binding, plan) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let path = stage.layout().unwrap().root().join("stage-binding.json");
        let original = fs::read(&path).unwrap();
        let raw = metadata_wire();
        let mut provider = Provider::new(Ok(raw.clone()));
        if after_reply {
            provider.change_stage = Some(path.clone());
        }
        let error = read_snapshot(&stage, 42, payload, &mut provider, |_| {
            if !after_reply {
                fs::write(&path, b"{}").unwrap();
            }
            Ok::<(), Infallible>(())
        })
        .unwrap_err();
        if after_reply {
            let IcSnapshotTransferReadExecutionError::AfterReplyStage { response, .. } = error
            else {
                panic!("retained reply after stage drift")
            };
            assert_eq!(response.input().reply, raw);
        } else {
            assert!(matches!(
                error,
                IcSnapshotTransferReadExecutionError::Stage(_)
            ));
        }
        assert_eq!(provider.calls, usize::from(after_reply));
        fs::write(&path, original).unwrap(); // Restore only this fixture's deliberately changed declaration.
        assert_pending(&stage, &plan);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn wrong_claim_and_replaced_layout_retain_reply_without_outcome_or_repair() {
    for replaced in [false, true] {
        let request = metadata_request();
        let payload = IcSnapshotTransferReadPayload::Metadata(&request);
        let (root, layout, binding, _) = prepare(payload);
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 0, &binding.digest()).unwrap();
        let stage_root = stage.layout().unwrap().root().to_path_buf();
        let raw = metadata_wire();
        let mut provider = Provider::new(Ok(raw.clone()));
        if replaced {
            provider.replace_layout = Some(stage_root.clone());
        } else {
            provider.target = Some("aaaaa-aa");
        }
        let error = read_snapshot(&stage, 42, payload, &mut provider, admit).unwrap_err();
        let response = match error {
            IcSnapshotTransferReadExecutionError::AfterReplyJournal { response, .. }
                if replaced =>
            {
                response
            }
            IcSnapshotTransferReadExecutionError::Association {
                source: IcSnapshotTransferReadAssociationError::TargetMismatch,
                response,
            } if !replaced => response,
            other => panic!("unexpected retained rejection: {other:?}"),
        };
        assert_eq!(response.input().reply, raw);
        let retained = if replaced {
            stage_root.with_extension("retained-original")
        } else {
            stage_root
        };
        let journal: crate::model::attempt_journal::AttemptJournalRecord =
            serde_json::from_slice(&fs::read(retained.join("attempt-42.json")).unwrap()).unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert!(!journal.view().applied);
        assert_eq!(provider.calls, 1);
        drop(stage);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
