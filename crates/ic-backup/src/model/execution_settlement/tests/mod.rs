use super::*;
use crate::test_support::{
    execution_settlement::{applied, record},
    membership::{hash, plan},
};
use serde_json::json;

fn schema() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/execution-settlement.schema.json"
    ))
    .unwrap()
}
#[test]
fn independent_goldens_bind_every_history_event_and_checkpoint_identity() {
    let schema = schema();
    for golden in schema["x-history-goldens"].as_array().unwrap() {
        let journal: AttemptJournalRecord =
            serde_json::from_value(golden["record"].clone()).unwrap();
        assert_eq!(
            journal.authority().digest().hash(),
            golden["authority_sha256"].as_str().unwrap()
        );
        assert_eq!(journal.digest().hash(), golden["sha256"].as_str().unwrap());
    }
    let golden = &schema["x-checkpoint-golden"];
    let record: ExecutionSettlementRecord =
        serde_json::from_value(golden["record"].clone()).unwrap();
    assert_eq!(record.digest().hash(), golden["sha256"].as_str().unwrap());
    assert_eq!(record.journals()[1].operation_sequence(), u64::MAX);
}
#[test]
fn strict_v1_rows_reject_missing_unknown_invalid_and_duplicate_identities() {
    let plan = plan();
    let record = record(&plan, &applied(&plan));
    let value = serde_json::to_value(&record).unwrap();
    for field in ["version", "plan_intent", "journals"] {
        let mut changed = value.clone();
        changed.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<ExecutionSettlementRecord>(changed).is_err());
    }
    for (field, replacement) in [
        ("version", json!(2)),
        ("version", json!(null)),
        ("plan_intent", json!("invalid")),
        ("journals", json!([])),
    ] {
        let mut changed = value.clone();
        changed[field] = replacement;
        assert!(serde_json::from_value::<ExecutionSettlementRecord>(changed).is_err());
    }
    let mut changed = value.clone();
    changed["complete"] = json!(true);
    assert!(serde_json::from_value::<ExecutionSettlementRecord>(changed).is_err());
    let mut changed = value.clone();
    changed["journals"][0]["applied"] = json!(true);
    assert!(serde_json::from_value::<ExecutionSettlementRecord>(changed).is_err());
    let mut changed = value;
    changed["journals"][1]["operation_sequence"] = json!(0);
    changed["journals"][1]["history"] = json!(hash("90"));
    assert!(serde_json::from_value::<ExecutionSettlementRecord>(changed).is_err());
}
#[test]
fn canonical_rows_and_hashes_preserve_order_independent_identity() {
    let plan = plan();
    let journals = applied(&plan);
    let record = record(&plan, &journals);
    let mut value = serde_json::to_value(&record).unwrap();
    value["journals"].as_array_mut().unwrap().reverse();
    value["plan_intent"]["hash"] = json!(record.plan_intent().hash().to_uppercase());
    for row in value["journals"].as_array_mut().unwrap() {
        row["history"]["hash"] = json!(row["history"]["hash"].as_str().unwrap().to_uppercase());
    }
    let canonical: ExecutionSettlementRecord = serde_json::from_value(value).unwrap();
    assert_eq!(record, canonical);
    assert_eq!(record.digest(), canonical.digest());
    let mut changed = serde_json::to_value(record.clone()).unwrap();
    changed["plan_intent"] = json!(hash("90"));
    let changed: ExecutionSettlementRecord = serde_json::from_value(changed).unwrap();
    assert_ne!(record.digest(), changed.digest());
}
#[test]
fn exact_operation_ceiling_fits_io_and_one_more_row_rejects_before_admission() {
    let rows: Vec<_> = (0..MAX_EFFECT_OPERATIONS)
        .map(|index| ExecutionSettlementJournalRecord {
            operation_sequence: u64::MAX - index as u64,
            history: hash("12"),
        })
        .collect();
    let record = ExecutionSettlementRecord::new(hash("ab"), rows.clone()).unwrap();
    assert_eq!(record.journals().len(), MAX_EFFECT_OPERATIONS);
    assert!(
        serde_json::to_vec_pretty(&record).unwrap().len() as u64 <= MAX_EXECUTION_SETTLEMENT_BYTES
    );
    let mut excess = rows;
    excess.push(ExecutionSettlementJournalRecord {
        operation_sequence: 0,
        history: hash("12"),
    });
    assert_eq!(
        ExecutionSettlementRecord::new(hash("ab"), excess.clone()).unwrap_err(),
        ExecutionSettlementError::InvalidJournalCount
    );
    assert!(
        serde_json::from_value::<ExecutionSettlementRecord>(
            json!({"version":1,"plan_intent":hash("ab"),"journals":excess})
        )
        .is_err()
    );
}
#[test]
fn same_progress_with_changed_receipt_still_changes_history_fingerprint() {
    let plan = plan();
    let journals = applied(&plan);
    let journal = &journals[0];
    let mut value = serde_json::to_value(journal).unwrap();
    value["events"][1]["evidence"] = json!(hash("34").hash());
    let changed: AttemptJournalRecord = serde_json::from_value(value).unwrap();
    assert_eq!(journal.view(), changed.view());
    assert_ne!(journal.digest(), changed.digest());
}

#[test]
fn execution_settlement_bulk_authorities_preserve_full_original_scalar_binding() {
    let plan = plan();
    let authorities = plan.attempt_authorities().unwrap();
    assert_eq!(authorities.len(), plan.operations().len());
    for (authority, operation) in authorities.iter().zip(plan.operations()) {
        assert_eq!(
            authority,
            &plan
                .attempt_authority(operation.operation_sequence())
                .unwrap()
        );
        assert_eq!(authority.binding().intent(), plan.digest().hash());
        assert_eq!(authority.budget(), operation.budget());
    }
}
