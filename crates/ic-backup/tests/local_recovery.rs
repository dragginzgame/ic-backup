//! Public local persistence journey; this test performs no IC effects.

use ic_backup::{
    model::artifacts::{ArtifactChecksumRecord, ChecksumError},
    ops::{
        artifacts::stage_relative_path,
        persistence::{
            ArtifactCommitOutcome, JournalLock, JournalLockError, PersistenceError,
            commit_artifact_directory, create_json_durable, read_json,
        },
    },
};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn public_primitives_preserve_intent_and_reconcile_only_matching_bytes() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("resolve test parent")
        .join(format!(
            "ic-backup-public-recovery-{}-{nonce}",
            std::process::id()
        ));
    fs::create_dir_all(root.join("source")).expect("create source");
    fs::write(root.join("source/state.bin"), b"exact snapshot bytes").expect("write source");
    let journal = root.join("intent.json");
    let guard = JournalLock::acquire(&journal).expect("acquire journal authority");
    assert!(matches!(
        JournalLock::acquire(&journal),
        Err(JournalLockError::Locked { .. })
    ));
    let temporary = root.join("staging");
    let canonical = root.join("snapshot");
    let checksum =
        stage_relative_path(&root, Path::new("source"), &temporary).expect("stage exact source");
    create_json_durable(&journal, &checksum).expect("persist exact intent");
    assert_eq!(
        commit_artifact_directory(&temporary, &canonical, checksum.hash())
            .expect("publish verified bytes"),
        ArtifactCommitOutcome::Published
    );
    let retained: ArtifactChecksumRecord =
        read_json(&journal, 4096).expect("recover original checksum record");
    assert_eq!(retained, checksum);
    assert_eq!(
        commit_artifact_directory(&temporary, &canonical, retained.hash())
            .expect("adopt after lost publication reply"),
        ArtifactCommitOutcome::Recovered
    );
    assert_eq!(
        fs::read(canonical.join("state.bin")).expect("read exact bytes"),
        b"exact snapshot bytes"
    );
    fs::write(canonical.join("state.bin"), b"changed").expect("simulate later corruption");
    assert!(matches!(
        commit_artifact_directory(&temporary, &canonical, retained.hash()),
        Err(PersistenceError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    let unchanged_intent: ArtifactChecksumRecord =
        read_json(&journal, 4096).expect("retain original intent");
    assert_eq!(unchanged_intent, retained);
    drop(guard);
    fs::remove_dir_all(root).expect("remove successful fixture");
}
