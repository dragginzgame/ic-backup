use super::*;
use crate::test_support::{fence_acquisition::original, membership::hash};

#[test]
fn acknowledgement_association_preserves_pending_mutation_and_consumed_spending() {
    let original = original(true);
    let request = FenceAcquisitionRequest::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        &original.payload,
    )
    .unwrap();
    let acknowledgement = FenceAcquisitionAcknowledgement::new(
        request.authority().digest(),
        request.mutation_attempt(),
        hash("12"),
    )
    .unwrap();
    let before = original.journal.clone();
    let associated =
        validate_acknowledgement(&request, &original.journal, &acknowledgement).unwrap();
    assert_eq!(associated.evidence, hash("12"));
    assert_eq!(original.journal, before);
    assert_eq!(original.journal.view().pending_mutation, Some(1));
    assert!(!original.journal.view().applied);
    assert_eq!(original.journal.view().mutations_remaining, 0);
}

#[test]
fn changed_reply_authority_attempt_and_current_recovery_evidence_reject() {
    let mut original = original(false);
    let request = FenceAcquisitionRequest::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        &original.payload,
    )
    .unwrap();
    let acknowledgement = FenceAcquisitionAcknowledgement::new(
        request.authority().digest(),
        request.mutation_attempt(),
        hash("12"),
    )
    .unwrap();
    let mut changed = acknowledgement.clone();
    changed.authority = hash("90");
    assert!(matches!(
        validate_acknowledgement(&request, &original.journal, &changed),
        Err(FenceAcquisitionAcknowledgementError::AuthorityMismatch)
    ));
    let mut changed = acknowledgement.clone();
    changed.mutation_attempt = 2;
    assert!(matches!(
        validate_acknowledgement(&request, &original.journal, &changed),
        Err(FenceAcquisitionAcknowledgementError::AttemptMismatch)
    ));
    original
        .journal
        .reserve_observation(1, hash("34").hash())
        .unwrap();
    assert!(matches!(
        validate_acknowledgement(&request, &original.journal, &acknowledgement),
        Err(FenceAcquisitionAcknowledgementError::Reservation(
            FenceAcquisitionError::ObservationPending
        ))
    ));
}
