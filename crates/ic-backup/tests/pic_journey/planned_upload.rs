//! Actual original metadata/data stages using the public one-upload coordinator.

use super::{Backend, Fault, capture, download, extents, lifecycle, status, verify_destination};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_mutation::{IcMutationAcknowledgement, IcMutationAcknowledgementInput},
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_transfer_read::IcSnapshotTransferReadPayload,
        ic_snapshot_upload::{
            IcSnapshotUploadAttempt, IcSnapshotUploadKind, IcSnapshotUploadReply,
            IcSnapshotUploadReplyKind, IcSnapshotUploadRequest,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard,
        create_execution_workflow, read_execution_progress,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
    workflow::ic_snapshot_upload::{
        IcSnapshotAllocationExecutionError, IcSnapshotUploadExecutionError, allocate_snapshot,
        upload_snapshot,
    },
};
use ic_management_canister_types::CanisterStatusType;
use serde_json::json;
use std::{collections::BTreeSet, convert::Infallible, fs, io::Write, os::unix::fs::DirBuilderExt};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Failure {
    None,
    MetadataLost,
    MetadataMalformed,
    DataLost,
    DataMalformed,
}
struct Provider<'a> {
    backend: &'a mut Backend,
    failure: Failure,
    submitted: BTreeSet<(String, u32)>,
}
impl IcSnapshotUploadProvider for Provider<'_> {
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        assert!(self.submitted.insert((
            request.authority().digest().hash().into(),
            request.mutation_attempt()
        )));
        let payload = request.payload();
        let actual = self.backend.plan(
            &payload.binding_digest(),
            request.authority().binding().operation_sequence(),
        );
        assert_eq!(request.plan().context(), actual.context());
        assert_eq!(payload.receiver(), "aaaaa-aa");
        assert_eq!(payload.target(), self.backend.target.to_text());
        let raw = self
            .backend
            .management(payload.method(), payload.arguments());
        let metadata = matches!(payload.kind(), IcSnapshotUploadKind::Metadata);
        let lost = matches!(
            (metadata, self.failure),
            (true, Failure::MetadataLost) | (false, Failure::DataLost)
        );
        let malformed = matches!(
            (metadata, self.failure),
            (true, Failure::MetadataMalformed) | (false, Failure::DataMalformed)
        );
        if lost || malformed {
            // The independent test oracle never enters outcome/destination admission.
            fs::write(
                self.backend.root.join("discarded-upload-oracle.candid"),
                &raw,
            )
            .unwrap();
        }
        if lost {
            return Err(IcMutationProviderError::Indeterminate);
        }
        let reply = if malformed { vec![] } else { raw };
        IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: actual.context().clone(),
            target: self.backend.target.to_text(),
            evidence: ArtifactChecksumRecord::from_bytes(&reply),
            reply,
        })
        .map_err(|_| IcMutationProviderError::Indeterminate)
    }
}
fn retain_applied(
    stage: &ExecutionStageGuard<'_>,
    sequence: u64,
    payload: &IcSnapshotUploadRequest<'_>,
    acknowledgement: &IcMutationAcknowledgement,
) -> ArtifactChecksumRecord {
    // Explicit fixture-owned exact ingress attribution, never a library wire-derived receipt.
    let layout = stage.layout().unwrap();
    let mut journal =
        AttemptJournalGuard::open(layout, &stage.plan().attempt_authority(sequence).unwrap())
            .unwrap();
    let reply = IcSnapshotUploadReply::decode(payload, &acknowledgement.input().reply).unwrap();
    retain_reply(layout, sequence, acknowledgement);
    let digest = reply.digest();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt: acknowledgement.input().mutation_attempt,
            request: payload.binding_digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: digest.hash().into(),
        })
        .unwrap();
    digest
}
fn retain_reply(
    layout: &BackupLayoutGuard,
    sequence: u64,
    acknowledgement: &IcMutationAcknowledgement,
) {
    let file = fs::File::create(layout.root().join(format!("reply-{sequence}.candid"))).unwrap();
    (&file).write_all(&acknowledgement.input().reply).unwrap();
    file.sync_all().unwrap();
    fs::File::open(layout.root()).unwrap().sync_all().unwrap();
}
fn allocation(
    backend: &Backend,
    metadata: &IcSnapshotUploadRequest<'_>,
    data_count: usize,
) -> ExecutionWorkflowRecord {
    let count = u32::try_from(data_count).unwrap();
    let mut plan = serde_json::to_value(backend.plan(&metadata.binding_digest(), 0)).unwrap();
    plan["graph"]["nodes"] = json!([
        {"operation_sequence":0,"depends_on":[]}, {"operation_sequence":7,"depends_on":[0]}
    ]);
    let mut data = plan["operations"][0].clone();
    data["operation_sequence"] = json!(7);
    data["request"] =
        json!(ArtifactChecksumRecord::from_bytes(b"original full data allocation").hash());
    data["budget"] = json!({"mutations":count,"observations":0});
    plan["operations"][0]["budget"] = json!({"mutations":1,"observations":0});
    plan["operations"].as_array_mut().unwrap().push(data);
    plan["budget"] = json!({"mutations":count+1,"observations":0});
    ExecutionWorkflowRecord::new(serde_json::from_value(plan).unwrap())
}
fn data_plan(backend: &Backend, payloads: &[IcSnapshotUploadRequest<'_>]) -> OperationPlanRecord {
    let mut plan = serde_json::to_value(backend.plan(&payloads[0].binding_digest(), 0)).unwrap();
    plan["graph"]["nodes"] = json!([]);
    plan["operations"] = json!([]);
    for (index, payload) in payloads.iter().enumerate() {
        let sequence = u64::try_from(index).unwrap();
        plan["graph"]["nodes"].as_array_mut().unwrap().push(json!({"operation_sequence":sequence,"depends_on":sequence.checked_sub(1).into_iter().collect::<Vec<_>>()}));
        plan["operations"].as_array_mut().unwrap().push(json!({"operation_sequence":sequence,"target":payload.target(),"request":payload.binding_digest().hash(),"budget":{"mutations":1,"observations":0}}));
    }
    plan["budget"] = json!({"mutations":payloads.len(),"observations":0});
    serde_json::from_value(plan).unwrap()
}
fn assert_failed(
    stage: ExecutionStageGuard<'_>,
    workflow_layout: &BackupLayoutGuard,
    payload: &IcSnapshotUploadRequest<'_>,
    sequence: u64,
    provider: &mut Provider<'_>,
    result: Result<IcMutationAcknowledgement, IcSnapshotUploadExecutionError<Infallible>>,
) {
    match result {
        Err(IcSnapshotUploadExecutionError::Provider(IcMutationProviderError::Indeterminate)) => {}
        Err(IcSnapshotUploadExecutionError::Association {
            acknowledgement, ..
        }) => assert_eq!(acknowledgement.input().reply, [] as [u8; 0]),
        _ => panic!("lost/malformed upload safe stop"),
    }
    let binding = stage.binding().clone();
    let root = stage.layout().unwrap().root().to_path_buf();
    let original = fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap();
    let references = stage.layout().unwrap().restore_references().unwrap();
    assert!(
        stage
            .checkpoint(ArtifactChecksumRecord::from_bytes(b"not an outcome"))
            .is_err()
    );
    drop(stage);
    let (stage, view) = ExecutionStageGuard::resume(
        workflow_layout,
        binding.workflow(),
        binding.stage_sequence(),
        &binding.digest(),
    )
    .unwrap();
    assert_eq!(view.attempts.mutations_used, 1);
    assert_eq!(view.applied_operations, 0);
    let calls = provider.backend.calls;
    assert!(
        upload_snapshot(
            &stage,
            sequence,
            payload,
            provider,
            |_| -> Result<(), Infallible> { panic!("pending upload never reissues") }
        )
        .is_err()
    );
    assert_eq!(provider.backend.calls, calls);
    assert_eq!(
        fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap(),
        original
    );
    assert_eq!(
        stage.layout().unwrap().restore_references().unwrap(),
        references
    );
}

#[expect(
    clippy::too_many_lines,
    reason = "retain the isolated source/allocation/data custody chronology in one qualification trace"
)]
pub(crate) fn run(failure: Failure) {
    let mut backend = Backend::new();
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
    let (source_plan, source_id, timestamp, size) = capture(&mut backend, Fault::None);
    let request = IcSnapshotMetadataRequest::new(&backend.target.to_text(), &source_id).unwrap();
    let (_, raw) = backend.read(IcSnapshotTransferReadPayload::Metadata(&request), false);
    let raw = raw.unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let source = download(
        &mut backend,
        &source_plan,
        &metadata,
        &raw,
        timestamp,
        size,
        false,
    )
    .unwrap();
    let source_layout = BackupLayoutGuard::acquire(&source.root).unwrap();
    let source_journal =
        DownloadJournalGuard::open(&source_layout, source_plan.digest().hash()).unwrap();
    let original_source = fs::read(source_journal.path()).unwrap();
    let original_references = source_layout.restore_references().unwrap();
    let upload = source_journal
        .prepare_ic_snapshot_upload_metadata(&source_plan, "retained-original-snapshot", &metadata)
        .unwrap();
    let kinds = extents(&metadata);
    let workflow = allocation(&backend, &upload, kinds.len());
    let root = backend.root.join("original-upload-workflow");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, &workflow).unwrap();
    let mut metadata_plan =
        serde_json::to_value(backend.plan(&upload.binding_digest(), 0)).unwrap();
    metadata_plan["operations"][0]["budget"]["observations"] = json!(0);
    metadata_plan["budget"]["observations"] = json!(0);
    let metadata_plan = serde_json::from_value(metadata_plan).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &metadata_plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, metadata_plan).unwrap();
    stage
        .layout()
        .unwrap()
        .retain_restore(
            &source.root.join("unfinished-upload.json"),
            source_plan.digest().hash(),
        )
        .unwrap();
    let mut provider = Provider {
        backend: &mut backend,
        failure,
        submitted: BTreeSet::new(),
    };
    let result = allocate_snapshot(
        &stage,
        0,
        &upload,
        &mut provider,
        |request| {
            // This stopped fixture owns current controller, immutable complete source and
            // original never-dispatched custody; the generic library has no default.
            assert_eq!(request.payload().source_checksum(), &source.checksum);
            Ok::<_, Infallible>(())
        },
        |request, acknowledgement| {
            // The fixture independently owns this exact successful allocation ingress.
            // Retain actual original reply bytes before qualifying its explicit receipt.
            retain_reply(stage.layout().unwrap(), 0, acknowledgement);
            let reply =
                IcSnapshotUploadReply::decode(request.payload(), &acknowledgement.input().reply)
                    .unwrap();
            Ok(MutationReceiptRequest {
                attempt: request.mutation_attempt(),
                request: request.payload().binding_digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: reply.digest().hash().into(),
            })
        },
    );
    if matches!(failure, Failure::MetadataLost | Failure::MetadataMalformed) {
        let result = match result {
            Err(IcSnapshotAllocationExecutionError::Upload(source)) => Err(source),
            Ok((acknowledgement, _)) => Ok(acknowledgement),
            Err(source) => panic!("unexpected allocation settlement: {source}"),
        };
        assert_failed(stage, &layout, &upload, 0, &mut provider, result);
        assert!(!layout.root().join("execution-stage-7").exists());
        assert_eq!(provider.submitted.len(), 1);
        assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
        assert_eq!(
            source_layout.restore_references().unwrap(),
            original_references
        );
        return;
    }
    let (acknowledgement, predecessor) = result.unwrap();
    let reply = IcSnapshotUploadReply::decode(&upload, &acknowledgement.input().reply).unwrap();
    let IcSnapshotUploadReplyKind::Metadata {
        snapshot_id: destination,
    } = reply.kind()
    else {
        panic!("exact allocation reply")
    };
    // The fixture owns exact successful allocation ingress, not singleton-list attribution.
    let payloads: Vec<_> = kinds
        .iter()
        .map(|kind| {
            source_journal
                .prepare_ic_snapshot_upload_data(
                    &source_plan,
                    "retained-original-snapshot",
                    &upload,
                    destination,
                    kind.clone(),
                )
                .unwrap()
        })
        .collect();
    drop(stage);
    let plan = data_plan(provider.backend, &payloads);
    let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![predecessor]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
    stage
        .layout()
        .unwrap()
        .retain_restore(
            &source.root.join("unfinished-upload.json"),
            source_plan.digest().hash(),
        )
        .unwrap();
    for (index, payload) in payloads.iter().enumerate() {
        let sequence = u64::try_from(index).unwrap();
        let result = upload_snapshot(&stage, sequence, payload, &mut provider, |request| {
            assert_eq!(request.payload().source_checksum(), &source.checksum);
            assert!(
                matches!(request.payload().kind(), IcSnapshotUploadKind::Data { snapshot_id, .. } if snapshot_id == destination)
            );
            Ok::<_, Infallible>(())
        });
        if matches!(failure, Failure::DataLost | Failure::DataMalformed) {
            assert_failed(stage, &layout, payload, sequence, &mut provider, result);
            assert_eq!(provider.submitted.len(), 2);
            assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
            assert_eq!(
                source_layout.restore_references().unwrap(),
                original_references
            );
            return;
        }
        let acknowledgement = result.unwrap();
        retain_applied(&stage, sequence, payload, &acknowledgement);
    }
    assert_eq!(
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest())
            .unwrap()
            .applied_operations,
        payloads.len()
    );
    let calls = provider.backend.calls;
    assert!(
        upload_snapshot(
            &stage,
            0,
            &payloads[0],
            &mut provider,
            |_| -> Result<(), Infallible> { panic!("Applied cannot replay") }
        )
        .is_err()
    );
    assert_eq!(provider.backend.calls, calls);
    verify_destination(provider.backend, &metadata, destination, &source.chunks);
    assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
    assert_eq!(
        source_layout.restore_references().unwrap(),
        original_references
    );
    assert_eq!(provider.submitted.len(), payloads.len() + 1);
}
