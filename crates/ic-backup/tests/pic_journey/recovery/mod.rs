//! Reserved real recovery under exclusive simulator fixture custody.

mod lifecycle;
pub(super) use lifecycle::lifecycle;

use super::backend::Backend;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptJournalRecordError, ObservationReceiptRequest},
        ic_observation::{
            IcCaptureAttribution, IcCaptureSettlement, IcObservationRequest, IcObservationResponse,
            IcObservationResponseInput,
        },
        ic_request::IcManagementRequestRecord,
        ic_snapshot_reply::IcSnapshotReply,
    },
    ops::persistence::{AttemptJournalError, AttemptJournalGuard},
    policy::ic_observation::validate_capture_settlement,
};
use std::fs;

pub(super) fn capture(
    backend: &mut Backend,
    index: u64,
    mutation: &IcManagementRequestRecord,
    list: &IcManagementRequestRecord,
    baseline: &IcSnapshotReply<'_>,
) -> (Vec<u8>, u64, u64) {
    let plan = backend.retained_plan(index);
    let layout = backend.layout(index);
    let mut journal =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(index).unwrap()).unwrap();
    assert!(matches!(
        journal.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { .. }
        ))
    ));
    assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
    fs::write(
        layout.root().join("observation-arguments.candid"),
        list.arguments(),
    )
    .unwrap();
    journal
        .reserve_observation(1, list.digest().hash())
        .unwrap();
    let request =
        IcObservationRequest::new(&plan, index, journal.record().unwrap(), mutation, list).unwrap();
    let raw = backend.management(list.method().name(), list.arguments());
    fs::write(layout.root().join("observation-reply.candid"), &raw).unwrap();
    let inventory = IcSnapshotReply::decode(list, &raw).unwrap();
    // This exact instance has one fixture owner and exactly one submitted capture.
    // No other writer, delete, or competing capture is permitted by this driver.
    // The ID is selected from that authenticated isolated history, not cardinality
    // alone, and the deliberately discarded reply is not read for settlement.
    let snapshot = &inventory.snapshots()[0];
    let evidence = backend.recovery_evidence(
        &plan,
        index,
        "original capture; no competing writer or deletion",
    );
    let response = IcObservationResponse::new(IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: list.digest(),
        context: plan.context().clone(),
        target: backend.target.to_text(),
        reply: raw,
        evidence: evidence.clone(),
    })
    .unwrap();
    let challenge = ArtifactChecksumRecord::from_bytes(
        format!("fresh capture recovery {}:{index}", plan.digest().hash()).as_bytes(),
    );
    let settlement = IcCaptureSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: challenge.clone(),
        baseline: baseline.digest(),
        inventory: inventory.digest(),
        observation_evidence: evidence.clone(),
        attribution: IcCaptureAttribution::Applied {
            snapshot_id: snapshot.id().to_vec(),
            attribution: evidence.clone(),
        },
        evidence,
    };
    let view = validate_capture_settlement(
        &request,
        journal.record().unwrap(),
        baseline,
        &response,
        &challenge,
        &settlement,
    )
    .unwrap();
    let captured = view.captured_snapshot().unwrap();
    let result = (
        captured.id().to_vec(),
        captured.taken_at_timestamp(),
        captured.total_size(),
    );
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: request.observation_attempt(),
            request: list.digest().hash().into(),
            outcome: view.outcome(),
            evidence: settlement.evidence.hash().into(),
        })
        .unwrap();
    assert_eq!(journal.record().unwrap().view().observations_remaining, 0);
    assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
    result
}

