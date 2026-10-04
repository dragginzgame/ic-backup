use super::{ArtifactCommitOutcome, supported::*};
use crate::{
    ops::{artifacts::checksum_directory, persistence::PersistenceError},
    test_support::temp_dir,
};

use std::{fs, io};

#[test]
fn publishes_and_recovers_the_exact_verified_tree() {
    let root = temp_dir("canic-backup-artifact-commit");
    let temporary = root.join("snapshot.tmp");
    let canonical = root.join("snapshot");
    write_tree(&temporary);
    let expected = checksum(&temporary);

    let published = commit_with_hook(&temporary, &canonical, &expected, |_, _| Ok(()))
        .expect("publish artifact");
    assert_eq!(published, ArtifactCommitOutcome::Published);
    assert!(!temporary.exists());
    assert_eq!(checksum(&canonical), expected);

    let recovered = commit_with_hook(&temporary, &canonical, &expected, |_, _| Ok(()))
        .expect("recover artifact");
    assert_eq!(recovered, ArtifactCommitOutcome::Recovered);
    assert_eq!(checksum(&canonical), expected);

    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn rejects_existing_destinations_and_unsupported_entries() {
    let conflict_root = temp_dir("canic-backup-artifact-conflict");
    let conflict_temporary = conflict_root.join("snapshot.tmp");
    let conflict_canonical = conflict_root.join("snapshot");
    write_tree(&conflict_temporary);
    write_tree(&conflict_canonical);
    let expected = checksum(&conflict_temporary);

    let conflict = commit_with_hook(
        &conflict_temporary,
        &conflict_canonical,
        &expected,
        |_, _| Ok(()),
    )
    .expect_err("existing destination must reject");
    assert!(matches!(
        conflict,
        PersistenceError::ArtifactCommitPathConflict { .. }
    ));

    let symlink_root = temp_dir("canic-backup-artifact-symlink");
    let symlink_temporary = symlink_root.join("snapshot.tmp");
    let symlink_canonical = symlink_root.join("snapshot");
    write_tree(&symlink_temporary);
    std::os::unix::fs::symlink(
        symlink_temporary.join("root.txt"),
        symlink_temporary.join("linked.txt"),
    )
    .expect("create artifact symlink");

    let symlink = commit_with_hook(&symlink_temporary, &symlink_canonical, &expected, |_, _| {
        Ok(())
    })
    .expect_err("symlink must reject");
    assert!(matches!(
        symlink,
        PersistenceError::UnsupportedArtifactEntry { .. }
    ));

    fs::remove_dir_all(conflict_root).expect("remove conflict root");
    fs::remove_dir_all(symlink_root).expect("remove symlink root");
}

#[test]
fn publication_race_cannot_replace_a_new_destination() {
    let root = temp_dir("canic-backup-artifact-publication-race");
    let temporary = root.join("snapshot.tmp");
    let canonical = root.join("snapshot");
    write_tree(&temporary);
    let expected = checksum(&temporary);

    let error = commit_with_hook(&temporary, &canonical, &expected, |step, _| {
        if step == ArtifactCommitStep::Publication {
            fs::create_dir(&canonical)?;
            fs::write(canonical.join("other.txt"), b"other")?;
        }
        Ok(())
    })
    .expect_err("atomic no-replace must reject a publication race");

    assert!(matches!(
        error,
        PersistenceError::Io(ref source)
            if source.kind() == io::ErrorKind::AlreadyExists
    ));
    assert!(temporary.exists());
    assert_eq!(
        fs::read(canonical.join("other.txt")).expect("read raced destination"),
        b"other"
    );

    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn changed_temporary_tree_cannot_be_published_under_an_old_checksum() {
    let root = temp_dir("canic-backup-artifact-changed-before-publication");
    let temporary = root.join("snapshot.tmp");
    let canonical = root.join("snapshot");
    write_tree(&temporary);
    let expected = checksum(&temporary);
    fs::write(temporary.join("root.txt"), b"changed snapshot").expect("change temporary artifact");

    let error = commit_with_hook(&temporary, &canonical, &expected, |_, _| Ok(()))
        .expect_err("changed temporary artifact must reject");

    assert!(matches!(
        error,
        PersistenceError::Checksum(crate::model::artifacts::ChecksumError::ChecksumMismatch { .. })
    ));
    assert!(temporary.exists());
    assert!(!canonical.exists());

    fs::remove_dir_all(root).expect("remove changed temporary root");
}

#[test]
fn injected_commit_failures_never_expose_a_partial_tree() {
    let steps = [
        ArtifactCommitStep::RegularFileSync,
        ArtifactCommitStep::NestedDirectorySync,
        ArtifactCommitStep::RootDirectorySync,
        ArtifactCommitStep::Publication,
        ArtifactCommitStep::ParentDirectorySync,
        ArtifactCommitStep::PublicationDurable,
    ];

    for step in steps {
        let root = temp_dir(&format!("canic-backup-artifact-failure-{step:?}"));
        let temporary = root.join("snapshot.tmp");
        let canonical = root.join("snapshot");
        write_tree(&temporary);
        let expected = checksum(&temporary);
        let mut failed = false;

        let error = commit_with_hook(&temporary, &canonical, &expected, |current, _| {
            if current == step && !failed {
                failed = true;
                return Err(io::Error::other("injected commit failure"));
            }
            Ok(())
        })
        .expect_err("injected step must fail");
        assert!(matches!(error, PersistenceError::Io(_)));

        if matches!(
            step,
            ArtifactCommitStep::ParentDirectorySync | ArtifactCommitStep::PublicationDurable
        ) {
            assert!(!temporary.exists());
            assert_eq!(checksum(&canonical), expected);
            let recovered = commit_with_hook(&temporary, &canonical, &expected, |_, _| Ok(()))
                .expect("recover post-publication artifact");
            assert_eq!(recovered, ArtifactCommitOutcome::Recovered);
        } else {
            assert!(temporary.exists());
            assert!(!canonical.exists());
        }

        fs::remove_dir_all(root).expect("remove failure root");
    }
}

fn write_tree(root: &std::path::Path) {
    fs::create_dir_all(root.join("nested")).expect("create artifact tree");
    fs::write(root.join("root.txt"), b"root snapshot").expect("write root artifact");
    fs::write(root.join("nested/state.bin"), b"nested snapshot").expect("write nested artifact");
}

fn checksum(path: &std::path::Path) -> String {
    checksum_directory(path)
        .expect("checksum artifact")
        .hash()
        .to_owned()
}
