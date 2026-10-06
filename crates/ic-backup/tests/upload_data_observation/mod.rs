//! Native retained readback/refusal only; no management effect simulation.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        ic_observation::IcObservationResponseInput,
        ic_snapshot_data::IcSnapshotDataRequest,
        ic_snapshot_metadata::{
            IcSnapshotMetadataReply, IcSnapshotMetadataRequest, MAX_IC_SNAPSHOT_METADATA_BYTES,
        },
        ic_snapshot_upload::{IcSnapshotUploadKind, IcSnapshotUploadRequest},
        ic_snapshot_upload_data_observation::{
            IcSnapshotUploadDataAttribution, IcSnapshotUploadDataObservationRequest,
            IcSnapshotUploadDataObservationResponse, IcSnapshotUploadDataSettlement,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_json_durable, read_json},
    policy::ic_snapshot_upload_data_observation::{validate_response, validate_settlement},
    ports::{
        ic_observation::IcObservationProviderError,
        ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationProvider,
    },
};
use std::fs;

struct RefusingProvider {
    calls: usize,
}
impl IcSnapshotUploadDataObservationProvider for RefusingProvider {
    fn observe_upload_data(
        &mut self,
        _request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
    ) -> Result<IcSnapshotUploadDataObservationResponse, IcObservationProviderError> {
        self.calls += 1;
        assert_eq!(self.calls, 1, "no hidden retry or local reopen dispatch");
        Err(IcObservationProviderError::Indeterminate)
    }
}

fn readback_response(
    request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
    chunk: &[u8],
) -> IcSnapshotUploadDataObservationResponse {
    IcSnapshotUploadDataObservationResponse::new(IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: request.payload().digest(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply: candid::encode_one(
            ic_management_canister_types::ReadCanisterSnapshotDataResult {
                chunk: chunk.to_vec(),
            },
        )
        .unwrap(),
        evidence: ArtifactChecksumRecord::from_bytes(b"separately retained readback declaration"),
    })
    .unwrap()
}

fn associate_original_chunks(
    request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
    attempts: &AttemptJournalGuard<'_>,
) {
    let before = fs::read(attempts.path()).unwrap();
    for chunk in [vec![42; 2], vec![0; 2]] {
        let response = readback_response(request, &chunk);
        let view = validate_response(request, attempts.record().unwrap(), &response).unwrap();
        assert_eq!(view.reply().chunk(), chunk);
        assert_eq!(view.matches_original_chunk(), chunk == [42; 2]);
        assert_eq!(fs::read(attempts.path()).unwrap(), before);
        assert!(!attempts.record().unwrap().view().applied);
    }
}

