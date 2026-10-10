//! Exact original upload spending and one-call custody; no live backend or source proof.

use super::*;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_lifecycle_reply::EMPTY_CANDID_REPLY,
        ic_mutation::IcMutationAcknowledgementInput,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{BackupLayoutGuard, create_execution_workflow, read_execution_progress},
    test_support::{
        ic_snapshot_upload::{
            DESTINATION_ID, SOURCE_ID, TARGET, raw, source_plan, unhex, upload_plan,
        },
        temp_dir,
    },
};
use ic_management_canister_types::SnapshotDataKind;
use std::{convert::Infallible, fs, io, path::PathBuf};

fn with_uploads(mut check: impl FnMut(&IcSnapshotUploadRequest<'_>, Vec<u8>)) {
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let raw = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"declared source tree"),
    )
    .unwrap();
    let data = IcSnapshotUploadRequest::data(
        &upload,
        DESTINATION_ID,
        SnapshotDataKind::WasmModule { offset: 0, size: 3 },
        &[7, 8, 9],
    )
    .unwrap();
    check(&upload, unhex("4449444c026c01b6b897890f016d7b0100031500ff"));
    check(&data, EMPTY_CANDID_REPLY.to_vec());
}
fn prepare(plan: OperationPlanRecord) -> (PathBuf, BackupLayoutGuard, ExecutionStageBindingRecord) {
    let root = temp_dir("ic-backup-upload-step");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    create_execution_workflow(&layout, &workflow).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![]).unwrap();
    drop(ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap());
    (root, layout, binding)
}
struct Provider {
    calls: usize,
    reply: Result<Vec<u8>, IcMutationProviderError>,
    change: Option<PathBuf>,
    target: Option<&'static str>,
}
impl Provider {
    fn new(reply: Vec<u8>) -> Self {
        Self {
            calls: 0,
            reply: Ok(reply),
            change: None,
            target: None,
        }
    }
}
impl IcSnapshotUploadProvider for Provider {
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        self.calls += 1;
        if let Some(path) = &self.change {
            fs::write(path, b"{}").unwrap();
        }
        let reply = self.reply.clone()?;
        IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: request.plan().context().clone(),
            target: self.target.unwrap_or(request.payload().target()).into(),
            evidence: ArtifactChecksumRecord::from_bytes(&reply),
            reply,
        })
        .map_err(|_| IcMutationProviderError::Indeterminate)
    }
}
#[expect(
    clippy::unnecessary_wraps,
    reason = "mandatory fallible integration admission"
)]
fn admit(_: &IcSnapshotUploadAttempt<'_, '_>) -> Result<(), Infallible> {
    Ok(())
}
fn pending(stage: &ExecutionStageGuard<'_>) {
    let view = read_execution_progress(stage.layout().unwrap(), &stage.plan().digest()).unwrap();
    assert_eq!(view.attempts.mutations_used, 1);
    assert_eq!(view.applied_operations, 0);
    let journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(7).unwrap(),
    )
    .unwrap();
    assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
}

#[test]
fn exact_metadata_and_data_are_durably_reserved_locked_and_remain_pending_on_success() {
    with_uploads(|payload, raw| {
        let (_, layout, binding) = prepare(upload_plan(payload));
        let stage =
            ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest()).unwrap();
        let mut provider = Provider::new(raw.clone());
        let response = upload_snapshot(&stage, 7, payload, &mut provider, |request| {
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
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            assert_eq!(request.payload().binding_digest(), payload.binding_digest());
            assert_eq!(request.payload().receiver(), "aaaaa-aa");
            Ok::<_, Infallible>(())
        })
        .unwrap();
        assert_eq!(response.input().reply, raw);
        assert_eq!(provider.calls, 1);
        pending(&stage);
        assert!(
            upload_snapshot(
                &stage,
                7,
                payload,
                &mut provider,
                |_| -> Result<(), Infallible> { panic!("pending is not new custody") }
            )
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
            let mut provider = Provider::new(raw.clone());
            provider.reply = Err(failure);
            assert!(matches!(
                upload_snapshot(&stage, 7, payload, &mut provider, admit),
                Err(IcSnapshotUploadExecutionError::Provider(_))
            ));
            pending(&stage);
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
                upload_snapshot(
                    &stage,
                    7,
                    payload,
                    &mut provider,
                    |_| -> Result<(), Infallible> { panic!("never reissue") }
                )
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
            let mut provider = Provider::new(expected.clone());
            if mismatched {
                provider.target = Some("aaaaa-aa");
            }
            let Err(IcSnapshotUploadExecutionError::Association {
                acknowledgement, ..
            }) = upload_snapshot(&stage, 7, payload, &mut provider, admit)
            else {
                panic!("passive association must reject")
            };
            assert_eq!(acknowledgement.input().reply, expected);
            pending(&stage);
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
        let mut provider = Provider::new(raw);
        assert!(matches!(
            upload_snapshot(&stage, 7, payload, &mut provider, |_| Err(
                io::Error::other("no fresh controller/source/destination custody")
            )),
            Err(IcSnapshotUploadExecutionError::Admission(_))
        ));
        pending(&stage);
        assert_eq!(provider.calls, 0);
    });
}

