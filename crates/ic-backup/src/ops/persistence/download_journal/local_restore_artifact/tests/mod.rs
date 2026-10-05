//! Private descriptor copies, conservative partial evidence and explicit local recovery.

use super::*;
use crate::{
    model::{artifacts::ArtifactChecksumRecord, restore_safety::RestoreSafetyRequirementRecord},
    ops::persistence::{create_operation_plan, create_restore_safety_requirement},
    test_support::{
        hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, local_restore_source,
        restore_safety, temp_dir,
    },
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
    process::Command,
};

struct Fixture {
    root: PathBuf,
    restore_layout: BackupLayoutGuard,
    source_layout: BackupLayoutGuard,
    restore: OperationPlanRecord,
    source: OperationPlanRecord,
    requirement: RestoreSafetyRequirementRecord,
}
fn fixture() -> Fixture {
    let root = temp_dir("ic-backup-local-restore-artifact");
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
        let staging = root.join(format!("source/artifacts/{target}.tmp/nested"));
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
fn stage<'a>(
    f: &'a Fixture,
    journal: &'a DownloadJournalGuard<'_>,
) -> Result<LocalRestoreArtifactView<'a>, LocalRestoreArtifactError> {
    journal.stage_local_restore_artifact(
        &f.restore_layout,
        &f.restore,
        &f.source,
        &f.requirement,
        7,
    )
}
fn verify<'a>(
    f: &'a Fixture,
    journal: &'a DownloadJournalGuard<'_>,
) -> Result<LocalRestoreArtifactView<'a>, LocalRestoreArtifactError> {
    journal.verify_staged_local_restore_artifact(
        &f.restore_layout,
        &f.restore,
        &f.source,
        &f.requirement,
        7,
    )
}

