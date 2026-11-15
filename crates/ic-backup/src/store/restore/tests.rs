//! Independent encoding/approval vectors; these passive records grant no source authority.
use super::*;

#[test]
fn canonical_plan_record_and_domain_commitment_are_frozen() {
    let bytes = include_bytes!("restore-plan.json");
    let record: RestorePlanRecord = serde_json::from_slice(bytes).unwrap();
    assert_eq!(serde_json::to_vec(&record).unwrap(), bytes);
    assert_eq!(
        plan_digest(bytes).to_string(),
        include_str!("restore-plan.sha256").trim()
    );
    assert_ne!(plan_digest(bytes), Sha256Digest::compute(bytes));
}

mod certificate_tests;
mod fixtures;
mod persistence_tests;
mod plan_tests;
mod preparation_tests;
mod request_tests;
mod status_tests;
mod transfer_tests;
