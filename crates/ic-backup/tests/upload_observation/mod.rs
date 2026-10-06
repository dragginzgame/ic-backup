//! Refusal and retained passive inventories; no management effect simulation.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        ic_observation::{IcObservationResponse, IcObservationResponseInput},
        ic_request::{
            IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord,
            MAX_IC_REQUEST_RECORD_BYTES,
        },
        ic_snapshot_reply::IcSnapshotReply,
        ic_snapshot_upload::IcSnapshotUploadRequest,
        ic_snapshot_upload_observation::{
            IcSnapshotUploadAttribution, IcSnapshotUploadObservationRequest,
            IcSnapshotUploadSettlement,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_json_durable, read_json},
    policy::{
        ic_observation::IcObservationReplyView,
        ic_snapshot_upload_observation::{validate_response, validate_settlement},
    },
    ports::{
        ic_observation::IcObservationProviderError,
        ic_snapshot_upload_observation::IcSnapshotUploadObservationProvider,
    },
};
use std::fs;

struct RefusingProvider {
    calls: usize,
}
impl IcSnapshotUploadObservationProvider for RefusingProvider {
    fn observe_upload(
        &mut self,
        _request: &IcSnapshotUploadObservationRequest<'_, '_>,
    ) -> Result<IcObservationResponse, IcObservationProviderError> {
        self.calls += 1;
        assert_eq!(self.calls, 1, "no hidden retry or local reopen dispatch");
        Err(IcObservationProviderError::Indeterminate)
    }
}

fn passive_inventory(
    request: &IcSnapshotUploadObservationRequest<'_, '_>,
    count: u8,
) -> IcObservationResponse {
    IcObservationResponse::new(IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: request.payload().digest(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply: candid::encode_one(
            (0..count)
                .map(|id| ic_management_canister_types::Snapshot {
                    id: vec![id, 0, 255],
                    taken_at_timestamp: 42,
                    total_size: u64::MAX,
                })
                .collect::<Vec<_>>(),
        )
        .unwrap(),
        evidence: ArtifactChecksumRecord::from_bytes(
            b"independently retained passive inventory declaration",
        ),
    })
    .unwrap()
}

pub fn retained_inventory_observation(
    plan: &OperationPlanRecord,
    upload: &IcSnapshotUploadRequest<'_>,
    layout: &BackupLayoutGuard,
    mut attempts: AttemptJournalGuard<'_>,
) {
    let path = layout.root().join("original-upload-list.json");
    let list: IcManagementRequestRecord = read_json(&path, MAX_IC_REQUEST_RECORD_BYTES).unwrap();
    let payload_bytes = fs::read(&path).unwrap();
    let plan_bytes = fs::read(layout.root().join("operation-plan.json")).unwrap();
    assert_eq!(
        attempts
            .reserve_observation(1, list.digest().hash())
            .unwrap(),
        2
    );
    let before = fs::read(attempts.path()).unwrap();
    let request =
        IcSnapshotUploadObservationRequest::new(plan, 7, attempts.record().unwrap(), upload, &list)
            .unwrap();
    let mut provider = RefusingProvider { calls: 0 };
    assert_eq!(
        provider.observe_upload(&request).unwrap_err(),
        IcObservationProviderError::Indeterminate
    );
    assert_eq!(fs::read(attempts.path()).unwrap(), before);
    // These are separately retained declarations, not replies fabricated by a provider.
    for count in 0..=2 {
        let response = passive_inventory(&request, count);
        let view = validate_response(&request, attempts.record().unwrap(), &response).unwrap();
        let IcObservationReplyView::Inventory(reply) = view.reply() else {
            panic!("list evidence")
        };
        assert_eq!(reply.snapshots().len(), usize::from(count));
        assert_eq!(
            reply.payload_checksum(),
            &ArtifactChecksumRecord::from_bytes(&response.input().reply)
        );
        assert_eq!(fs::read(attempts.path()).unwrap(), before);
        assert!(!attempts.record().unwrap().view().applied);
    }
    drop(attempts);
    let list: IcManagementRequestRecord = read_json(&path, MAX_IC_REQUEST_RECORD_BYTES).unwrap();
    let mut attempts =
        AttemptJournalGuard::open(layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let request =
        IcSnapshotUploadObservationRequest::new(plan, 7, attempts.record().unwrap(), upload, &list)
            .unwrap();
    let response = passive_inventory(&request, 1);
    validate_response(&request, attempts.record().unwrap(), &response).unwrap();
    let progress = attempts.record().unwrap().view();
    assert_eq!(progress.pending_mutation, Some(1));
    assert_eq!(progress.pending_observation, Some(2));
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
            .reserve_observation(1, list.digest().hash())
            .is_err()
    );
    assert_eq!(fs::read(attempts.path()).unwrap(), before);
    assert_eq!(fs::read(&path).unwrap(), payload_bytes);
    assert_eq!(
        fs::read(layout.root().join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    assert_eq!(provider.calls, 1);
    retained_settlement(plan, upload, layout, attempts);
}

/// Retain native declarations before the original mutation; this proves no IC chronology.
pub fn retain_original_inventory(layout: &BackupLayoutGuard, upload: &IcSnapshotUploadRequest<'_>) {
    let list = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::ListCanisterSnapshots,
        target: upload.target().into(),
        snapshot_id: None,
    })
    .unwrap();
    create_json_durable(&layout.root().join("original-upload-list.json"), &list).unwrap();
    let raw = candid::encode_one(Vec::<ic_management_canister_types::Snapshot>::new()).unwrap();
    create_json_durable(&layout.root().join("original-upload-baseline.json"), &raw).unwrap();
}

