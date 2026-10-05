//! Guarded original local source joins and fresh byte/custody failure preservation.

use super::*;
use crate::{
    model::{artifacts::ChecksumError, restore_safety::RestoreSafetyRequirementRecord},
    ops::persistence::{
        PersistenceError, create_operation_plan, create_restore_safety_requirement,
    },
    test_support::{local_restore_source, restore_safety, temp_dir},
};
use serde_json::json;
use std::{fs, os::unix::fs::symlink, path::PathBuf};

struct Fixture {
    root: PathBuf,
    restore_layout: BackupLayoutGuard,
    source_layout: BackupLayoutGuard,
    restore: OperationPlanRecord,
    source: OperationPlanRecord,
    requirement: RestoreSafetyRequirementRecord,
}
fn fixture() -> Fixture {
    let root = temp_dir("ic-backup-local-restore-source");
    fs::create_dir_all(root.join("source")).unwrap();
    fs::create_dir(root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let restore_layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let restore = restore_safety::plan();
    let source = local_restore_source::source();
    create_operation_plan(&source_layout, &source).unwrap();
    create_operation_plan(&restore_layout, &restore).unwrap();
    let mut journal = DownloadJournalGuard::create(
        &source_layout,
        source.digest().hash(),
        local_restore_source::requests(&source),
    )
    .unwrap();
    for target in source.selected_targets() {
        let staging = root.join(format!("source/artifacts/{target}.tmp"));
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("heap.bin"), target.as_bytes()).unwrap();
        let snapshot = format!("Original-{target}");
        journal.record_downloaded(target, &snapshot).unwrap();
        journal.verify_artifact(target, &snapshot).unwrap();
        journal.finalize_artifact(target, &snapshot).unwrap();
    }
    journal.publish_download_manifest(&source).unwrap();
    let requirement =
        local_restore_source::requirement(&restore, &source, journal.record().unwrap());
    create_restore_safety_requirement(
        &restore_layout,
        &source_layout,
        &restore,
        &source,
        &requirement,
    )
    .unwrap();
    drop(journal);
    Fixture {
        root,
        restore_layout,
        source_layout,
        restore,
        source,
        requirement,
    }
}
fn verify<'a>(
    f: &'a Fixture,
    journal: &'a DownloadJournalGuard<'_>,
) -> Result<LocalRestoreSourceView<'a>, LocalRestoreSourceError> {
    journal.verify_local_restore_source(&f.restore_layout, &f.restore, &f.source, &f.requirement)
}

#[test]
fn local_restore_source_retains_exact_original_records_and_selects_existing_ids() {
    let f = fixture();
    let paths = [
        "source/download-journal.json",
        "source/download-manifest.json",
        "restore/restore-safety-requirement.json",
    ];
    let originals: Vec<_> = paths
        .iter()
        .map(|path| fs::read(f.root.join(path)).unwrap())
        .collect();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    let replay = journal
        .read_download_manifest(&f.source, f.requirement.source_artifacts())
        .unwrap();
    let view = verify(&f, &journal).unwrap();
    assert_eq!(view.selected_artifacts().count(), 1);
    assert_eq!(view.source().artifacts().len(), 2);
    assert_eq!(view.source().journal(), &replay);
    for (path, bytes) in paths.iter().zip(originals) {
        assert_eq!(fs::read(f.root.join(path)).unwrap(), bytes);
    }
    drop(journal);
    let root = f.root.clone();
    drop(f);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_restore_source_checks_unselected_original_bytes_and_never_repairs_unsafe_trees() {
    for kind in ["changed", "missing", "symlink"] {
        let f = fixture();
        let journal =
            DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
        let path = f.root.join("source/artifacts/aaaaa-aa");
        if kind == "changed" {
            fs::write(path.join("heap.bin"), b"changed original unselected bytes").unwrap();
        } else {
            fs::rename(&path, f.root.join("retained-root-artifact")).unwrap();
            if kind == "symlink" {
                symlink(f.root.join("retained-root-artifact"), &path).unwrap();
            }
        }
        let original = fs::read(journal.path()).unwrap();
        journal
            .read_download_manifest(&f.source, f.requirement.source_artifacts())
            .unwrap();
        let result = verify(&f, &journal);
        if kind == "changed" {
            assert!(matches!(
                result,
                Err(LocalRestoreSourceError::Integrity(
                    DownloadIntegrityError::Checksum(ChecksumError::ChecksumMismatch { .. })
                ))
            ));
        } else {
            assert!(result.is_err());
        }
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        drop(journal);
        let root = f.root.clone();
        drop(f);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn local_restore_source_rejects_missing_changed_and_unsafe_original_evidence() {
    for entry in [
        "source/operation-plan.json",
        "restore/operation-plan.json",
        "restore/restore-safety-requirement.json",
        "source/download-manifest.json",
        "source/download-journal.json",
    ] {
        for kind in ["missing", "changed", "symlink"] {
            let f = fixture();
            let journal =
                DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
            let path = f.root.join(entry);
            let original = fs::read(&path).unwrap();
            fs::rename(&path, f.root.join("retained-original")).unwrap();
            if kind == "symlink" {
                symlink(f.root.join("retained-original"), &path).unwrap();
            }
            if kind == "changed" {
                let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
                if entry.contains("operation-plan") {
                    value["context"]["release"] = json!("ef".repeat(32));
                } else if entry.contains("requirement") {
                    value["source_artifacts"]["hash"] = json!("ef".repeat(32));
                } else {
                    value["artifacts"][0]["snapshot_id"] = json!("changed-token");
                }
                fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            assert!(verify(&f, &journal).is_err(), "{entry}: {kind}");
            assert_eq!(
                fs::read(f.root.join("retained-original")).unwrap(),
                original
            );
            if kind == "missing" {
                assert!(!path.exists());
            }
            drop(journal);
            let root = f.root.clone();
            drop(f);
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[test]
fn local_restore_source_replaced_layouts_and_manifest_contention_retain_originals() {
    for replace_source in [false, true] {
        let f = fixture();
        let journal =
            DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
        let name = if replace_source { "source" } else { "restore" };
        fs::rename(f.root.join(name), f.root.join("retained-layout")).unwrap();
        fs::create_dir(f.root.join(name)).unwrap();
        assert!(matches!(
            verify(&f, &journal),
            Err(LocalRestoreSourceError::Requirement(
                crate::ops::persistence::RestoreSafetyPersistenceError::Plan(
                    crate::ops::persistence::OperationPlanPersistenceError::Persistence(
                        PersistenceError::LayoutChanged { .. }
                    )
                )
            ))
        ));
        drop(journal);
        let root = f.root.clone();
        drop(f);
        fs::remove_dir_all(root).unwrap();
    }
    let f = fixture();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    let lock = crate::ops::persistence::JournalLock::acquire(
        &f.root.join("source/download-manifest.json"),
    )
    .unwrap();
    assert!(matches!(
        verify(&f, &journal),
        Err(LocalRestoreSourceError::Manifest(
            DownloadManifestError::Lock(_)
        ))
    ));
    drop(lock);
    verify(&f, &journal).unwrap();
    drop(journal);
    let root = f.root.clone();
    drop(f);
    fs::remove_dir_all(root).unwrap();
}
