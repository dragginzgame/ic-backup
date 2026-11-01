//! Shared public-test declarations; effect chronology and qualification stay in each suite.
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        fence_obligation::FenceObligationRecord,
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreFenceBindingRecord, RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest,
        },
    },
    ops::persistence::{BackupLayoutGuard, FenceObligationRequirement},
};
use serde_json::json;

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}

pub(crate) struct RestoreOriginals {
    pub(crate) plan: OperationPlanRecord,
    pub(crate) source: OperationPlanRecord,
    pub(crate) requirement: RestoreSafetyRequirementRecord,
    pub(crate) obligation: FenceObligationRecord,
}
impl RestoreOriginals {
    pub(crate) fn new(plan: OperationPlanRecord, sequence: u64) -> Self {
        let mut source = serde_json::to_value(&plan).unwrap();
        source["operations"][0]["request"] = json!(hash("ef").hash());
        let source = serde_json::from_value(source).unwrap();
        let requirement = RestoreSafetyRequirementRecord::new(
            &plan,
            &source,
            RestoreSafetyRequirementRequest {
                source_artifacts: hash("12"),
                safety: RestoreSafetyLaneRecord::ApplicationFenced,
                expected_fence: Some(RestoreFenceBindingRecord {
                    identity: hash("56"),
                    membership_revision: hash("78"),
                    external_obligations_revision: hash("90"),
                }),
            },
        )
        .unwrap();
        let obligation =
            FenceObligationRecord::for_restore(&plan, &source, &requirement, sequence).unwrap();
        Self {
            plan,
            source,
            requirement,
            obligation,
        }
    }
    pub(crate) fn input<'a>(
        &'a self,
        source_layout: &'a BackupLayoutGuard,
    ) -> FenceObligationRequirement<'a> {
        FenceObligationRequirement::Restore {
            source_layout,
            source: &self.source,
            requirement: &self.requirement,
        }
    }
}
