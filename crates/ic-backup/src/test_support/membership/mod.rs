//! Original plan fixture shared by current-membership model/policy qualification.

use crate::model::{artifacts::ArtifactChecksumRecord, operation_plan::OperationPlanRecord};

pub const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";

pub fn plan() -> OperationPlanRecord {
    OperationPlanRecord::new(crate::test_support::operation_plan::request())
        .expect("original qualified plan fixture")
}

pub fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).expect("opaque exact SHA-256")
}