#[test]
fn local_restore_artifact_exact_private_copy_preserves_source_and_originals_on_drop() {
    let f = fixture();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    let original = fs::read(journal.path()).unwrap();
    let requirement = fs::read(f.root.join("restore/restore-safety-requirement.json")).unwrap();
    let view = stage(&f, &journal).unwrap();
    let path = view.path().to_path_buf();
    assert_eq!(view.operation().operation_sequence(), 7);
    assert_eq!(view.artifact().canister_id(), view.operation().target());
    assert_eq!(
        view.artifact().snapshot_id(),
        format!("Original-{}", view.operation().target())
    );
    assert_eq!(
        checksum_directory(&path).unwrap(),
        *view.artifact().checksum().unwrap()
    );
    for directory in [&path, &path.join("nested")] {
        assert_eq!(
            fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    assert_eq!(
        fs::metadata(path.join("nested/heap.bin"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(view.source().requirement(), &f.requirement);
    drop(view);
    assert!(path.exists());
    assert_eq!(fs::read(journal.path()).unwrap(), original);
    assert_eq!(
        fs::read(f.root.join("restore/restore-safety-requirement.json")).unwrap(),
        requirement
    );
    assert!(
        matches!(stage(&f,&journal),Err(LocalRestoreArtifactError::Artifact(ArtifactError::Io(error))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        journal.stage_local_restore_artifact(
            &f.restore_layout,
            &f.restore,
            &f.source,
            &f.requirement,
            u64::MAX
        ),
        Err(LocalRestoreArtifactError::Operation(_))
    ));
    verify(&f, &journal).unwrap();
    drop(journal);
    let root = f.root.clone();
    drop(f);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_restore_artifact_recovery_checks_copy_without_rereading_or_recreating_source_trees() {
    let f = fixture();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    let path = stage(&f, &journal).unwrap().path().to_path_buf();
    let bytes = fs::read(path.join("nested/heap.bin")).unwrap();
    fs::rename(
        f.root.join("source/artifacts"),
        f.root.join("retained-source-artifacts"),
    )
    .unwrap();
    verify(&f, &journal).unwrap();
    assert!(
        journal
            .stage_local_restore_artifact(
                &f.restore_layout,
                &f.restore,
                &f.source,
                &f.requirement,
                8
            )
            .is_err()
    );
    assert!(!f.root.join("restore/restore-artifact-8.tmp").exists());
    fs::write(path.join("nested/heap.bin"), b"changed copy bytes").unwrap();
    assert!(matches!(
        verify(&f, &journal),
        Err(LocalRestoreArtifactError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    assert_eq!(
        fs::read(path.join("nested/heap.bin")).unwrap(),
        b"changed copy bytes"
    );
    fs::write(path.join("nested/heap.bin"), bytes).unwrap();
    verify(&f, &journal).unwrap();
    let moved = f.root.join("retained-copy");
    fs::rename(&path, &moved).unwrap();
    symlink(&moved, &path).unwrap();
    assert!(matches!(
        verify(&f, &journal),
        Err(LocalRestoreArtifactError::Artifact(_))
    ));
    drop(journal);
    let root = f.root.clone();
    drop(f);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_restore_artifact_partial_and_lost_copy_replies_preserve_unfinished_evidence() {
    for kind in ["before", "partial", "after", "changed-source"] {
        let f = fixture();
        let journal =
            DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
        let original = fs::read(journal.path()).unwrap();
        let path = staged_path(&f.restore_layout, 7);
        let result = journal.stage_restore_artifact_with(
            &f.restore_layout,
            &f.restore,
            &f.source,
            &f.requirement,
            7,
            |root, relative, destination| {
                if kind == "partial" {
                    fs::create_dir(destination)?;
                    fs::write(destination.join("partial.bin"), b"partial retained bytes")?;
                }
                if kind == "after" {
                    stage_relative_path(root, relative, destination)?;
                }
                if kind == "changed-source" {
                    fs::write(
                        root.join(relative).join("nested/heap.bin"),
                        b"changed between validation and copy",
                    )?;
                    return stage_relative_path(root, relative, destination);
                }
                Err(io::Error::other("lost or incomplete local copy response").into())
            },
        );
        assert!(result.is_err());
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        assert_eq!(path.exists(), kind != "before");
        if kind == "after" {
            verify(&f, &journal).unwrap();
        } else {
            assert!(verify(&f, &journal).is_err());
        }
        if kind == "partial" {
            assert_eq!(
                fs::read(path.join("partial.bin")).unwrap(),
                b"partial retained bytes"
            );
        }
        if kind == "changed-source" {
            assert!(matches!(
                result,
                Err(LocalRestoreArtifactError::Checksum(
                    ChecksumError::ChecksumMismatch { .. }
                ))
            ));
        }
        drop(journal);
        let root = f.root.clone();
        drop(f);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn local_restore_artifact_closing_original_admission_and_contention_preserve_copied_bytes() {
    let f = fixture();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    let path = staged_path(&f.restore_layout, 7);
    let requirement_path = f.root.join("restore/restore-safety-requirement.json");
    let original = fs::read(&requirement_path).unwrap();
    let result = journal.stage_restore_artifact_with(
        &f.restore_layout,
        &f.restore,
        &f.source,
        &f.requirement,
        7,
        |root, relative, destination| {
            let copied = stage_relative_path(root, relative, destination)?;
            let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
            changed["source_artifacts"] = serde_json::to_value(ArtifactChecksumRecord::from_bytes(
                b"different retained binding",
            ))
            .unwrap();
            fs::write(&requirement_path, serde_json::to_vec(&changed).unwrap())?;
            Ok(copied)
        },
    );
    assert!(matches!(result, Err(LocalRestoreArtifactError::Source(_))));
    assert!(path.is_dir());
    fs::write(&requirement_path, original).unwrap();
    verify(&f, &journal).unwrap();
    let lock = JournalLock::acquire(&path).unwrap();
    assert!(matches!(
        verify(&f, &journal),
        Err(LocalRestoreArtifactError::Lock(_))
    ));
    drop(lock);
    drop(journal);
    let root = f.root.clone();
    drop(f);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_restore_artifact_acknowledged_death_before_or_after_copy_recovers_without_cleanup() {
    const ROOT: &str = "IC_BACKUP_RESTORE_ARTIFACT_CHILD_ROOT";
    const AFTER: &str = "IC_BACKUP_RESTORE_ARTIFACT_CHILD_AFTER";
    const HANDSHAKE: &str = "IC_BACKUP_RESTORE_ARTIFACT_CHILD_HANDSHAKE";
    if let Some(root) = std::env::var_os(ROOT) {
        let root = PathBuf::from(root);
        let handshake = PathBuf::from(std::env::var_os(HANDSHAKE).unwrap());
        let restore = restore_safety::plan();
        let source = local_restore_source::source();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let restore_layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let journal = DownloadJournalGuard::open(&source_layout, source.digest().hash()).unwrap();
        let requirement =
            local_restore_source::requirement(&restore, &source, journal.record().unwrap());
        let after = std::env::var(AFTER).unwrap() == "yes";
        journal
            .stage_restore_artifact_with(
                &restore_layout,
                &restore,
                &source,
                &requirement,
                7,
                |root, relative, destination| {
                    if !after {
                        hold_at_acknowledged_barrier(&handshake);
                    }
                    stage_relative_path(root, relative, destination)?;
                    hold_at_acknowledged_barrier(&handshake)
                },
            )
            .unwrap();
        panic!("restore artifact child passed acknowledged crash barrier");
    }
    for after in [false, true] {
        let f = fixture();
        let root = f.root.clone();
        let original = fs::read(root.join("source/download-journal.json")).unwrap();
        drop(f);
        let handshake = temp_dir("ic-backup-restore-artifact-handshake");
        fs::create_dir(&handshake).unwrap();
        let mut child=Command::new(std::env::current_exe().unwrap())
            .args(["--exact","ops::persistence::download_journal::local_restore_artifact::tests::local_restore_artifact_acknowledged_death_before_or_after_copy_recovers_without_cleanup","--nocapture"])
            .env(ROOT,&root).env(AFTER,if after {"yes"} else {"no"}).env(HANDSHAKE,&handshake).spawn().unwrap();
        kill_child_at_acknowledged_barrier(&mut child, &handshake);
        let restore = restore_safety::plan();
        let source = local_restore_source::source();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let restore_layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let journal = DownloadJournalGuard::open(&source_layout, source.digest().hash()).unwrap();
        let requirement =
            local_restore_source::requirement(&restore, &source, journal.record().unwrap());
        assert_eq!(root.join("restore/restore-artifact-7.tmp").exists(), after);
        if !after {
            journal
                .stage_local_restore_artifact(&restore_layout, &restore, &source, &requirement, 7)
                .unwrap();
        }
        journal
            .verify_staged_local_restore_artifact(
                &restore_layout,
                &restore,
                &source,
                &requirement,
                7,
            )
            .unwrap();
        assert_eq!(
            fs::read(root.join("source/download-journal.json")).unwrap(),
            original
        );
        drop(journal);
        drop(restore_layout);
        drop(source_layout);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(handshake).unwrap();
    }
}
