//! Typed parent-owned fixture claims; admission still hashes the original bytes.

use ic_backup::model::{artifacts::ArtifactChecksumRecord, operation_plan::PlanContextRecord};
use ic_platform::management_canister_types::CanisterStatusType;
use serde::Deserialize;

// Preserve exact textual comparison rather than normalizing claimed identities.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObservedContext {
    network: String,
    caller: String,
    release: String,
}
impl ObservedContext {
    pub(super) fn matches(&self, expected: &PlanContextRecord) -> bool {
        self.network == expected.network()
            && self.caller == expected.caller()
            && self.release == expected.release()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeclaredDigest {
    algorithm: String,
    hash: String,
}
impl DeclaredDigest {
    pub(super) fn matches(&self, expected: &ArtifactChecksumRecord) -> bool {
        self.algorithm == "sha256" && self.hash == expected.hash()
    }
}

#[derive(Deserialize)]
pub(super) struct AdmissionRecord {
    pub context: ObservedContext,
    pub target: String,
    pub controllers: Vec<String>,
    pub module_hash: String,
    pub status: CanisterStatusType,
}

#[derive(Deserialize)]
pub(super) struct AcceptanceRecord {
    pub restore: DeclaredDigest,
    pub source_metadata: DeclaredDigest,
    pub destination: Vec<u8>,
    pub complete_state: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_backup::model::operation_plan::PlanContextRequest;
    use serde_json::json;

    #[test]
    fn claims_require_original_fields_and_compare_exact_text() {
        let expected = PlanContextRecord::new(&PlanContextRequest {
            network: "ab".repeat(32),
            caller: "2vxsx-fae".into(),
            release: "cd".repeat(32),
        })
        .unwrap();
        let raw = json!({"context":expected,"target":"aaaaa-aa","controllers":["2vxsx-fae"],"module_hash":expected.release(),"status":"stopped"});
        let admitted: AdmissionRecord = serde_json::from_value(raw.clone()).unwrap();
        assert!(admitted.context.matches(&expected));
        for field in ["context", "target", "controllers", "module_hash", "status"] {
            let mut missing = raw.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<AdmissionRecord>(missing).is_err());
        }
        let mut changed = raw.clone();
        changed["context"]["release"] = json!(expected.release().to_uppercase());
        let admitted: AdmissionRecord = serde_json::from_value(changed).unwrap();
        assert!(!admitted.context.matches(&expected));
        let mut extra = raw;
        extra["context"]["unbound"] = json!(true);
        assert!(serde_json::from_value::<AdmissionRecord>(extra).is_err());
    }

    #[test]
    fn acceptance_retains_exact_checksum_declarations_and_requires_complete_state() {
        let expected = ArtifactChecksumRecord::from_bytes(b"original evidence");
        let raw = json!({"restore":expected,"source_metadata":expected,"destination":[0,255],"complete_state":true});
        let acceptance: AcceptanceRecord = serde_json::from_value(raw.clone()).unwrap();
        assert!(acceptance.restore.matches(&expected));
        assert_eq!(acceptance.destination, [0, 255]);
        for (field, replacement) in [
            ("hash", json!(expected.hash().to_uppercase())),
            ("algorithm", json!("other")),
        ] {
            let mut changed = raw.clone();
            changed["restore"][field] = replacement;
            let acceptance: AcceptanceRecord = serde_json::from_value(changed).unwrap();
            assert!(!acceptance.restore.matches(&expected));
        }
        let mut missing = raw;
        missing.as_object_mut().unwrap().remove("complete_state");
        assert!(serde_json::from_value::<AcceptanceRecord>(missing).is_err());
    }
}
