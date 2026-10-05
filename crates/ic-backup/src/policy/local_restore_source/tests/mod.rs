//! Exact original local bindings, full source admission and read-only subset projections.

use super::*;
use crate::{
    model::{artifacts::ArtifactChecksumRecord, restore_safety::RestoreSafetyRequirementRequest},
    test_support::{
        local_restore_source::{manifest, requirement, source},
        membership::APP,
        restore_safety,
    },
};
use serde_json::json;

#[test]
fn local_restore_source_projects_exact_subset_without_source_caller_rebinding() {
    let restore = restore_safety::plan();
    let source = source();
    let manifest = manifest(&source);
    let requirement = requirement(&restore, &source, &manifest);
    let view = validate(&restore, &source, &requirement, &manifest).unwrap();
    assert_eq!(view.restore(), &restore);
    assert_eq!(view.requirement(), &requirement);
    assert_eq!(view.source().plan(), &source);
    assert_eq!(view.source().journal(), &manifest);
    assert_eq!(view.source().artifacts().len(), 2);
    let selected: Vec<_> = view.selected_artifacts().collect();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].artifact().canister_id(), APP);
    assert_eq!(
        selected[0].artifact().snapshot_id(),
        format!("Original-{APP}")
    );
    assert_eq!(
        selected[0].artifact().snapshot_taken_at_timestamp(),
        u64::MAX
    );
    assert_ne!(source.context().caller(), restore.context().caller());
}

#[test]
fn local_restore_source_rejects_changed_original_plan_and_exact_manifest_identity() {
    let restore = restore_safety::plan();
    let source = source();
    let manifest = manifest(&source);
    let requirement = requirement(&restore, &source, &manifest);
    let mut changed = serde_json::to_value(&restore).unwrap();
    changed["operations"][0]["budget"]["mutations"] = json!(0);
    let changed: OperationPlanRecord = serde_json::from_value(changed).unwrap();
    assert!(matches!(
        validate(&changed, &source, &requirement, &manifest),
        Err(LocalRestoreSourcePolicyError::Requirement(
            RestoreSafetyRequirementError::PlanMismatch
        ))
    ));
    let mut changed = serde_json::to_value(&source).unwrap();
    changed["context"]["caller"] = json!("2vxsx-fae");
    let changed: OperationPlanRecord = serde_json::from_value(changed).unwrap();
    assert!(matches!(
        validate(&restore, &changed, &requirement, &manifest),
        Err(LocalRestoreSourcePolicyError::Requirement(
            RestoreSafetyRequirementError::SourcePlanMismatch
        ))
    ));
    for (field, value) in [
        ("snapshot_id", json!("substituted-token")),
        ("snapshot_taken_at_timestamp", json!(0)),
        ("snapshot_total_size_bytes", json!(0)),
        (
            "checksum",
            json!({"algorithm":"sha256","hash":"34".repeat(32)}),
        ),
    ] {
        let mut changed = serde_json::to_value(&manifest).unwrap();
        changed["artifacts"][0][field] = value;
        let changed: DownloadJournalRecord = serde_json::from_value(changed).unwrap();
        assert!(matches!(
            validate(&restore, &source, &requirement, &changed),
            Err(LocalRestoreSourcePolicyError::ManifestMismatch)
        ));
    }
}

#[test]
fn local_restore_source_cannot_accept_incomplete_or_non_durable_original_source() {
    let restore = restore_safety::plan();
    let source = source();
    let original = manifest(&source);
    for kind in ["missing", "non-durable", "intent"] {
        let mut value = serde_json::to_value(&original).unwrap();
        match kind {
            "missing" => {
                value["artifacts"].as_array_mut().unwrap().remove(0);
            }
            "non-durable" => {
                value["artifacts"][0]["state"] = json!("ChecksumVerified");
            }
            "intent" => {
                value["intent"] = json!("ab".repeat(32));
            }
            _ => unreachable!(),
        }
        let manifest: DownloadJournalRecord = serde_json::from_value(value).unwrap();
        let requirement = requirement(&restore, &source, &manifest);
        let result = validate(&restore, &source, &requirement, &manifest);
        match kind {
            "missing" => assert!(matches!(
                result,
                Err(LocalRestoreSourcePolicyError::Download(
                    DownloadIntegrityPolicyError::TargetSetMismatch
                ))
            )),
            "non-durable" => assert!(matches!(
                result,
                Err(LocalRestoreSourcePolicyError::Download(
                    DownloadIntegrityPolicyError::NonDurableArtifact { .. }
                ))
            )),
            "intent" => assert!(matches!(
                result,
                Err(LocalRestoreSourcePolicyError::Download(
                    DownloadIntegrityPolicyError::IntentMismatch
                ))
            )),
            _ => unreachable!(),
        }
    }
    let generic = RestoreSafetyRequirementRecord::new(
        &restore,
        &source,
        RestoreSafetyRequirementRequest {
            source_artifacts: ArtifactChecksumRecord::from_bytes(
                b"integration-specific complete source",
            ),
            safety: crate::model::restore_safety::RestoreSafetyLaneRecord::NoIrreversibleEffects,
            expected_fence: None,
        },
    )
    .unwrap();
    generic.validate_plans(&restore, &source).unwrap();
    assert!(matches!(
        validate(&restore, &source, &generic, &original),
        Err(LocalRestoreSourcePolicyError::ManifestMismatch)
    ));
}
