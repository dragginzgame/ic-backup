use super::*;
use crate::{
    model::attempt_journal::{
        MutationOutcomeRecord, MutationReceiptRequest, ObservationOutcomeRecord,
        ObservationReceiptRequest,
    },
    test_support::{
        execution_settlement::{applied, journals, record},
        membership::{hash, plan},
    },
};
use serde_json::json;

#[test]
fn complete_applied_evidence_admits_in_any_input_order_without_refunds() {
    let plan = plan();
    let mut journals = applied(&plan);
    let record = record(&plan, &journals);
    let before = journals.clone();
    let view = validate(&plan, &journals.iter().collect::<Vec<_>>(), &record).unwrap();
    assert_eq!(view.applied_operations, plan.operations().len());
    assert_eq!(view.attempts.mutations_used, 2);
    assert_eq!(view.attempts.mutations_remaining, 1);
    assert_eq!(journals, before);
    journals.reverse();
    assert_eq!(
        validate(&plan, &journals.iter().collect::<Vec<_>>(), &record).unwrap(),
        view
    );
}
#[test]
fn unused_pending_lost_observation_uncertain_and_not_applied_are_not_settlement() {
    let plan = plan();
    for phase in 0..5 {
        let mut journals = journals(&plan);
        if phase > 0 {
            let attempt = journals[0].reserve_mutation().unwrap();
            if phase == 4 {
                let request = journals[0].authority().binding().request().into();
                journals[0]
                    .record_mutation(MutationReceiptRequest {
                        attempt,
                        request,
                        outcome: MutationOutcomeRecord::NotApplied,
                        evidence: hash("12").hash().into(),
                    })
                    .unwrap();
            } else if phase >= 2 {
                let observation = journals[0]
                    .reserve_observation(attempt, hash("34").hash())
                    .unwrap();
                if phase == 3 {
                    journals[0]
                        .record_observation(ObservationReceiptRequest {
                            attempt: observation,
                            request: hash("34").hash().into(),
                            outcome: ObservationOutcomeRecord::Uncertain,
                            evidence: hash("12").hash().into(),
                        })
                        .unwrap();
                }
            }
        }
        let record = record(&plan, &journals);
        let before = journals.clone();
        assert_eq!(
            validate(&plan, &journals.iter().collect::<Vec<_>>(), &record).unwrap_err(),
            ExecutionSettlementPolicyError::UnsettledOperation(0)
        );
        assert_eq!(journals, before);
    }
}
#[test]
fn canonical_progress_owner_rejects_missing_duplicate_changed_and_premature_evidence() {
    let plan = plan();
    let journals = applied(&plan);
    let record = record(&plan, &journals);
    assert!(matches!(
        validate(&plan, &[&journals[0]], &record),
        Err(ExecutionSettlementPolicyError::Progress(
            ExecutionProgressError::MissingJournal(7)
        ))
    ));
    assert!(matches!(
        validate(&plan, &[&journals[0], &journals[0]], &record),
        Err(ExecutionSettlementPolicyError::Progress(
            ExecutionProgressError::DuplicateJournal(0)
        ))
    ));
    let mut changed = serde_json::to_value(&journals[0]).unwrap();
    changed["authority"]["budget"]["mutations"] = json!(2);
    let changed = serde_json::from_value(changed).unwrap();
    assert!(matches!(
        validate(&plan, &[&changed, &journals[1]], &record),
        Err(ExecutionSettlementPolicyError::Progress(
            ExecutionProgressError::AuthorityMismatch(0)
        ))
    ));
    let mut pending = self::journals(&plan);
    let attempt = pending[1].reserve_mutation().unwrap();
    let request = pending[1].authority().binding().request().into();
    pending[1]
        .record_mutation(MutationReceiptRequest {
            attempt,
            request,
            outcome: MutationOutcomeRecord::Applied,
            evidence: hash("12").hash().into(),
        })
        .unwrap();
    let premature = self::record(&plan, &pending);
    assert!(matches!(
        validate(&plan, &pending.iter().collect::<Vec<_>>(), &premature),
        Err(ExecutionSettlementPolicyError::Progress(
            ExecutionProgressError::PrematureAttempt {
                operation_sequence: 7,
                prerequisite: 0
            }
        ))
    ));
}
#[test]
fn checkpoint_requires_exact_plan_row_set_and_receipt_history_even_for_identical_views() {
    let plan = plan();
    let journals = applied(&plan);
    let record = record(&plan, &journals);
    let mut changed = serde_json::to_value(&record).unwrap();
    changed["plan_intent"] = json!(hash("90"));
    let changed = serde_json::from_value(changed).unwrap();
    assert_eq!(
        validate(&plan, &journals.iter().collect::<Vec<_>>(), &changed).unwrap_err(),
        ExecutionSettlementPolicyError::IntentMismatch
    );
    let incomplete =
        ExecutionSettlementRecord::new(plan.digest(), record.journals()[..1].to_vec()).unwrap();
    assert_eq!(
        validate(&plan, &journals.iter().collect::<Vec<_>>(), &incomplete).unwrap_err(),
        ExecutionSettlementPolicyError::JournalSetMismatch
    );
    let mut changed = serde_json::to_value(&journals[0]).unwrap();
    changed["events"][1]["evidence"] = json!(hash("34").hash());
    let changed: AttemptJournalRecord = serde_json::from_value(changed).unwrap();
    assert_eq!(changed.view(), journals[0].view());
    assert_eq!(
        validate(&plan, &[&changed, &journals[1]], &record).unwrap_err(),
        ExecutionSettlementPolicyError::HistoryMismatch(0)
    );
}
