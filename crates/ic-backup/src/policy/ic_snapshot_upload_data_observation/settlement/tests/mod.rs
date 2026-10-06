use super::*;
use crate::{
    model::attempt_journal::ObservationReceiptRequest,
    test_support::ic_snapshot_upload_data_observation::{input, with_original},
};
use ic_management_canister_types::SnapshotDataKind;

fn settlement(
    request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
    response: &IcSnapshotUploadDataObservationResponse,
    challenge: &ArtifactChecksumRecord,
    attribution: IcSnapshotUploadDataAttribution,
) -> IcSnapshotUploadDataSettlement {
    IcSnapshotUploadDataSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: challenge.clone(),
        readback: crate::model::ic_snapshot_data::IcSnapshotDataReply::decode(
            request.payload(),
            &response.input().reply,
        )
        .unwrap()
        .digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution,
        evidence: ArtifactChecksumRecord::from_bytes(
            b"retained integration settlement declaration",
        ),
    }
}

fn attributions() -> [(IcSnapshotUploadDataAttribution, ObservationOutcomeRecord); 3] {
    let proof = ArtifactChecksumRecord::from_bytes(b"separately qualified evidence declaration");
    [
        (
            IcSnapshotUploadDataAttribution::Applied {
                attribution: proof.clone(),
            },
            ObservationOutcomeRecord::Applied,
        ),
        (
            IcSnapshotUploadDataAttribution::NotApplied {
                exclusion: proof.clone(),
            },
            ObservationOutcomeRecord::NotApplied,
        ),
        (
            IcSnapshotUploadDataAttribution::Unresolved { uncertainty: proof },
            ObservationOutcomeRecord::Uncertain,
        ),
    ]
}

#[test]
fn equal_zero_bytes_need_independent_claim_and_pure_views_leave_all_spending_pending() {
    for kind in [
        SnapshotDataKind::WasmModule { offset: 0, size: 2 },
        SnapshotDataKind::WasmMemory { offset: 0, size: 2 },
        SnapshotDataKind::StableMemory { offset: 0, size: 2 },
    ] {
        with_original(kind, &[0; 2], |plan, upload, read, journal| {
            let before = journal.digest();
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[0; 2])).unwrap();
            let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification");
            for (attribution, outcome) in attributions() {
                let actual = settlement(&request, &response, &challenge, attribution);
                let view =
                    validate_settlement(&request, journal, &response, &challenge, &actual).unwrap();
                assert_eq!(view.outcome(), outcome);
                assert!(view.observation().matches_original_chunk());
                assert!(std::ptr::eq(view.settlement(), &raw const actual));
                assert!(std::ptr::eq(
                    view.observation().response(),
                    &raw const response
                ));
                assert_eq!(journal.digest(), before);
                assert_eq!(journal.view().pending_mutation, Some(1));
                assert_eq!(journal.view().pending_observation, Some(2));
                assert_eq!(journal.view().mutations_remaining, 0);
                assert_eq!(journal.view().observations_remaining, 0);
                assert!(!journal.view().applied);
                let mut retained = journal.clone();
                retained
                    .record_observation(ObservationReceiptRequest {
                        attempt: actual.observation_attempt,
                        request: read.digest().hash().into(),
                        outcome: view.outcome(),
                        evidence: actual.evidence.hash().into(),
                    })
                    .unwrap();
                assert_eq!(retained.view().pending_observation, None);
                assert_eq!(
                    retained.view().pending_mutation,
                    if outcome == ObservationOutcomeRecord::Uncertain {
                        Some(1)
                    } else {
                        None
                    }
                );
                assert_eq!(
                    retained.view().applied,
                    outcome == ObservationOutcomeRecord::Applied
                );
                assert_eq!(retained.view().mutations_used, 1);
                assert_eq!(retained.view().observations_used, 1);
                assert_eq!(retained.view().mutations_remaining, 0);
                assert_eq!(retained.view().observations_remaining, 0);
                assert!(retained.reserve_mutation().is_err());
                assert!(
                    retained
                        .reserve_observation(1, read.digest().hash())
                        .is_err()
                );
            }
        });
    }
}

#[test]
fn changed_destination_metadata_cannot_reuse_identical_read_arguments_and_raw_data() {
    with_original(
        SnapshotDataKind::WasmMemory { offset: 0, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[42; 2])).unwrap();
            let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification");
            let actual = settlement(&request, &response, &challenge, attributions()[0].0.clone());
            let mut values = read.metadata().metadata().clone();
            values.taken_at_timestamp ^= 1;
            let bytes = candid::encode_one(values).unwrap();
            let metadata = crate::model::ic_snapshot_metadata::IcSnapshotMetadataReply::decode(
                read.metadata().request(),
                &bytes,
            )
            .unwrap();
            let other_read = crate::model::ic_snapshot_data::IcSnapshotDataRequest::new(
                &metadata,
                read.kind().clone(),
            )
            .unwrap();
            assert_eq!(other_read.digest(), read.digest());
            let other =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, &other_read)
                    .unwrap();
            assert!(matches!(
                validate_settlement(&other, journal, &response, &challenge, &actual),
                Err(IcSnapshotUploadDataSettlementError::ReadbackMismatch)
            ));
        },
    );
}

