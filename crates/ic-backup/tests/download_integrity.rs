//! Public primitive composition preserves exact local source evidence and pending obligations.

#![cfg(unix)]

#[expect(
    dead_code,
    reason = "this integration target uses only a subset of the shared fixture helpers"
)]
mod support;

use ic_backup::{
    model::{
        artifacts::{ArtifactChecksumRecord, ChecksumError},
        download_journal::DownloadArtifactRequest,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreFenceBindingRecord, RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest,
        },
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, IcSnapshotArtifactError,
        create_operation_plan, create_restore_safety_requirement, read_restore_safety_requirement,
    },
};
use serde_json::json;
use std::fs;

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";

fn source_plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":ROOT,"release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[
            {"canister_id":ROOT,"parent_canister_id":null,"role":null,"module_hash":null},
            {"canister_id":APP,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[APP],
        "graph":{"version":1,"nodes":[{"operation_sequence":7,"depends_on":[]}]},
        "operations":[{"operation_sequence":7,"target":APP,"request":"01".repeat(32),"budget":{"mutations":1,"observations":1}}],
        "budget":{"mutations":1,"observations":1}
    })).expect("original immutable source plan")
}

fn retain_requirement(
    layout: &BackupLayoutGuard,
    source_layout: &BackupLayoutGuard,
    restore: &OperationPlanRecord,
    source: &OperationPlanRecord,
    manifest: ArtifactChecksumRecord,
) -> RestoreSafetyRequirementRecord {
    let hash = |bytes: &[u8]| ArtifactChecksumRecord::from_bytes(bytes);
    let requirement = RestoreSafetyRequirementRecord::new(
        restore,
        source,
        RestoreSafetyRequirementRequest {
            source_artifacts: manifest,
            safety: RestoreSafetyLaneRecord::ApplicationFenced,
            expected_fence: Some(RestoreFenceBindingRecord {
                identity: hash(b"original fence"),
                membership_revision: hash(b"original membership"),
                external_obligations_revision: hash(b"original obligations"),
            }),
        },
    )
    .unwrap();
    create_restore_safety_requirement(layout, source_layout, restore, source, &requirement)
        .unwrap();
    fs::write(
        layout.root().join("application-obligation.json"),
        b"original integration-owned fence and external-obligation evidence",
    )
    .unwrap();
    requirement
}

// This native fixture independently selected the raw IC ID in finish_ic_artifact.
// Never infer it from the journal's opaque local token.
fn verify_ic_source(
    journal: &DownloadJournalGuard<'_>,
    plan: &OperationPlanRecord,
) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
    let entry = &journal.record().unwrap().artifacts()[0];
    let root = journal.path().parent().unwrap().to_path_buf();
    let raw = fs::read(root.join(entry.artifact_path()).join("metadata.candid")).unwrap();
    let request = IcSnapshotMetadataRequest::new(APP, &[0, 255, 17]).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    journal.verify_ic_snapshot_artifact(plan, entry.snapshot_id(), &metadata)
}

