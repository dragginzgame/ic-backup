//! Explicit example inputs refuse malformed targets, absent trust and signer rebinding.
use super::*;
use ic_backup::model::operation_plan::{PlanContextRecord, PlanContextRequest};

#[test]
fn standalone_inputs_require_the_supplied_signer_context_target_and_trust() {
    let identity = ic_platform::agent::BasicIdentity::from_raw_key(&[9; 32]);
    let caller = ic_platform::agent::Identity::sender(&identity)
        .unwrap()
        .to_text();
    let mut config = Config {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: "ab".repeat(32),
            caller,
            release: "cd".repeat(32),
        })
        .unwrap(),
        target: "renrk-eyaaa-aaaaa-aaada-cai".into(),
        endpoint: "https://example.invalid".into(),
        key: vec![1],
        identity_seed: [9; 32],
    };
    // Construction is local. The authenticated consumer separately qualifies real trust.
    assert!(config.connection().is_ok());
    config.identity_seed = [7; 32];
    assert!(config.connection().is_err());
    config.identity_seed = [9; 32];
    config.key.clear();
    assert!(config.connection().is_err());
    config.key = vec![1];
    config.target = "not-a-principal".into();
    assert!(config.connection().is_err());
}
