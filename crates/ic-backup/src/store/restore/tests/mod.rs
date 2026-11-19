//! Restore-specific source/approval/geometry regressions; call mechanics have one shared test owner.

pub(in crate::store::restore) mod fixtures;
mod persistence_tests;
mod plan_tests;
mod transfer_tests;

use super::*;

#[test]
fn original_restore_intent_has_an_independent_encoding_and_domain_vector() {
    let bytes = include_bytes!("../restore-intent.json");
    let plan = RestorePlan::decode(bytes).unwrap();
    assert_eq!(plan.encoded(), bytes);
    assert_eq!(
        plan.digest().to_string(),
        include_str!("../restore-intent.sha256").trim()
    );
    assert_ne!(plan.digest(), BackupDigest::compute(bytes));
}
