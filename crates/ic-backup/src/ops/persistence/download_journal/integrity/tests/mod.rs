//! Fresh no-follow published-byte verification and retained declaration rejection.

use super::*;
use crate::{
    model::download_journal::{ArtifactStateRecord, DownloadArtifactRequest},
    ops::persistence::{BackupLayoutGuard, create_operation_plan},
    test_support::temp_dir,
};
use serde_json::json;
use std::{fs, io, os::unix::fs::symlink, path::PathBuf};

const ROOT: &str = "aaaaa-aa";
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";

fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[
            {"canister_id":ROOT,"parent_canister_id":null,"role":null,"module_hash":null},
            {"canister_id":APP,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[ROOT,APP],"graph":{"version":1,"nodes":[{"operation_sequence":0,"depends_on":[]},{"operation_sequence":7,"depends_on":[]}]},
        "operations":[{"operation_sequence":0,"target":ROOT,"request":"ef".repeat(32),"budget":{"mutations":1,"observations":0}},
            {"operation_sequence":7,"target":APP,"request":"01".repeat(32),"budget":{"mutations":1,"observations":0}}],
        "budget":{"mutations":2,"observations":0}
    })).expect("original plan")
}

fn fixture() -> PathBuf {
    let root = temp_dir("ic-backup-download-integrity");
    for target in [ROOT, APP] {
        let directory = root.join(format!("artifacts/{target}.tmp/nested"));
        fs::create_dir_all(&directory).expect("private staging");
        fs::write(directory.join("heap.bin"), target.as_bytes()).expect("exact fixture bytes");
    }
    root
}

fn guard<'a>(
    layout: &'a BackupLayoutGuard,
    plan: &OperationPlanRecord,
    durable: bool,
) -> DownloadJournalGuard<'a> {
    let mut guard = DownloadJournalGuard::create(
        layout,
        plan.digest().hash(),
        [ROOT, APP]
            .into_iter()
            .map(|id| DownloadArtifactRequest {
                canister_id: id.into(),
                snapshot_id: format!("snap-{id}"),
                snapshot_taken_at_timestamp: 17,
                snapshot_total_size_bytes: 123,
            })
            .collect(),
    )
    .expect("original journal");
    if durable {
        for target in [ROOT, APP] {
            let snapshot = format!("snap-{target}");
            guard
                .record_downloaded(target, &snapshot)
                .expect("local complete-artifact attestation");
            guard
                .verify_artifact(target, &snapshot)
                .expect("staged byte verification");
            guard
                .finalize_artifact(target, &snapshot)
                .expect("durable publication");
        }
    }
    guard
}

#[test]
fn checks_every_selected_published_tree_and_preserves_exact_evidence() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let plan = plan();
    create_operation_plan(&layout, &plan).expect("retained original plan");
    let guard = guard(&layout, &plan, true);
    let bytes = fs::read(guard.path()).expect("retained bytes");
    let view = guard
        .verify_durable_artifacts(&plan)
        .expect("fresh local verification");
    assert_eq!(view.artifacts().len(), 2);
    assert_eq!(view.journal().resume_view().pending_artifacts, 0);
    for entry in view.artifacts() {
        assert_eq!(entry.artifact().state(), ArtifactStateRecord::Durable);
        assert_eq!(
            checksum_directory(&root.join(entry.artifact().artifact_path()))
                .expect("fixture checksum"),
            *entry.checksum()
        );
    }
    assert_eq!(
        fs::read(guard.path()).expect("journal bytes unchanged"),
        bytes
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(&root).expect("clean successful owned fixture");
}

