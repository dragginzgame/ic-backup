//! Refusal and retained passive inventories; no management effect simulation.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_observation::{IcObservationResponse, IcObservationResponseInput},
        ic_request::{
            IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord,
            MAX_IC_REQUEST_RECORD_BYTES,
        },
        ic_snapshot_upload::IcSnapshotUploadRequest,
        ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest,
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_json_durable, read_json},
    policy::{
        ic_observation::IcObservationReplyView, ic_snapshot_upload_observation::validate_response,
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
    let list = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::ListCanisterSnapshots,
        target: upload.target().into(),
        snapshot_id: None,
    })
    .unwrap();
    let path = layout.root().join("original-upload-list.json");
    create_json_durable(&path, &list).unwrap();
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
}
