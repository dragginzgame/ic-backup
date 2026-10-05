//! Passive native Applied journal fixtures, without effect or terminal qualification.

use crate::{
    model::{
        attempt_journal::{AttemptJournalRecord, MutationOutcomeRecord, MutationReceiptRequest},
        execution_settlement::{ExecutionSettlementJournalRecord, ExecutionSettlementRecord},
        operation_plan::OperationPlanRecord,
    },
    test_support::membership,
};

pub fn journals(plan: &OperationPlanRecord) -> Vec<AttemptJournalRecord> {
    plan.operations()
        .iter()
        .map(|operation| {
            AttemptJournalRecord::new(
                plan.attempt_authority(operation.operation_sequence())
                    .unwrap(),
            )
        })
        .collect()
}
pub fn applied(plan: &OperationPlanRecord) -> Vec<AttemptJournalRecord> {
    let mut records = journals(plan);
    for journal in &mut records {
        let attempt = journal.reserve_mutation().unwrap();
        journal
            .record_mutation(MutationReceiptRequest {
                attempt,
                request: journal.authority().binding().request().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: membership::hash("12").hash().into(),
            })
            .unwrap();
    }
    records
}
pub fn record(
    plan: &OperationPlanRecord,
    journals: &[AttemptJournalRecord],
) -> ExecutionSettlementRecord {
    ExecutionSettlementRecord::new(
        plan.digest(),
        journals
            .iter()
            .map(ExecutionSettlementJournalRecord::from_journal)
            .collect(),
    )
    .unwrap()
}