#[test]
fn known_empty_chunk_uses_existing_hash_decoder_and_no_new_allowance() {
    with_original(
        SnapshotDataKind::WasmChunk {
            hash: crate::test_support::ic_snapshot_upload::unhex(
                ArtifactChecksumRecord::from_bytes(&[]).hash(),
            ),
        },
        &[],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[])).unwrap();
            let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification");
            let actual = settlement(&request, &response, &challenge, attributions()[0].0.clone());
            let view =
                validate_settlement(&request, journal, &response, &challenge, &actual).unwrap();
            assert_eq!(view.outcome(), ObservationOutcomeRecord::Applied);
            assert_eq!(view.observation().reply().chunk(), &[] as &[u8]);
            assert_eq!(journal.view().pending_observation, Some(2));
            assert_eq!(journal.view().observations_remaining, 0);
        },
    );
}

#[test]
fn different_bytes_reject_applied_but_never_infer_nonapplication_or_uncertainty() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[0; 2])).unwrap();
            let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification");
            for (attribution, outcome) in attributions() {
                let actual = settlement(&request, &response, &challenge, attribution);
                let result = validate_settlement(&request, journal, &response, &challenge, &actual);
                if outcome == ObservationOutcomeRecord::Applied {
                    assert!(matches!(
                        result,
                        Err(IcSnapshotUploadDataSettlementError::AppliedBytesMismatch)
                    ));
                } else {
                    let view = result.unwrap();
                    assert_eq!(view.outcome(), outcome);
                    assert!(!view.observation().matches_original_chunk());
                }
            }
        },
    );
}

#[test]
fn exact_authority_both_attempts_challenge_readback_and_observation_evidence_are_required() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[42; 2])).unwrap();
            let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification");
            let base = settlement(&request, &response, &challenge, attributions()[0].0.clone());
            let other = ArtifactChecksumRecord::from_bytes(b"changed original");
            for change in 0..6 {
                let mut actual = base.clone();
                match change {
                    0 => actual.authority = other.clone(),
                    1 => actual.mutation_attempt = 0,
                    2 => actual.observation_attempt = 1025,
                    3 => actual.challenge = other.clone(),
                    4 => actual.readback = other.clone(),
                    _ => actual.observation_evidence = other.clone(),
                }
                let error = validate_settlement(&request, journal, &response, &challenge, &actual)
                    .unwrap_err();
                assert!(match change {
                    0 => matches!(
                        error,
                        IcSnapshotUploadDataSettlementError::AuthorityMismatch
                    ),
                    1 | 2 => matches!(error, IcSnapshotUploadDataSettlementError::AttemptMismatch),
                    3 => matches!(
                        error,
                        IcSnapshotUploadDataSettlementError::ChallengeMismatch
                    ),
                    4 => matches!(error, IcSnapshotUploadDataSettlementError::ReadbackMismatch),
                    _ => matches!(
                        error,
                        IcSnapshotUploadDataSettlementError::ObservationEvidenceMismatch
                    ),
                });
            }
        },
    );
}

#[test]
fn changed_raw_bytes_and_equal_bytes_with_different_provenance_cannot_rebind_settlement() {
    with_original(
        SnapshotDataKind::StableMemory { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[42; 2])).unwrap();
            let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification");
            let actual = settlement(&request, &response, &challenge, attributions()[2].0.clone());
            let other =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[0; 2])).unwrap();
            assert!(matches!(
                validate_settlement(&request, journal, &other, &challenge, &actual),
                Err(IcSnapshotUploadDataSettlementError::ReadbackMismatch)
            ));
            let mut fields = response.input().clone();
            fields.evidence = ArtifactChecksumRecord::from_bytes(b"different provider evidence");
            let other = IcSnapshotUploadDataObservationResponse::new(fields).unwrap();
            assert!(matches!(
                validate_settlement(&request, journal, &other, &challenge, &actual),
                Err(IcSnapshotUploadDataSettlementError::ObservationEvidenceMismatch)
            ));
        },
    );
}

#[test]
fn stale_or_resolved_reservations_and_lost_or_malformed_reads_stay_outside_settlement() {
    with_original(
        SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        &[42; 2],
        |plan, upload, read, journal| {
            let request =
                IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                    .unwrap();
            let response =
                IcSnapshotUploadDataObservationResponse::new(input(&request, &[42; 2])).unwrap();
            let challenge = ArtifactChecksumRecord::from_bytes(b"current qualification");
            let actual = settlement(&request, &response, &challenge, attributions()[2].0.clone());
            let mut settled = journal.clone();
            settled
                .record_observation(ObservationReceiptRequest {
                    attempt: 2,
                    request: read.digest().hash().into(),
                    outcome: ObservationOutcomeRecord::Uncertain,
                    evidence: actual.evidence.hash().into(),
                })
                .unwrap();
            for current in [
                &settled,
                &AttemptJournalRecord::new(journal.authority().clone()),
            ] {
                assert!(matches!(
                    validate_settlement(&request, current, &response, &challenge, &actual),
                    Err(IcSnapshotUploadDataSettlementError::Observation(_))
                ));
            }
            for reply in [vec![], b"lost or invalid reply".to_vec()] {
                let mut fields = response.input().clone();
                fields.reply = reply;
                let absent = IcSnapshotUploadDataObservationResponse::new(fields).unwrap();
                assert!(matches!(
                    validate_settlement(&request, journal, &absent, &challenge, &actual),
                    Err(IcSnapshotUploadDataSettlementError::Observation(_))
                ));
            }
            assert_eq!(journal.view().pending_observation, Some(2));
            assert_eq!(settled.view().pending_mutation, Some(1));
            assert_eq!(settled.view().observations_remaining, 0);
        },
    );
}
