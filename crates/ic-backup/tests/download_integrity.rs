//! Native public local integrity checks preserve original spending and retention.

#![cfg(unix)]

mod support;

use ic_backup::{
    model::{
        artifacts::ChecksumError, download_journal::DownloadArtifactRequest,
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadIntegrityError, DownloadJournalGuard,
        create_operation_plan, read_operation_plan,
    },
};
use serde_json::json;
use std::fs;

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";

fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[
            {"canister_id":ROOT,"parent_canister_id":null,"role":null,"module_hash":null},
            {"canister_id":APP,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[ROOT,APP],
        "graph":{"version":1,"nodes":[{"operation_sequence":0,"depends_on":[]},{"operation_sequence":7,"depends_on":[]}]},
        "operations":[{"operation_sequence":0,"target":ROOT,"request":"ef".repeat(32),"budget":{"mutations":1,"observations":1}},
            {"operation_sequence":7,"target":APP,"request":"01".repeat(32),"budget":{"mutations":1,"observations":1}}],
        "budget":{"mutations":2,"observations":2}
    })).expect("original immutable plan")
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one public recovery journey keeps artifact verification, spending and restore retention together"
)]
fn fresh_verification_preserves_pending_spending_and_restore_references() {
    let root = support::temp_root("ic-backup-public-download-integrity");
    let layout = BackupLayoutGuard::acquire(&root).expect("layout custody");
    let original = plan();
    create_operation_plan(&layout, &original).expect("retain original plan");
    let plan_bytes = fs::read(root.join("operation-plan.json")).expect("original plan bytes");
    let intent = original.digest();
    let mut attempts = AttemptJournalGuard::create(
        &layout,
        original.attempt_authority(7).expect("original allowance"),
    )
    .expect("original attempt journal");
    let mutation = attempts.reserve_mutation().expect("spend mutation");
    let observation = attempts
        .reserve_observation(mutation, &"02".repeat(32))
        .expect("spend observation");
    let spent = attempts
        .record()
        .expect("original pending accounting")
        .view();
    assert_eq!(spent.pending_mutation, Some(mutation));
    assert_eq!(spent.pending_observation, Some(observation));
    assert_eq!(spent.mutations_remaining, 0);
    assert_eq!(spent.observations_remaining, 0);
    let attempt_path = attempts.path();
    let attempt_bytes = fs::read(&attempt_path).expect("spent bytes");
    drop(attempts);

    let reference = layout
        .retain_restore(&root.join("unfinished-restore.json"), intent.hash())
        .expect("retain unfinished restore dependency");
    let references = layout.restore_references().expect("retained references");
    let reference_bytes = fs::read(root.join("restore-references.json")).expect("reference bytes");
    let mut downloads = DownloadJournalGuard::create(
        &layout,
        intent.hash(),
        [ROOT, APP]
            .into_iter()
            .map(|target| DownloadArtifactRequest {
                canister_id: target.into(),
                snapshot_id: format!("snapshot-{target}"),
                snapshot_taken_at_timestamp: u64::MAX,
                snapshot_total_size_bytes: u64::MAX,
            })
            .collect(),
    )
    .expect("exact immutable download identity");
    for target in [ROOT, APP] {
        let staging = root.join(format!("artifacts/{target}.tmp"));
        fs::create_dir_all(&staging).expect("private staging");
        fs::write(staging.join("heap.bin"), target.as_bytes()).expect("local fixture bytes");
        let snapshot = format!("snapshot-{target}");
        downloads
            .record_downloaded(target, &snapshot)
            .expect("local integration attestation");
        downloads
            .verify_artifact(target, &snapshot)
            .expect("exact staged checksum");
        downloads
            .finalize_artifact(target, &snapshot)
            .expect("durable local publication");
    }
    let download_path = downloads.path();
    let download_bytes = fs::read(&download_path).expect("retained download bytes");
    let record = downloads.record().expect("original journal").clone();
    drop(downloads);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).expect("new layout owner");
    let plan = read_operation_plan(&layout, &intent).expect("original persisted plan");
    let downloads = DownloadJournalGuard::open(&layout, intent.hash()).expect("local resume");
    let view = downloads
        .verify_durable_artifacts(&plan)
        .expect("fresh published-byte checks");
    assert_eq!(view.journal(), &record);
    assert_eq!(view.artifacts().len(), 2);
    assert_eq!(view.journal().resume_view().pending_artifacts, 0);
    fs::write(
        root.join(format!("artifacts/{APP}/heap.bin")),
        b"changed published bytes",
    )
    .expect("native fixture corruption");
    assert!(matches!(
        downloads.verify_durable_artifacts(&plan),
        Err(DownloadIntegrityError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    assert_eq!(
        downloads
            .record()
            .expect("local replay retains durable state"),
        &record
    );
    let attempts = AttemptJournalGuard::open(
        &layout,
        &plan.attempt_authority(7).expect("original authority"),
    )
    .expect("same spent journal");
    assert_eq!(
        attempts.record().expect("accounting unchanged").view(),
        spent
    );
    assert_eq!(
        layout.restore_references().expect("references unchanged"),
        references
    );
    assert!(
        layout
            .has_restore_references()
            .expect("unfinished dependency retained")
    );
    assert_eq!(
        layout
            .retain_restore(&root.join("unfinished-restore.json"), intent.hash())
            .expect("same retained reference"),
        reference
    );
    assert_eq!(
        fs::read(&download_path).expect("download bytes unchanged"),
        download_bytes
    );
    assert_eq!(
        fs::read(root.join("operation-plan.json")).expect("original plan bytes unchanged"),
        plan_bytes
    );
    assert_eq!(
        fs::read(&attempt_path).expect("attempt bytes unchanged"),
        attempt_bytes
    );
    assert_eq!(
        fs::read(root.join("restore-references.json")).expect("reference bytes unchanged"),
        reference_bytes
    );
    drop(attempts);
    drop(downloads);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful owned fixture");
}