fn retained_settlement(
    plan: &OperationPlanRecord,
    upload: &IcSnapshotUploadRequest<'_>,
    layout: &BackupLayoutGuard,
    mut attempts: AttemptJournalGuard<'_>,
) {
    let list_path = layout.root().join("original-upload-list.json");
    let list_bytes = fs::read(&list_path).unwrap();
    let list: IcManagementRequestRecord =
        read_json(&list_path, MAX_IC_REQUEST_RECORD_BYTES).unwrap();
    let baseline_path = layout.root().join("original-upload-baseline.json");
    let baseline_bytes = fs::read(&baseline_path).unwrap();
    let raw: Vec<u8> = read_json(&baseline_path, 1 << 20).unwrap();
    let baseline = IcSnapshotReply::decode(&list, &raw).unwrap();
    let original = attempts.record().unwrap().clone();
    let request =
        IcSnapshotUploadObservationRequest::new(plan, 7, &original, upload, &list).unwrap();
    let response = passive_inventory(&request, 2);
    let challenge = ArtifactChecksumRecord::from_bytes(b"fresh local qualification challenge");
    let settlement = IcSnapshotUploadSettlement {
        authority: request.authority().digest(),
        mutation_attempt: 1,
        observation_attempt: 2,
        challenge: challenge.clone(),
        baseline: baseline.digest(),
        inventory: IcSnapshotReply::decode(&list, &response.input().reply)
            .unwrap()
            .digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution: IcSnapshotUploadAttribution::Unresolved {
            uncertainty: ArtifactChecksumRecord::from_bytes(b"synthetic settled uncertainty"),
        },
        evidence: ArtifactChecksumRecord::from_bytes(
            b"local transition qualification only; no IC effect proof",
        ),
    };
    let before = fs::read(attempts.path()).unwrap();
    let view = validate_settlement(
        &request,
        &original,
        &baseline,
        &response,
        &challenge,
        &settlement,
    )
    .unwrap();
    assert_eq!(view.outcome(), ObservationOutcomeRecord::Uncertain);
    assert!(view.allocated_snapshot().is_none());
    assert_eq!(fs::read(attempts.path()).unwrap(), before);
    attempts
        .record_observation(ObservationReceiptRequest {
            attempt: 2,
            request: list.digest().hash().into(),
            outcome: view.outcome(),
            evidence: settlement.evidence.hash().into(),
        })
        .unwrap();
    let settled = fs::read(attempts.path()).unwrap();
    assert_ne!(settled, before);
    drop(attempts);
    let mut reopened =
        AttemptJournalGuard::open(layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let progress = reopened.record().unwrap().view();
    assert_eq!(progress.pending_mutation, Some(1));
    assert_eq!(progress.pending_observation, None);
    assert_eq!(
        (progress.mutations_used, progress.observations_used),
        (1, 1)
    );
    assert_eq!(
        (
            progress.mutations_remaining,
            progress.observations_remaining
        ),
        (0, 0)
    );
    assert!(!progress.applied);
    assert!(
        validate_settlement(
            &request,
            reopened.record().unwrap(),
            &baseline,
            &response,
            &challenge,
            &settlement
        )
        .is_err()
    );
    assert!(reopened.reserve_mutation().is_err());
    assert!(
        reopened
            .reserve_observation(1, list.digest().hash())
            .is_err()
    );
    assert_eq!(fs::read(reopened.path()).unwrap(), settled);
    assert_eq!(fs::read(&list_path).unwrap(), list_bytes);
    assert_eq!(fs::read(&baseline_path).unwrap(), baseline_bytes);
}