pub fn retained_readback<'layout>(
    plan: &OperationPlanRecord,
    upload: &IcSnapshotUploadRequest<'_>,
    layout: &'layout BackupLayoutGuard,
    mut attempts: AttemptJournalGuard<'layout>,
) -> AttemptJournalGuard<'layout> {
    let IcSnapshotUploadKind::Data {
        snapshot_id,
        source_kind,
        ..
    } = upload.kind()
    else {
        panic!("original data upload")
    };
    let original = IcSnapshotMetadataRequest::new(upload.target(), snapshot_id).unwrap();
    let mut values = upload.source().metadata().clone();
    // Independently declared original destination metadata; this is no allocation proof.
    values.source =
        Some(ic_management_canister_types::SnapshotSource::MetadataUpload(candid::Reserved));
    let raw = candid::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&original, &raw).unwrap();
    let read = IcSnapshotDataRequest::new(&metadata, source_kind.clone()).unwrap();
    let metadata_path = layout.root().join("original-upload-data-metadata.json");
    let read_path = layout.root().join("original-upload-data-read.json");
    create_json_durable(&metadata_path, &raw).unwrap();
    create_json_durable(&read_path, &read.arguments()).unwrap();
    let metadata_bytes = fs::read(&metadata_path).unwrap();
    let read_bytes = fs::read(&read_path).unwrap();
    let plan_bytes = fs::read(layout.root().join("operation-plan.json")).unwrap();
    assert_eq!(
        attempts
            .reserve_observation(1, read.digest().hash())
            .unwrap(),
        2
    );
    let before = fs::read(attempts.path()).unwrap();
    let request = IcSnapshotUploadDataObservationRequest::new(
        plan,
        7,
        attempts.record().unwrap(),
        upload,
        &read,
    )
    .unwrap();
    let mut provider = RefusingProvider { calls: 0 };
    assert_eq!(
        provider.observe_upload_data(&request).unwrap_err(),
        IcObservationProviderError::Indeterminate
    );
    assert_eq!(fs::read(attempts.path()).unwrap(), before);
    associate_original_chunks(&request, &attempts);
    drop(attempts);

    let raw: Vec<u8> = read_json(&metadata_path, MAX_IC_SNAPSHOT_METADATA_BYTES as u64).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&original, &raw).unwrap();
    let read = IcSnapshotDataRequest::new(&metadata, source_kind.clone()).unwrap();
    let original_arguments: Vec<u8> = read_json(&read_path, 8192).unwrap();
    assert_eq!(read.arguments(), original_arguments);
    let mut attempts =
        AttemptJournalGuard::open(layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let request = IcSnapshotUploadDataObservationRequest::new(
        plan,
        7,
        attempts.record().unwrap(),
        upload,
        &read,
    )
    .unwrap();
    associate_original_chunks(&request, &attempts);
    let progress = attempts.record().unwrap().view();
    assert_eq!(
        (progress.pending_mutation, progress.pending_observation),
        (Some(1), Some(2))
    );
    assert_eq!(
        (
            progress.mutations_remaining,
            progress.observations_remaining
        ),
        (0, 0)
    );
    assert!(!progress.applied);
    assert!(attempts.reserve_mutation().is_err());
    assert!(
        attempts
            .reserve_observation(1, read.digest().hash())
            .is_err()
    );
    assert_eq!(fs::read(attempts.path()).unwrap(), before);
    assert_eq!(fs::read(&metadata_path).unwrap(), metadata_bytes);
    assert_eq!(fs::read(&read_path).unwrap(), read_bytes);
    assert_eq!(
        fs::read(layout.root().join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    assert_eq!(provider.calls, 1);
    attempts
}

// This is synthetic qualified evidence for local persistence only, not IC behavior.
pub fn retained_settled_readback(
    plan: &OperationPlanRecord,
    upload: &IcSnapshotUploadRequest<'_>,
    layout: &BackupLayoutGuard,
    mut attempts: AttemptJournalGuard<'_>,
) {
    let IcSnapshotUploadKind::Data {
        snapshot_id,
        source_kind,
        ..
    } = upload.kind()
    else {
        panic!("original data upload")
    };
    let original = IcSnapshotMetadataRequest::new(upload.target(), snapshot_id).unwrap();
    let raw: Vec<u8> = read_json(
        &layout.root().join("original-upload-data-metadata.json"),
        MAX_IC_SNAPSHOT_METADATA_BYTES as u64,
    )
    .unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&original, &raw).unwrap();
    let read = IcSnapshotDataRequest::new(&metadata, source_kind.clone()).unwrap();
    let request = IcSnapshotUploadDataObservationRequest::new(
        plan,
        7,
        attempts.record().unwrap(),
        upload,
        &read,
    )
    .unwrap();
    let response = readback_response(&request, &[42; 2]);
    let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification challenge fixture");
    let associated = validate_response(&request, attempts.record().unwrap(), &response).unwrap();
    let actual = IcSnapshotUploadDataSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: challenge.clone(),
        readback: associated.reply().digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution: IcSnapshotUploadDataAttribution::Unresolved {
            uncertainty: ArtifactChecksumRecord::from_bytes(
                b"settled uncertainty fixture, not lost reply",
            ),
        },
        evidence: ArtifactChecksumRecord::from_bytes(
            b"retained complete qualification evidence fixture",
        ),
    };
    let before = fs::read(attempts.path()).unwrap();
    let view = validate_settlement(
        &request,
        attempts.record().unwrap(),
        &response,
        &challenge,
        &actual,
    )
    .unwrap();
    assert_eq!(view.outcome(), ObservationOutcomeRecord::Uncertain);
    assert_eq!(fs::read(attempts.path()).unwrap(), before);
    // Only the explicit integration-owned transition changes the existing journal.
    attempts
        .record_observation(ObservationReceiptRequest {
            attempt: actual.observation_attempt,
            request: read.digest().hash().into(),
            outcome: view.outcome(),
            evidence: actual.evidence.hash().into(),
        })
        .unwrap();
    let path = attempts.path();
    let retained = fs::read(&path).unwrap();
    assert_ne!(retained, before);
    drop(attempts);
    let mut reopened =
        AttemptJournalGuard::open(layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let current = reopened.record().unwrap().view();
    assert_eq!(current.pending_mutation, Some(1));
    assert_eq!(current.pending_observation, None);
    assert_eq!((current.mutations_used, current.observations_used), (1, 1));
    assert_eq!(
        (current.mutations_remaining, current.observations_remaining),
        (0, 0)
    );
    assert!(!current.applied);
    assert!(
        validate_settlement(
            &request,
            reopened.record().unwrap(),
            &response,
            &challenge,
            &actual
        )
        .is_err()
    );
    assert!(reopened.reserve_mutation().is_err());
    assert!(
        reopened
            .reserve_observation(1, read.digest().hash())
            .is_err()
    );
    assert_eq!(fs::read(path).unwrap(), retained);
}
