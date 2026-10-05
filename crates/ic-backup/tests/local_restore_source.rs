//! Public exact local source recovery preserves spent attempts, fence and source references.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        download_journal::DownloadArtifactRequest,
        fence_obligation::FenceObligationRecord,
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreFenceBindingRecord, RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest,
        },
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, FenceObligationRequirement,
        create_fence_obligation, create_operation_plan, create_restore_safety_requirement,
    },
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
fn root() -> PathBuf {
    std::env::temp_dir().canonicalize().unwrap().join(format!(
        "ic-backup-public-local-source-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn plans() -> (OperationPlanRecord, OperationPlanRecord) {
    let value = json!({"version":1,
        "context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":TARGET,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[TARGET],"graph":{"version":1,"nodes":[{"operation_sequence":0,"depends_on":[]}]},
        "operations":[{"operation_sequence":0,"target":TARGET,"request":"ef".repeat(32),"budget":{"mutations":2,"observations":1}}],
        "budget":{"mutations":2,"observations":1}});
    let restore = serde_json::from_value(value.clone()).unwrap();
    let mut source = value;
    source["context"]["caller"] = json!("aaaaa-aa");
    source["operations"][0]["request"] = json!("01".repeat(32));
    (restore, serde_json::from_value(source).unwrap())
}
fn retain_requirement(
    layout: &BackupLayoutGuard,
    source_layout: &BackupLayoutGuard,
    restore: &OperationPlanRecord,
    source: &OperationPlanRecord,
    source_artifacts: ArtifactChecksumRecord,
) -> RestoreSafetyRequirementRecord {
    let requirement = RestoreSafetyRequirementRecord::new(
        restore,
        source,
        RestoreSafetyRequirementRequest {
            source_artifacts,
            safety: RestoreSafetyLaneRecord::ApplicationFenced,
            expected_fence: Some(RestoreFenceBindingRecord {
                identity: hash("12"),
                membership_revision: hash("34"),
                external_obligations_revision: hash("56"),
            }),
        },
    )
    .unwrap();
    create_restore_safety_requirement(layout, source_layout, restore, source, &requirement)
        .unwrap();
    let obligation = FenceObligationRecord::for_restore(restore, source, &requirement, 0).unwrap();
    create_fence_obligation(
        layout,
        restore,
        FenceObligationRequirement::Restore {
            source_layout,
            source,
            requirement: &requirement,
        },
        &obligation,
    )
    .unwrap();
    requirement
}

fn retain_durable_source<'a>(
    layout: &'a BackupLayoutGuard,
    source: &OperationPlanRecord,
) -> DownloadJournalGuard<'a> {
    let mut journal = DownloadJournalGuard::create(
        layout,
        source.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: TARGET.into(),
            snapshot_id: "Original-Snapshot".into(),
            snapshot_taken_at_timestamp: u64::MAX,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap();
    journal
        .record_downloaded(TARGET, "Original-Snapshot")
        .unwrap();
    journal
        .verify_artifact(TARGET, "Original-Snapshot")
        .unwrap();
    journal
        .finalize_artifact(TARGET, "Original-Snapshot")
        .unwrap();
    journal
}

#[test]
fn fresh_local_source_verification_reopens_without_new_spending_or_releasing_obligations() {
    let root = root();
    let staging = root.join(format!("source/artifacts/{TARGET}.tmp"));
    fs::create_dir_all(&staging).unwrap();
    fs::create_dir(root.join("restore")).unwrap();
    fs::write(staging.join("heap.bin"), b"original source bytes").unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let (restore, source) = plans();
    create_operation_plan(&source_layout, &source).unwrap();
    create_operation_plan(&layout, &restore).unwrap();
    let journal = retain_durable_source(&source_layout, &source);
    let manifest = journal.publish_download_manifest(&source).unwrap();
    let requirement = retain_requirement(&layout, &source_layout, &restore, &source, manifest);
    let mut attempt =
        AttemptJournalGuard::create(&layout, restore.attempt_authority(0).unwrap()).unwrap();
    attempt.reserve_mutation().unwrap();
    source_layout
        .retain_restore(&attempt.path(), restore.digest().hash())
        .unwrap();
    let references = source_layout.restore_references().unwrap();
    let names = [
        "source/download-journal.json",
        "source/download-manifest.json",
        "restore/restore-safety-requirement.json",
        "restore/fence-obligation.json",
        "restore/attempt-0.json",
    ];
    let original: Vec<_> = names
        .iter()
        .map(|name| fs::read(root.join(name)).unwrap())
        .collect();
    let view = journal
        .verify_local_restore_source(&layout, &restore, &source, &requirement)
        .unwrap();
    let artifact = view.selected_artifacts().next().unwrap();
    assert_eq!(artifact.artifact().canister_id(), TARGET);
    assert_eq!(artifact.artifact().snapshot_id(), "Original-Snapshot");
    assert_eq!(artifact.artifact().snapshot_total_size_bytes(), u64::MAX);
    let published = root.join(format!("source/artifacts/{TARGET}/heap.bin"));
    fs::write(&published, b"changed current bytes").unwrap();
    journal
        .read_download_manifest(&source, requirement.source_artifacts())
        .unwrap();
    assert!(
        journal
            .verify_local_restore_source(&layout, &restore, &source, &requirement)
            .is_err()
    );
    fs::write(&published, b"original source bytes").unwrap();
    drop(attempt);
    drop(journal);
    drop(layout);
    drop(source_layout);
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let journal = DownloadJournalGuard::open(&source_layout, source.digest().hash()).unwrap();
    journal
        .verify_local_restore_source(&layout, &restore, &source, &requirement)
        .unwrap();
    let attempt =
        AttemptJournalGuard::open(&layout, &restore.attempt_authority(0).unwrap()).unwrap();
    let view = attempt.record().unwrap().view();
    assert_eq!(view.pending_mutation, Some(1));
    assert_eq!(view.mutations_used, 1);
    assert_eq!(view.mutations_remaining, 1);
    assert_eq!(source_layout.restore_references().unwrap(), references);
    for (name, bytes) in names.iter().zip(original) {
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes);
    }
    drop(attempt);
    drop(journal);
    drop(layout);
    drop(source_layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn private_restore_copy_recovers_with_source_trees_absent_and_original_obligations_retained() {
    let root = root();
    let staging = root.join(format!("source/artifacts/{TARGET}.tmp"));
    fs::create_dir_all(&staging).unwrap();
    fs::create_dir(root.join("restore")).unwrap();
    fs::write(staging.join("heap.bin"), b"original source bytes").unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let (restore, source) = plans();
    create_operation_plan(&source_layout, &source).unwrap();
    create_operation_plan(&layout, &restore).unwrap();
    let journal = retain_durable_source(&source_layout, &source);
    let manifest = journal.publish_download_manifest(&source).unwrap();
    let requirement = retain_requirement(&layout, &source_layout, &restore, &source, manifest);
    let mut attempt =
        AttemptJournalGuard::create(&layout, restore.attempt_authority(0).unwrap()).unwrap();
    attempt.reserve_mutation().unwrap();
    source_layout
        .retain_restore(&attempt.path(), restore.digest().hash())
        .unwrap();
    let references = source_layout.restore_references().unwrap();
    let names = [
        "source/download-journal.json",
        "source/download-manifest.json",
        "restore/restore-safety-requirement.json",
        "restore/fence-obligation.json",
        "restore/attempt-0.json",
    ];
    let originals: Vec<_> = names
        .iter()
        .map(|name| fs::read(root.join(name)).unwrap())
        .collect();
    let copy = journal
        .stage_local_restore_artifact(&layout, &restore, &source, &requirement, 0)
        .unwrap();
    assert_eq!(copy.operation().operation_sequence(), 0);
    assert_eq!(copy.artifact().snapshot_id(), "Original-Snapshot");
    let path = copy.path().to_owned();
    assert_eq!(
        fs::read(path.join("heap.bin")).unwrap(),
        b"original source bytes"
    );
    drop(copy);
    assert!(
        journal
            .stage_local_restore_artifact(&layout, &restore, &source, &requirement, 0)
            .is_err()
    );
    fs::rename(
        root.join("source/artifacts"),
        root.join("retained-source-artifacts"),
    )
    .unwrap();
    drop(attempt);
    drop(journal);
    drop(layout);
    drop(source_layout);
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let journal = DownloadJournalGuard::open(&source_layout, source.digest().hash()).unwrap();
    let copy = journal
        .verify_staged_local_restore_artifact(&layout, &restore, &source, &requirement, 0)
        .unwrap();
    assert_eq!(copy.path(), path);
    assert_eq!(
        fs::read(copy.path().join("heap.bin")).unwrap(),
        b"original source bytes"
    );
    assert!(
        journal
            .verify_local_restore_source(&layout, &restore, &source, &requirement)
            .is_err()
    );
    let attempt =
        AttemptJournalGuard::open(&layout, &restore.attempt_authority(0).unwrap()).unwrap();
    let progress = attempt.record().unwrap().view();
    assert_eq!(progress.pending_mutation, Some(1));
    assert_eq!(
        (progress.mutations_used, progress.mutations_remaining),
        (1, 1)
    );
    assert_eq!(source_layout.restore_references().unwrap(), references);
    for (name, bytes) in names.iter().zip(originals) {
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes);
    }
    drop(copy);
    drop(attempt);
    drop(journal);
    drop(layout);
    drop(source_layout);
    fs::remove_dir_all(root).unwrap();
}