pub(super) fn allocation(
    backend: &mut Backend,
    index: u64,
    mutation: &ic_backup::model::ic_snapshot_upload::IcSnapshotUploadRequest<'_>,
    list: &IcManagementRequestRecord,
    baseline: &IcSnapshotReply<'_>,
) -> Vec<u8> {
    use ic_backup::{
        model::ic_snapshot_upload_observation::{
            IcSnapshotUploadAttribution, IcSnapshotUploadObservationRequest,
            IcSnapshotUploadSettlement,
        },
        policy::ic_snapshot_upload_observation::validate_settlement,
    };
    let plan = backend.retained_plan(index);
    let layout = backend.layout(index);
    let mut journal =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(index).unwrap()).unwrap();
    assert!(matches!(
        journal.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { .. }
        ))
    ));
    fs::write(
        layout.root().join("observation-arguments.candid"),
        list.arguments(),
    )
    .unwrap();
    journal
        .reserve_observation(1, list.digest().hash())
        .unwrap();
    let request = IcSnapshotUploadObservationRequest::new(
        &plan,
        index,
        journal.record().unwrap(),
        mutation,
        list,
    )
    .unwrap();
    let raw = backend.management(list.method().name(), list.arguments());
    fs::write(layout.root().join("observation-reply.candid"), &raw).unwrap();
    let inventory = IcSnapshotReply::decode(list, &raw).unwrap();
    // Independently controlled completed original ingress, exclusive allocation and
    // no competing writer/delete in this private instance qualify the new identity.
    let allocated = inventory
        .snapshots()
        .iter()
        .find(|snapshot| {
            !baseline
                .snapshots()
                .iter()
                .any(|original| original.id() == snapshot.id())
        })
        .unwrap();
    let evidence = backend.recovery_evidence(
        &plan,
        index,
        "original allocation; no competing allocator or deletion",
    );
    let response = IcObservationResponse::new(IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: list.digest(),
        context: plan.context().clone(),
        target: backend.target.to_text(),
        reply: raw,
        evidence: evidence.clone(),
    })
    .unwrap();
    let challenge = ArtifactChecksumRecord::from_bytes(
        format!("fresh allocation recovery {}:{index}", plan.digest().hash()).as_bytes(),
    );
    let settlement = IcSnapshotUploadSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: challenge.clone(),
        baseline: baseline.digest(),
        inventory: inventory.digest(),
        observation_evidence: evidence.clone(),
        attribution: IcSnapshotUploadAttribution::Applied {
            snapshot_id: allocated.id().to_vec(),
            attribution: evidence.clone(),
        },
        evidence,
    };
    let view = validate_settlement(
        &request,
        journal.record().unwrap(),
        baseline,
        &response,
        &challenge,
        &settlement,
    )
    .unwrap();
    let result = view.allocated_snapshot().unwrap().id().to_vec();
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: request.observation_attempt(),
            request: list.digest().hash().into(),
            outcome: view.outcome(),
            evidence: settlement.evidence.hash().into(),
        })
        .unwrap();
    assert_eq!(journal.record().unwrap().view().observations_remaining, 0);
    assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
    result
}

pub(super) fn data(
    backend: &mut Backend,
    index: u64,
    mutation: &ic_backup::model::ic_snapshot_upload::IcSnapshotUploadRequest<'_>,
    read: &ic_backup::model::ic_snapshot_data::IcSnapshotDataRequest<'_>,
) {
    use ic_backup::{
        model::{
            ic_snapshot_data::IcSnapshotDataReply,
            ic_snapshot_upload_data_observation::{
                IcSnapshotUploadDataAttribution, IcSnapshotUploadDataObservationRequest,
                IcSnapshotUploadDataObservationResponse, IcSnapshotUploadDataSettlement,
            },
        },
        policy::ic_snapshot_upload_data_observation::validate_settlement,
    };
    let plan = backend.retained_plan(index);
    let layout = backend.layout(index);
    let mut journal =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(index).unwrap()).unwrap();
    assert!(matches!(
        journal.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { .. }
        ))
    ));
    fs::write(
        layout.root().join("observation-arguments.candid"),
        read.arguments(),
    )
    .unwrap();
    journal
        .reserve_observation(1, read.digest().hash())
        .unwrap();
    let request = IcSnapshotUploadDataObservationRequest::new(
        &plan,
        index,
        journal.record().unwrap(),
        mutation,
        read,
    )
    .unwrap();
    let raw = backend.management(read.method(), read.arguments());
    fs::write(layout.root().join("observation-reply.candid"), &raw).unwrap();
    let reply = IcSnapshotDataReply::decode(read, &raw).unwrap();
    let evidence = backend.recovery_evidence(
        &plan,
        index,
        "original extent; fresh allocation and no competing writer",
    );
    let response = IcSnapshotUploadDataObservationResponse::new(IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: read.digest(),
        context: plan.context().clone(),
        target: backend.target.to_text(),
        reply: raw,
        evidence: evidence.clone(),
    })
    .unwrap();
    let challenge = ArtifactChecksumRecord::from_bytes(
        format!("fresh extent recovery {}:{index}", plan.digest().hash()).as_bytes(),
    );
    let settlement = IcSnapshotUploadDataSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: challenge.clone(),
        readback: reply.digest(),
        observation_evidence: evidence.clone(),
        attribution: IcSnapshotUploadDataAttribution::Applied {
            attribution: evidence.clone(),
        },
        evidence,
    };
    let view = validate_settlement(
        &request,
        journal.record().unwrap(),
        &response,
        &challenge,
        &settlement,
    )
    .unwrap();
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: request.observation_attempt(),
            request: read.digest().hash().into(),
            outcome: view.outcome(),
            evidence: settlement.evidence.hash().into(),
        })
        .unwrap();
    assert_eq!(journal.record().unwrap().view().observations_remaining, 0);
    assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
}
