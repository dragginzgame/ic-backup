//! Public local persistence journey; this test performs no IC effects.

use ic_backup::{
    model::{
        artifacts::{ArtifactChecksumRecord, ChecksumError},
        command_custody::{CommandCustodyRecord, MAX_COMMAND_CUSTODY_RECORD_BYTES},
    },
    ops::{
        artifacts::stage_relative_path,
        persistence::{
            ArtifactCommitOutcome, BackupLayoutGuard, CommandLifetimeLock, CommandQuiescenceGuard,
            JournalLock, JournalLockError, PersistenceError, commit_artifact_directory,
            create_json_durable, read_json,
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
    let layout = BackupLayoutGuard::acquire(&root).expect("exclude other layout users");
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
    let reference = layout
        .retain_restore(&root.join("external-restore.json"), retained.hash())
        .expect("retain before restore journal publication");
    assert!(!reference.journal().exists());
    let mut command =
        CommandLifetimeLock::acquire(&journal, 0).expect("acquire native command custody");
    let custody = command.record().clone();
    let custody_path = root.join("command-custody.json");
    create_json_durable(&custody_path, &custody).expect("persist exact custody before dispatch");
    let mut native = std::process::Command::new("python3");
    native.args(["-c", "pass"]);
    assert!(
        command
            .spawn(native)
            .expect("spawn native fixture")
            .wait()
            .expect("reap native fixture")
            .success()
    );
    let quiescent = command.finish().expect("fresh exclusive native quiescence");
    let retained_custody: CommandCustodyRecord =
        read_json(&custody_path, MAX_COMMAND_CUSTODY_RECORD_BYTES)
            .expect("recover custody identity");
    assert_eq!(quiescent.record(), &retained_custody);
    drop(quiescent);
    let quiescent = CommandQuiescenceGuard::acquire(&retained_custody)
        .expect("reobserve retained exact sidecar");
    assert_eq!(quiescent.record(), &custody);
    drop(quiescent);
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
    assert_eq!(
        layout
            .restore_references()
            .expect("retain source dependency despite changed bytes")
            .entries(),
        &[reference]
    );
    drop(layout);
    drop(guard);
    fs::remove_dir_all(root).expect("remove successful fixture");
}
