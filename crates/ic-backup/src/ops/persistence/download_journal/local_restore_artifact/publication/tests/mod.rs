//! Native exact publication, retained failures and acknowledged process-death recovery.

use super::*;
use crate::{
    model::artifacts::ArtifactChecksumRecord,
    ops::artifacts::checksum_directory,
    ops::persistence::download_journal::local_restore_artifact::tests::{Fixture, fixture, stage},
    test_support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir},
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
    process::Command,
};

fn publish<'a>(
    f: &'a Fixture,
    journal: &'a DownloadJournalGuard<'_>,
) -> Result<
    (LocalRestoreArtifactView<'a>, ArtifactCommitOutcome),
    LocalRestoreArtifactPublicationError,
> {
    journal.publish_staged_local_restore_artifact(
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
    journal.verify_published_local_restore_artifact(
        &f.restore_layout,
        &f.restore,
        &f.source,
        &f.requirement,
        7,
    )
}
fn finish(f: Fixture) {
    let root = f.root.clone();
    drop(f);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn original_restore_copy_publishes_recovers_and_checks_without_source_tree_reads() {
    let f = fixture();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    let original = fs::read(journal.path()).unwrap();
    let requirement = fs::read(f.root.join("restore/restore-safety-requirement.json")).unwrap();
    let staging = stage(&f, &journal).unwrap().path().to_path_buf();
    fs::rename(
        f.root.join("source/artifacts"),
        f.root.join("retained-source-artifacts"),
    )
    .unwrap();
    let (view, outcome) = publish(&f, &journal).unwrap();
    assert_eq!(outcome, ArtifactCommitOutcome::Published);
    assert_eq!(view.path(), f.root.join("restore/restore-artifact-7"));
    assert_eq!(view.operation().operation_sequence(), 7);
    assert_eq!(
        checksum_directory(view.path()).unwrap(),
        *view.artifact().checksum().unwrap()
    );
    assert_eq!(
        fs::metadata(view.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(view.path().join("nested/heap.bin"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(!staging.exists());
    drop(view);
    assert_eq!(
        publish(&f, &journal).unwrap().1,
        ArtifactCommitOutcome::Recovered
    );
    verify(&f, &journal).unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), original);
    assert_eq!(
        fs::read(f.root.join("restore/restore-safety-requirement.json")).unwrap(),
        requirement
    );
    assert!(matches!(
        journal.publish_staged_local_restore_artifact(
            &f.restore_layout,
            &f.restore,
            &f.source,
            &f.requirement,
            u64::MAX
        ),
        Err(LocalRestoreArtifactPublicationError::Admission(
            LocalRestoreArtifactError::Operation(_)
        ))
    ));
    drop(journal);
    finish(f);
}

#[test]
fn missing_conflicting_changed_and_unsafe_restore_copies_remain_retained() {
    for kind in ["missing", "both", "changed", "symlink"] {
        let f = fixture();
        let journal =
            DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
        let temporary = staged_path(&f.restore_layout, 7);
        let canonical = published_path(&f.restore_layout, 7);
        if kind != "missing" {
            stage(&f, &journal).unwrap();
        }
        match kind {
            "both" => {
                fs::create_dir(&canonical).unwrap();
                fs::write(canonical.join("other"), b"retained conflict").unwrap();
            }
            "changed" => fs::write(temporary.join("nested/heap.bin"), b"changed copy").unwrap(),
            "symlink" => {
                fs::rename(&temporary, f.root.join("retained-staged-copy")).unwrap();
                symlink(f.root.join("retained-staged-copy"), &temporary).unwrap();
            }
            _ => {}
        }
        let error = publish(&f, &journal).unwrap_err();
        assert!(match kind {
            "missing" => matches!(
                error,
                LocalRestoreArtifactPublicationError::Publication(
                    PersistenceError::ArtifactCommitPathMissing { .. }
                )
            ),
            "both" => matches!(
                error,
                LocalRestoreArtifactPublicationError::Publication(
                    PersistenceError::ArtifactCommitPathConflict { .. }
                )
            ),
            "changed" => matches!(
                error,
                LocalRestoreArtifactPublicationError::Publication(PersistenceError::Checksum(_))
            ),
            _ => matches!(error, LocalRestoreArtifactPublicationError::Publication(_)),
        });
        assert_eq!(temporary.symlink_metadata().is_ok(), kind != "missing");
        assert_eq!(canonical.exists(), kind == "both");
        if kind == "both" {
            assert_eq!(
                fs::read(canonical.join("other")).unwrap(),
                b"retained conflict"
            );
        }
        if kind == "changed" {
            assert_eq!(
                fs::read(temporary.join("nested/heap.bin")).unwrap(),
                b"changed copy"
            );
        }
        drop(journal);
        finish(f);
    }
}

#[test]
fn lost_publication_reply_and_closing_original_drift_preserve_canonical_recovery() {
    for kind in ["before", "after", "original-drift", "published-drift"] {
        let f = fixture();
        let journal =
            DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
        stage(&f, &journal).unwrap();
        let original = fs::read(journal.path()).unwrap();
        let requirement_path = f.root.join("restore/restore-safety-requirement.json");
        let requirement = fs::read(&requirement_path).unwrap();
        let result = journal.publish_restore_artifact_with(
            &f.restore_layout,
            &f.restore,
            &f.source,
            &f.requirement,
            7,
            |temporary, canonical, expected| {
                if kind == "before" {
                    return Err(io::Error::other("lost before publication").into());
                }
                let outcome = commit_artifact_directory(temporary, canonical, expected)?;
                if kind == "original-drift" {
                    let mut changed: serde_json::Value =
                        serde_json::from_slice(&requirement).unwrap();
                    changed["source_artifacts"] = serde_json::to_value(
                        ArtifactChecksumRecord::from_bytes(b"changed source binding"),
                    )
                    .unwrap();
                    fs::write(&requirement_path, serde_json::to_vec(&changed).unwrap())?;
                    return Ok(outcome);
                }
                if kind == "published-drift" {
                    fs::write(canonical.join("nested/heap.bin"), b"changed published copy")?;
                    return Ok(outcome);
                }
                Err(io::Error::other("lost after durable publication").into())
            },
        );
        assert!(match kind {
            "original-drift" => matches!(
                result,
                Err(LocalRestoreArtifactPublicationError::Admission(
                    LocalRestoreArtifactError::Source(_)
                ))
            ),
            "published-drift" => matches!(
                result,
                Err(LocalRestoreArtifactPublicationError::Admission(
                    LocalRestoreArtifactError::Checksum(_)
                ))
            ),
            _ => matches!(
                result,
                Err(LocalRestoreArtifactPublicationError::Publication(
                    PersistenceError::Io(_)
                ))
            ),
        });
        assert_eq!(staged_path(&f.restore_layout, 7).exists(), kind == "before");
        assert_eq!(
            published_path(&f.restore_layout, 7).exists(),
            kind != "before"
        );
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        if kind == "original-drift" {
            fs::write(&requirement_path, &requirement).unwrap();
        }
        if kind == "published-drift" {
            assert!(matches!(
                publish(&f, &journal),
                Err(LocalRestoreArtifactPublicationError::Publication(
                    PersistenceError::Checksum(_)
                ))
            ));
            assert!(matches!(
                verify(&f, &journal),
                Err(LocalRestoreArtifactError::Checksum(_))
            ));
        } else {
            assert_eq!(
                publish(&f, &journal).unwrap().1,
                if kind == "before" {
                    ArtifactCommitOutcome::Published
                } else {
                    ArtifactCommitOutcome::Recovered
                }
            );
        }
        drop(journal);
        finish(f);
    }
}

#[test]
fn original_admission_and_shared_operation_contention_precede_publication() {
    let f = fixture();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    let temporary = stage(&f, &journal).unwrap().path().to_path_buf();
    let lock = JournalLock::acquire(&temporary).unwrap();
    assert!(matches!(
        publish(&f, &journal),
        Err(LocalRestoreArtifactPublicationError::Admission(
            LocalRestoreArtifactError::Lock(_)
        ))
    ));
    assert!(matches!(
        verify(&f, &journal),
        Err(LocalRestoreArtifactError::Lock(_))
    ));
    drop(lock);
    let requirement_path = f.root.join("restore/restore-safety-requirement.json");
    fs::rename(&requirement_path, f.root.join("retained-requirement.json")).unwrap();
    assert!(matches!(
        publish(&f, &journal),
        Err(LocalRestoreArtifactPublicationError::Admission(
            LocalRestoreArtifactError::Source(_)
        ))
    ));
    assert!(temporary.is_dir());
    assert!(!published_path(&f.restore_layout, 7).exists());
    drop(journal);
    finish(f);
}

#[test]
fn replaced_restore_layout_rejects_before_writing_into_the_replacement() {
    let f = fixture();
    let journal = DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
    stage(&f, &journal).unwrap();
    let retained = f.root.join("retained-restore-layout");
    fs::rename(f.restore_layout.root(), &retained).unwrap();
    fs::create_dir(f.restore_layout.root()).unwrap();
    assert!(matches!(
        publish(&f, &journal),
        Err(LocalRestoreArtifactPublicationError::Admission(
            LocalRestoreArtifactError::Source(_)
        ))
    ));
    assert!(matches!(
        verify(&f, &journal),
        Err(LocalRestoreArtifactError::Source(_))
    ));
    assert_eq!(fs::read_dir(f.restore_layout.root()).unwrap().count(), 0);
    assert!(retained.join("restore-artifact-7.tmp").is_dir());
    drop(journal);
    finish(f);
}

#[test]
fn acknowledged_owner_death_before_or_after_publication_recovers_exact_original_copy() {
    const ROOT: &str = "IC_BACKUP_RESTORE_PUBLICATION_ROOT";
    const AFTER: &str = "IC_BACKUP_RESTORE_PUBLICATION_AFTER";
    const HANDSHAKE: &str = "IC_BACKUP_RESTORE_PUBLICATION_HANDSHAKE";
    if let Some(root) = std::env::var_os(ROOT) {
        let root = PathBuf::from(root);
        let restore = crate::test_support::restore_safety::plan();
        let source = crate::test_support::local_restore_source::source();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let restore_layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let journal = DownloadJournalGuard::open(&source_layout, source.digest().hash()).unwrap();
        let requirement = crate::test_support::local_restore_source::requirement(
            &restore,
            &source,
            journal.record().unwrap(),
        );
        let handshake = PathBuf::from(std::env::var_os(HANDSHAKE).unwrap());
        let after = std::env::var(AFTER).unwrap() == "yes";
        journal
            .publish_restore_artifact_with(
                &restore_layout,
                &restore,
                &source,
                &requirement,
                7,
                |temporary, canonical, expected| {
                    if !after {
                        hold_at_acknowledged_barrier(&handshake);
                    }
                    commit_artifact_directory(temporary, canonical, expected)?;
                    hold_at_acknowledged_barrier(&handshake)
                },
            )
            .unwrap();
        panic!("publication child passed crash barrier");
    }
    for after in [false, true] {
        let f = fixture();
        let journal =
            DownloadJournalGuard::open(&f.source_layout, f.source.digest().hash()).unwrap();
        stage(&f, &journal).unwrap();
        let original = fs::read(journal.path()).unwrap();
        drop(journal);
        let root = f.root.clone();
        drop(f);
        let handshake = temp_dir("ic-backup-restore-publication-handshake");
        fs::create_dir(&handshake).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ops::persistence::download_journal::local_restore_artifact::publication::tests::acknowledged_owner_death_before_or_after_publication_recovers_exact_original_copy", "--nocapture"])
            .env(ROOT, &root).env(AFTER, if after { "yes" } else { "no" }).env(HANDSHAKE, &handshake).spawn().unwrap();
        kill_child_at_acknowledged_barrier(&mut child, &handshake);
        let restore = crate::test_support::restore_safety::plan();
        let source = crate::test_support::local_restore_source::source();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let restore_layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let journal = DownloadJournalGuard::open(&source_layout, source.digest().hash()).unwrap();
        let requirement = crate::test_support::local_restore_source::requirement(
            &restore,
            &source,
            journal.record().unwrap(),
        );
        assert_eq!(root.join("restore/restore-artifact-7.tmp").exists(), !after);
        let (_, outcome) = journal
            .publish_staged_local_restore_artifact(
                &restore_layout,
                &restore,
                &source,
                &requirement,
                7,
            )
            .unwrap();
        assert_eq!(
            outcome,
            if after {
                ArtifactCommitOutcome::Recovered
            } else {
                ArtifactCommitOutcome::Published
            }
        );
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        drop(journal);
        drop(restore_layout);
        drop(source_layout);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(handshake).unwrap();
    }
}
