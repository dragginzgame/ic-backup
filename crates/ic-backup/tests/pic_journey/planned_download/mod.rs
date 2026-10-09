//! Real bounded downloads under one original workflow; this is an isolated test driver.

use super::{Fault, backend::Backend, lifecycle, management, status};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        download_journal::{ArtifactStateRecord, DownloadArtifactRequest},
        execution_settlement::{ExecutionSettlementJournalRecord, ExecutionSettlementRecord},
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
        },
        ic_mutation::IcMutationRequest,
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_reply::IcSnapshotReply,
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadPayload, IcSnapshotTransferReadRequest,
            IcSnapshotTransferReadResponse,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard,
        create_execution_settlement, create_execution_workflow, read_execution_progress,
    },
    policy::ic_snapshot_transfer_read::{IcSnapshotTransferReadReply, validate_response},
    ports::ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
};
use ic_management_canister_types::CanisterStatusType;
use serde_json::json;
use std::{fs, os::unix::fs::DirBuilderExt};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReadFailure {
    None,
    Lost,
    Malformed,
}

fn workflow(backend: &Backend) -> ExecutionWorkflowRecord {
    let mut allocation = serde_json::to_value(
        backend.plan(&ArtifactChecksumRecord::from_bytes(b"capture contract"), 0),
    )
    .unwrap();
    allocation["graph"]["nodes"] = json!([
        {"operation_sequence":0,"depends_on":[]},
        {"operation_sequence":7,"depends_on":[0]},
        {"operation_sequence":9,"depends_on":[7]}
    ]);
    allocation["operations"] = json!([
        {"operation_sequence":0,"target":backend.target.to_text(),"request":ArtifactChecksumRecord::from_bytes(b"capture contract").hash(),"budget":{"mutations":1,"observations":1}},
        {"operation_sequence":7,"target":backend.target.to_text(),"request":ArtifactChecksumRecord::from_bytes(b"metadata contract").hash(),"budget":{"mutations":1,"observations":1}},
        {"operation_sequence":9,"target":backend.target.to_text(),"request":ArtifactChecksumRecord::from_bytes(b"complete original metadata data reads").hash(),"budget":{"mutations":64,"observations":0}}
    ]);
    allocation["budget"] = json!({"mutations":66,"observations":2});
    ExecutionWorkflowRecord::new(serde_json::from_value(allocation).unwrap())
}
fn record_applied(journal: &mut AttemptJournalGuard<'_>, evidence: &ArtifactChecksumRecord) {
    let record = journal.record().unwrap();
    let receipt = MutationReceiptRequest {
        attempt: record.view().pending_mutation.unwrap(),
        request: record.authority().binding().request().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: evidence.hash().into(),
    };
    journal.record_mutation(receipt).unwrap();
}
fn settle(layout: &BackupLayoutGuard, plan: &OperationPlanRecord) -> ArtifactChecksumRecord {
    let rows = plan
        .attempt_authorities()
        .unwrap()
        .iter()
        .map(|authority| {
            let journal = AttemptJournalGuard::open(layout, authority).unwrap();
            ExecutionSettlementJournalRecord::from_journal(journal.record().unwrap())
        })
        .collect();
    let record = ExecutionSettlementRecord::new(plan.digest(), rows).unwrap();
    create_execution_settlement(layout, &record).unwrap();
    record.digest()
}
fn capture(
    backend: &mut Backend,
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
) -> (
    ic_backup::model::ic_snapshot_reply::IcSnapshotInfo,
    ExecutionStagePredecessorRecord,
) {
    let payload = management(backend, Method::TakeCanisterSnapshot, None);
    let plan = backend.plan(&payload.digest(), 0);
    let binding = ExecutionStageBindingRecord::new(workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(layout, binding.clone(), plan).unwrap();
    let stage_layout = stage.layout().unwrap();
    let mut journal =
        AttemptJournalGuard::open(stage_layout, &stage.plan().attempt_authority(0).unwrap())
            .unwrap();
    journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    let request =
        IcMutationRequest::new(stage.plan(), 0, journal.record().unwrap(), &payload).unwrap();
    let raw = backend.management(
        request.payload().method().name(),
        request.payload().arguments(),
    );
    fs::write(stage_layout.root().join("capture-reply.candid"), &raw).unwrap();
    let reply = IcSnapshotReply::decode(&payload, &raw).unwrap();
    // The isolated fixture owns this exact successful ingress; no list cardinality
    // or discarded reply is used to infer original attribution.
    record_applied(&mut journal, &reply.digest());
    drop(journal);
    let settlement = settle(stage_layout, stage.plan());
    (
        reply.snapshots()[0].clone(),
        ExecutionStagePredecessorRecord::new(0, binding.digest(), settlement, reply.digest()),
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "keep the isolated ingress, spending and retained-byte chronology in one qualification trace"
)]
pub(crate) fn run(failure: ReadFailure) {
    let mut backend = Backend::new();
    // Actual fixture-specific stopped/no-external-effects admission and separately
    // accounted lifecycle/status calls; this is never a generic application default.
    assert!(
        lifecycle(
            &mut backend,
            Method::StopCanister,
            None,
            Fault::None,
            |_| None
        )
        .is_none()
    );
    status(&mut backend, CanisterStatusType::Stopped);
    let root = backend.root.join("planned-workflow");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let workflow = workflow(&backend);
    create_execution_workflow(&layout, &workflow).unwrap();
    let (snapshot, capture) = capture(&mut backend, &layout, &workflow);
    let metadata_request =
        IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    let metadata_plan = backend.plan(&metadata_request.digest(), 7);
    let metadata_binding =
        ExecutionStageBindingRecord::new(&workflow, 7, &metadata_plan, vec![capture]).unwrap();
    let metadata_stage =
        ExecutionStageGuard::prepare(&layout, metadata_binding.clone(), metadata_plan.clone())
            .unwrap();
    let mut journal = AttemptJournalGuard::open(
        metadata_stage.layout().unwrap(),
        &metadata_plan.attempt_authority(7).unwrap(),
    )
    .unwrap();
    journal
        .reserve_planned_mutation(&metadata_plan.digest())
        .unwrap();
    let request = IcSnapshotTransferReadRequest::new(
        &metadata_plan,
        7,
        journal.record().unwrap(),
        IcSnapshotTransferReadPayload::Metadata(&metadata_request),
    )
    .unwrap();
    let response = backend.read_snapshot(&request).unwrap();
    let admitted = validate_response(&request, journal.record().unwrap(), &response).unwrap();
    assert!(matches!(
        admitted.reply(),
        IcSnapshotTransferReadReply::Metadata(_)
    ));
    let raw_metadata = &response.input().reply;
    fs::write(
        metadata_stage
            .layout()
            .unwrap()
            .root()
            .join("metadata.candid"),
        raw_metadata,
    )
    .unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, raw_metadata).unwrap();
    record_applied(&mut journal, &metadata.digest());
    drop(journal);
    let metadata_settlement = settle(metadata_stage.layout().unwrap(), &metadata_plan);
    drop(metadata_stage);
    let original = ExecutionStagePredecessorRecord::new(
        7,
        metadata_binding.digest(),
        metadata_settlement,
        metadata.digest(),
    );
    let download = IcSnapshotDownloadPlan::new(&workflow, 9, &metadata, 32 * 1024).unwrap();
    let binding = download
        .bind(&metadata_binding, &metadata_plan, vec![original])
        .unwrap();
    let plan = download.plan().unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan.clone()).unwrap();
    let stage_layout = stage.layout().unwrap();
    // The public preparation owner retained every original journal before ingress.
    fs::DirBuilder::new()
        .mode(0o700)
        .create(stage_layout.root().join("artifacts"))
        .unwrap();
    stage_layout
        .retain_restore(
            &backend.root.join("unfinished-restore.json"),
            plan.digest().hash(),
        )
        .unwrap();
    let references = stage_layout.restore_references().unwrap();
    let token = "qualified-original-raw-snapshot";
    let mut artifacts = DownloadJournalGuard::create(
        stage_layout,
        plan.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: backend.target.to_text(),
            snapshot_id: token.into(),
            snapshot_taken_at_timestamp: snapshot.taken_at_timestamp(),
            snapshot_total_size_bytes: snapshot.total_size(),
        }],
    )
    .unwrap();
    let mut writer = artifacts
        .stage_ic_snapshot_artifact(token, &metadata, raw_metadata)
        .unwrap();
    let calls_before_data = backend.calls;
    let mut stopped = false;
    for (index, payload) in download.requests().iter().enumerate() {
        let sequence = u64::try_from(index).unwrap();
        let mut journal =
            AttemptJournalGuard::open(stage_layout, &plan.attempt_authority(sequence).unwrap())
                .unwrap();
        journal.reserve_planned_mutation(&plan.digest()).unwrap();
        let request = IcSnapshotTransferReadRequest::new(
            plan,
            sequence,
            journal.record().unwrap(),
            IcSnapshotTransferReadPayload::Data(payload),
        )
        .unwrap();
        let response = backend.read_snapshot(&request).unwrap();
        if index == 1 && failure != ReadFailure::None {
            fs::write(
                stage_layout.root().join("discarded-data-oracle.candid"),
                &response.input().reply,
            )
            .unwrap();
            if failure == ReadFailure::Malformed {
                let mut input = response.input().clone();
                input.reply.clear();
                let malformed = IcSnapshotTransferReadResponse::new(input).unwrap();
                assert!(
                    validate_response(&request, journal.record().unwrap(), &malformed).is_err()
                );
            }
            assert!(journal.record().unwrap().view().pending_mutation.is_some());
            stopped = true;
            break;
        }
        let admitted = validate_response(&request, journal.record().unwrap(), &response).unwrap();
        let IcSnapshotTransferReadReply::Data(reply) = admitted.reply() else {
            panic!("exact data method");
        };
        fs::write(
            stage_layout.root().join(format!("data-{sequence}.candid")),
            &response.input().reply,
        )
        .unwrap();
        let evidence = reply.digest();
        writer = writer.append(reply).unwrap();
        record_applied(&mut journal, &evidence);
    }
    if stopped {
        drop(writer);
        let view = read_execution_progress(stage_layout, &plan.digest()).unwrap();
        assert_eq!(view.attempts.mutations_used, 2);
        assert_eq!(view.applied_operations, 1);
        assert_eq!(backend.calls, calls_before_data + 2);
        assert_eq!(
            artifacts.record().unwrap().artifacts()[0].state(),
            ArtifactStateRecord::Created
        );
        let mut next =
            AttemptJournalGuard::open(stage_layout, &plan.attempt_authority(2).unwrap()).unwrap();
        assert!(next.reserve_planned_mutation(&plan.digest()).is_err());
        drop(next);
        let original = fs::read(stage_layout.root().join("attempt-1.json")).unwrap();
        drop(artifacts);
        drop(stage);
        let (reopened, resumed) =
            ExecutionStageGuard::resume(&layout, &workflow.digest(), 9, &binding.digest()).unwrap();
        let current = reopened.layout().unwrap();
        assert_eq!(resumed, view);
        assert_eq!(
            fs::read(current.root().join("attempt-1.json")).unwrap(),
            original
        );
        assert_eq!(current.restore_references().unwrap(), references);
        assert_eq!(backend.calls, calls_before_data + 2);
        assert!(ExecutionStageGuard::create(&layout, binding, plan.clone()).is_err());
        return;
    }
    let checksum = writer.finish().unwrap();
    assert_eq!(
        artifacts
            .verify_ic_snapshot_artifact(plan, token, &metadata)
            .unwrap(),
        checksum
    );
    let manifest = artifacts.publish_download_manifest(plan).unwrap();
    let retained = artifacts.record().unwrap().clone();
    drop(artifacts);
    settle(stage_layout, plan);
    let view = read_execution_progress(stage_layout, &plan.digest()).unwrap();
    assert_eq!(view.applied_operations, download.requests().len());
    assert_eq!(view.attempts.mutations_remaining, 0); // Original spare headroom is unassigned.
    assert_eq!(
        backend.calls,
        calls_before_data + u64::try_from(download.requests().len()).unwrap()
    );
    let calls = backend.calls;
    drop(stage);
    let (reopened, resumed) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 9, &binding.digest()).unwrap();
    assert_eq!(resumed, view);
    let journal =
        DownloadJournalGuard::open(reopened.layout().unwrap(), plan.digest().hash()).unwrap();
    assert_eq!(
        journal.read_download_manifest(plan, &manifest).unwrap(),
        retained
    );
    assert_eq!(
        reopened.layout().unwrap().restore_references().unwrap(),
        references
    );
    assert_eq!(backend.calls, calls);
}