#[test]
fn ordinary_reopen_skips_but_explicit_verification_rejects_changed_second_tree() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let plan = plan();
    create_operation_plan(&layout, &plan).expect("plan");
    let original = guard(&layout, &plan, true);
    let path = original.path();
    let bytes = fs::read(&path).expect("retained journal");
    drop(original);
    fs::write(
        root.join(format!("artifacts/{APP}/nested/heap.bin")),
        b"changed exact bytes",
    )
    .expect("fixture corruption");
    let reopened = DownloadJournalGuard::open(&layout, plan.digest().hash())
        .expect("local replay does not verify bytes");
    assert_eq!(
        reopened
            .record()
            .expect("retained projection")
            .resume_view()
            .pending_artifacts,
        0
    );
    assert!(matches!(
        reopened.verify_durable_artifacts(&plan),
        Err(DownloadIntegrityError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    assert_eq!(fs::read(&path).expect("retained evidence"), bytes);
    drop(reopened);
    drop(layout);
    fs::remove_dir_all(&root).expect("clean successful owned fixture");
}

#[test]
fn rejects_non_durable_set_before_attempting_any_artifact_reads() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let plan = plan();
    create_operation_plan(&layout, &plan).expect("plan");
    let guard = guard(&layout, &plan, false);
    assert!(matches!(
        guard.verify_durable_artifacts(&plan),
        Err(DownloadIntegrityError::Policy(
            DownloadIntegrityPolicyError::NonDurableArtifact { .. }
        ))
    ));
    drop(guard);
    drop(layout);
    fs::remove_dir_all(&root).expect("clean successful owned fixture");
}

#[test]
fn requires_original_persisted_plan_and_unchanged_guarded_journal() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let plan = plan();
    let guard = guard(&layout, &plan, true);
    assert!(
        matches!(guard.verify_durable_artifacts(&plan),Err(DownloadIntegrityError::Plan(
        OperationPlanPersistenceError::Persistence(PersistenceError::Io(error)))) if error.kind()==io::ErrorKind::NotFound)
    );
    create_operation_plan(&layout, &plan).expect("plan retained");
    let mut changed =
        serde_json::to_value(guard.record().expect("record")).expect("fixture declaration");
    changed["artifacts"][0]["snapshot_id"] = json!("changed-source-token");
    fs::write(
        guard.path(),
        serde_json::to_vec(&changed).expect("fixture bytes"),
    )
    .expect("simulated external journal edit");
    assert!(matches!(
        guard.verify_durable_artifacts(&plan),
        Err(DownloadIntegrityError::JournalChanged)
    ));
    assert_eq!(
        fs::read(guard.path()).expect("changed evidence retained"),
        serde_json::to_vec(&changed).expect("exact bytes")
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(&root).expect("clean successful owned fixture");
}

#[test]
fn rejects_replaced_original_plan_without_rebinding_the_download_journal() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let plan = plan();
    create_operation_plan(&layout, &plan).expect("original plan");
    let plan_path = root.join("operation-plan.json");
    let guard = guard(&layout, &plan, true);
    let journal_bytes = fs::read(guard.path()).expect("original journal bytes");
    let mut changed = serde_json::to_value(&plan).expect("fixture plan");
    changed["context"]["release"] = json!("02".repeat(32));
    let changed_bytes = serde_json::to_vec(&changed).expect("changed plan bytes");
    fs::write(&plan_path, &changed_bytes).expect("simulated external plan replacement");
    assert!(matches!(
        guard.verify_durable_artifacts(&plan),
        Err(DownloadIntegrityError::Plan(
            OperationPlanPersistenceError::DigestMismatch
        ))
    ));
    assert_eq!(
        fs::read(&plan_path).expect("replacement evidence retained"),
        changed_bytes
    );
    assert_eq!(
        fs::read(guard.path()).expect("journal unchanged"),
        journal_bytes
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful owned fixture");
}

#[test]
fn rejects_missing_regular_file_and_symlink_artifact_substitutions() {
    for kind in ["missing", "file", "symlink"] {
        let root = fixture();
        let layout = BackupLayoutGuard::acquire(&root).expect("layout");
        let plan = plan();
        create_operation_plan(&layout, &plan).expect("plan");
        let guard = guard(&layout, &plan, true);
        let path = root.join(format!("artifacts/{APP}"));
        fs::rename(&path, root.join("retained-second-tree"))
            .expect("retain substituted fixture evidence");
        if kind == "file" {
            fs::write(&path, b"regular file replacement").expect("file");
        }
        if kind == "symlink" {
            symlink(root.join("retained-second-tree"), &path).expect("symlink");
        }
        let result = guard.verify_durable_artifacts(&plan);
        if kind == "file" {
            assert!(matches!(
                result,
                Err(DownloadIntegrityError::Artifact(
                    ArtifactError::UnsupportedEntry { .. }
                ))
            ));
        } else {
            assert!(matches!(
                result,
                Err(DownloadIntegrityError::Artifact(ArtifactError::Io(_)))
            ));
        }
        assert!(root.join("retained-second-tree/nested/heap.bin").exists());
        drop(guard);
        drop(layout);
        fs::remove_dir_all(&root).expect("clean successful owned fixture");
    }
}

#[test]
fn unusable_custody_and_corrupt_retained_journal_stop_without_changes() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let plan = plan();
    create_operation_plan(&layout, &plan).expect("plan");
    let mut guard = guard(&layout, &plan, true);
    guard.usable = false;
    assert!(matches!(
        guard.verify_durable_artifacts(&plan),
        Err(DownloadIntegrityError::Journal(
            DownloadJournalError::IndeterminateWrite
        ))
    ));
    guard.usable = true;
    fs::write(guard.path(), b"corrupt retained journal").expect("fixture corruption");
    assert!(matches!(
        guard.verify_durable_artifacts(&plan),
        Err(DownloadIntegrityError::Persistence(PersistenceError::Json(
            _
        )))
    ));
    assert_eq!(
        fs::read(guard.path()).expect("corrupt evidence retained"),
        b"corrupt retained journal"
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(&root).expect("clean successful owned fixture");
}
