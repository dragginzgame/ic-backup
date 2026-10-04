use super::*;
use crate::{
    ops::artifacts::checksum_directory,
    test_support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn missing_or_non_sibling_paths_reject_without_publication() {
    let root = temp_dir("ic-backup-publication-paths");
    fs::create_dir(&root).expect("create parent");
    let temporary = root.join("staging");
    let canonical = root.join("snapshot");
    let digest = crate::model::artifacts::ArtifactChecksumRecord::from_bytes(b"");
    assert!(matches!(
        commit_artifact_directory(&temporary, &canonical, digest.hash()),
        Err(PersistenceError::ArtifactCommitPathMissing { .. })
    ));
    assert!(matches!(
        commit_artifact_directory(&temporary, &temporary, digest.hash()),
        Err(PersistenceError::ArtifactCommitPathMismatch { .. })
    ));
    assert!(matches!(
        commit_artifact_directory(&temporary, &root.join("other/snapshot"), digest.hash()),
        Err(PersistenceError::ArtifactCommitPathMismatch { .. })
    ));
    fs::remove_dir_all(root).expect("remove successful fixture");
}

#[test]
fn process_death_recovers_only_verified_bytes() {
    use super::supported::{ArtifactCommitStep, commit_with_hook};
    const ROOT_ENV: &str = "IC_BACKUP_TEST_ARTIFACT_CRASH_ROOT";
    const SIDE_ENV: &str = "IC_BACKUP_TEST_ARTIFACT_CRASH_SIDE";
    if let Some(root) = std::env::var_os(ROOT_ENV) {
        let root = PathBuf::from(root);
        let side = std::env::var(SIDE_ENV).expect("child barrier");
        let temporary = root.join("staging");
        let expected = checksum(&temporary);
        commit_with_hook(&temporary, &root.join("snapshot"), &expected, |step, _| {
            if matches!(
                (side.as_str(), step),
                ("before", ArtifactCommitStep::Publication)
                    | ("after", ArtifactCommitStep::PublicationDurable)
            ) {
                hold_at_acknowledged_barrier(&root);
            }
            Ok(())
        })
        .expect("publish at crash barrier");
        panic!("child passed acknowledged crash barrier");
    }
    for side in ["before", "after"] {
        let root = temp_dir("ic-backup-artifact-crash");
        let temporary = root.join("staging");
        let canonical = root.join("snapshot");
        fs::create_dir_all(temporary.join("nested")).expect("create artifact tree");
        fs::write(temporary.join("nested/state.bin"), b"snapshot bytes").expect("write artifact");
        let expected = checksum(&temporary);
        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", "ops::persistence::artifact_commit::regressions::process_death_recovers_only_verified_bytes", "--nocapture"])
            .env(ROOT_ENV, &root).env(SIDE_ENV, side).spawn().expect("spawn crash child");
        kill_child_at_acknowledged_barrier(&mut child, &root);
        let outcome = commit_artifact_directory(&temporary, &canonical, &expected)
            .expect("reconcile exact artifact");
        assert_eq!(
            outcome,
            if side == "after" {
                ArtifactCommitOutcome::Recovered
            } else {
                ArtifactCommitOutcome::Published
            }
        );
        assert_eq!(checksum(&canonical), expected);
        fs::remove_dir_all(root).expect("remove verified crash fixture");
    }
}

fn checksum(path: &Path) -> String {
    checksum_directory(path)
        .expect("checksum tree")
        .hash()
        .to_owned()
}