#[test]
fn changed_payload_source_network_or_release_reject_before_spending() {
    with_uploads(|payload, raw| {
        for field in ["request", "network", "release"] {
            let mut plan = serde_json::to_value(upload_plan(payload)).unwrap();
            if field == "request" {
                plan["operations"][0]["request"] = serde_json::json!("00".repeat(32));
            } else {
                plan["context"][field] = serde_json::json!("00".repeat(32));
            }
            let (_, layout, binding) = prepare(serde_json::from_value(plan).unwrap());
            let stage =
                ExecutionStageGuard::open(&layout, binding.workflow(), 7, &binding.digest())
                    .unwrap();
            let mut provider = Provider::new(raw.clone());
            assert!(matches!(
                upload_snapshot(
                    &stage,
                    7,
                    payload,
                    &mut provider,
                    |_| -> Result<(), Infallible> { panic!("wrong originals") }
                ),
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
        let mut provider = Provider::new(raw);
        let mut journal = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &stage.plan().attempt_authority(7).unwrap(),
        )
        .unwrap();
        assert!(upload_snapshot(&stage, 7, payload, &mut provider, admit).is_err());
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
        assert!(upload_snapshot(&stage, 7, payload, &mut provider, admit).is_err());
        fs::rename(&path, path.with_extension("retained-original")).unwrap();
        assert!(upload_snapshot(&stage, 7, payload, &mut provider, admit).is_err());
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
            let mut provider = Provider::new(raw.clone());
            if after_reply {
                provider.change = Some(path.clone());
            }
            let result = upload_snapshot(&stage, 7, payload, &mut provider, |_| {
                if !after_reply {
                    fs::write(&path, b"{}").unwrap();
                }
                Ok::<_, Infallible>(())
            });
            if after_reply {
                let Err(IcSnapshotUploadExecutionError::AfterReplyStage {
                    acknowledgement, ..
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
        let mut child = serde_json::to_value(upload_plan(payload)).unwrap();
        let mut allocation = child.clone();
        allocation["operations"][0]["budget"]["mutations"] = serde_json::json!(2);
        allocation["budget"]["mutations"] = serde_json::json!(2);
        let workflow = ExecutionWorkflowRecord::new(serde_json::from_value(allocation).unwrap());
        child["budget"]["mutations"] = serde_json::json!(2);
        child["graph"]["nodes"] = serde_json::json!([
            {"operation_sequence":7,"depends_on":[]},
            {"operation_sequence":8,"depends_on":[7]}
        ]);
        let mut second = child["operations"][0].clone();
        second["operation_sequence"] = serde_json::json!(8);
        second["budget"]["observations"] = serde_json::json!(0);
        child["operations"].as_array_mut().unwrap().push(second);
        let plan = serde_json::from_value(child).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![]).unwrap();
        let root = temp_dir("ic-backup-upload-prerequisite");
        fs::create_dir(&root).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        create_execution_workflow(&layout, &workflow).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
        let mut provider = Provider::new(raw);
        assert!(upload_snapshot(&stage, 8, payload, &mut provider, admit).is_err());
        upload_snapshot(&stage, 7, payload, &mut provider, admit).unwrap();
        assert!(upload_snapshot(&stage, 8, payload, &mut provider, admit).is_err());
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
        upload_snapshot(&stage, 8, payload, &mut provider, admit).unwrap();
        assert_eq!(provider.calls, 2);
    });
}