// Test-only composition of current public owners, with before/after retained admission.
// Applications still own stable bytes, authentic IC evidence and actual subset safety.
fn verify_source(
    layout: &BackupLayoutGuard,
    source_layout: &BackupLayoutGuard,
    restore: &OperationPlanRecord,
    source: &OperationPlanRecord,
    journal: &DownloadJournalGuard<'_>,
    requirement: &RestoreSafetyRequirementRecord,
) {
    let admit = || {
        assert_eq!(
            read_restore_safety_requirement(
                layout,
                source_layout,
                restore,
                source,
                &requirement.digest(),
            )
            .unwrap(),
            *requirement
        );
        journal
            .read_download_manifest(source, requirement.source_artifacts())
            .unwrap();
    };
    admit();
    verify_ic_source(journal, source).unwrap();
    assert_eq!(restore.selected_targets(), [APP]);
    assert_ne!(restore.context().caller(), source.context().caller());
    admit();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one public recovery journey keeps fresh source checks, spent attempts and fence/reference retention together"
)]
fn fresh_verification_preserves_pending_spending_and_restore_references() {
    let root = support::temp_root("ic-backup-public-download-integrity");
    let source_root = root.join("source");
    let restore_root = root.join("restore");
    fs::create_dir(&source_root).unwrap();
    fs::create_dir(source_root.join("artifacts")).unwrap();
    fs::create_dir(&restore_root).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&source_root).unwrap();
    let layout = BackupLayoutGuard::acquire(&restore_root).unwrap();
    let source = source_plan();
    let restore = support::single_operation_plan(APP, 7, &"03".repeat(32));
    create_operation_plan(&source_layout, &source).unwrap();
    create_operation_plan(&layout, &restore).unwrap();
    let mut journal = DownloadJournalGuard::create(
        &source_layout,
        source.digest().hash(),
        DownloadArtifactRequest {
            canister_id: APP.into(),
            snapshot_id: format!("snapshot-{APP}"),
            snapshot_taken_at_timestamp: u64::MAX,
            snapshot_total_size_bytes: u64::MAX,
        },
    )
    .unwrap();
    {
        let target = APP;
        let snapshot = format!("snapshot-{target}");
        support::finish_ic_artifact(&mut journal, target, &snapshot, target.as_bytes());
    }
    let manifest = journal.publish_download_manifest(&source).unwrap();
    let requirement = retain_requirement(&layout, &source_layout, &restore, &source, manifest);
    let mut attempts =
        AttemptJournalGuard::create(&layout, restore.attempt_authority(7).unwrap()).unwrap();
    let mutation = attempts
        .reserve_planned_mutation(&restore.digest())
        .unwrap();
    let observation = attempts
        .reserve_planned_observation(&restore.digest(), mutation, &"02".repeat(32))
        .unwrap();
    let spent = attempts.record().unwrap().view();
    assert_eq!(spent.pending_mutation, Some(mutation));
    assert_eq!(spent.pending_observation, Some(observation));
    assert_eq!(spent.mutations_remaining, 0);
    assert_eq!(spent.observations_remaining, 0);
    source_layout
        .retain_restore(&attempts.path(), restore.digest().hash())
        .unwrap();
    let references = source_layout.restore_references().unwrap();
    let names = [
        "source/operation-plan.json",
        "source/download-journal.json",
        "source/download-manifest.json",
        "source/restore-references.json",
        "restore/operation-plan.json",
        "restore/attempt-7.json",
        "restore/restore-safety-requirement.json",
        "restore/application-obligation.json",
    ];
    let original: Vec<_> = names
        .iter()
        .map(|name| fs::read(root.join(name)).unwrap())
        .collect();
    verify_source(
        &layout,
        &source_layout,
        &restore,
        &source,
        &journal,
        &requirement,
    );
    // The original single-artifact source must still be checked after manifest replay.
    let published = source_root.join(format!("artifacts/{APP}/wasm-module.bin"));
    fs::write(&published, vec![b'x'; APP.len()]).unwrap();
    journal
        .read_download_manifest(&source, requirement.source_artifacts())
        .unwrap();
    assert!(matches!(
        verify_ic_source(&journal, &source),
        Err(IcSnapshotArtifactError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    for (name, bytes) in names.iter().zip(&original) {
        assert_eq!(&fs::read(root.join(name)).unwrap(), bytes);
    }
    fs::write(&published, APP.as_bytes()).unwrap();
    drop(attempts);
    drop(journal);
    drop(layout);
    drop(source_layout);

    let source_layout = BackupLayoutGuard::acquire(&source_root).unwrap();
    let layout = BackupLayoutGuard::acquire(&restore_root).unwrap();
    let journal = DownloadJournalGuard::open(&source_layout, source.digest().hash()).unwrap();
    verify_source(
        &layout,
        &source_layout,
        &restore,
        &source,
        &journal,
        &requirement,
    );
    let attempts =
        AttemptJournalGuard::open(&layout, &restore.attempt_authority(7).unwrap()).unwrap();
    assert_eq!(attempts.record().unwrap().view(), spent);
    assert_eq!(source_layout.restore_references().unwrap(), references);
    assert!(source_layout.has_restore_references().unwrap());
    for (name, bytes) in names.iter().zip(original) {
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes);
    }
    drop(attempts);
    drop(journal);
    drop(layout);
    drop(source_layout);
    fs::remove_dir_all(root).unwrap();
}
