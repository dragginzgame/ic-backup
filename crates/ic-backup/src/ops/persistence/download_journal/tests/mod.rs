//! Fresh local lifecycle, uncertainty and process-death evidence; no IC executor.

use super::*;
use crate::model::artifacts::ArtifactChecksumRecord;
use crate::ops::artifacts::checksum_directory;
use crate::{
    model::{artifacts::ChecksumError, download_journal::ArtifactStateRecord},
    tests::support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir},
};
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    process::Command,
};

const CANISTER: &str = "aaaaa-aa";
const SNAPSHOT: &str = "snap-1";
const INTENT: &str = "abababababababababababababababababababababababababababababababab";

fn request() -> DownloadArtifactRequest {
    DownloadArtifactRequest {
        canister_id: CANISTER.to_owned(),
        snapshot_id: SNAPSHOT.to_owned(),
        snapshot_taken_at_timestamp: 17,
        snapshot_total_size_bytes: 123,
    }
}

fn fixture() -> PathBuf {
    let root = temp_dir("ic-backup-download-journal");
    fs::create_dir_all(root.join("artifacts/aaaaa-aa.tmp/nested")).expect("staged fixture");
    fs::write(
        root.join("artifacts/aaaaa-aa.tmp/nested/heap.bin"),
        b"opaque snapshot bytes",
    )
    .expect("exact bytes");
    root
}

// Native filesystem fixture: seed a checksum-bound state, without claiming IC coverage.
pub(super) fn retain_verified_fixture(
    guard: &mut DownloadJournalGuard<'_>,
    canister: &str,
    snapshot: &str,
) -> Result<(), DownloadJournalError> {
    let entry = guard.record()?.artifact(canister, snapshot)?;
    let checksum = checksum_directory(&guard.layout.root().join(entry.staging_path()))?;
    let next = guard.next(
        canister,
        snapshot,
        ArtifactTransition::ChecksumVerified(checksum),
    )?;
    guard.store(next, write_encoded)
}

#[test]
fn reopen_validates_exact_digest_text_before_locking_without_changing_evidence() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let guard = DownloadJournalGuard::create(&layout, INTENT, request()).unwrap();
    let bytes = fs::read(guard.path()).unwrap();
    let malformed = "Bad SHA-256 input";
    assert!(
        matches!(DownloadJournalGuard::open(&layout, malformed), Err(DownloadJournalError::Record(DownloadJournalRecordError::Checksum(ChecksumError::InvalidHash(original)))) if original == malformed)
    );
    assert!(matches!(
        DownloadJournalGuard::open(&layout, &INTENT.to_ascii_uppercase()),
        Err(DownloadJournalError::Lock(JournalLockError::Locked { .. }))
    ));
    assert_eq!(fs::read(guard.path()).unwrap(), bytes);
    drop(guard);
    let guard = DownloadJournalGuard::open(&layout, &INTENT.to_ascii_uppercase()).unwrap();
    assert_eq!(guard.record().unwrap().intent(), INTENT);
    assert_eq!(fs::read(guard.path()).unwrap(), bytes);
    drop(guard);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn exact_guarded_lifecycle_and_local_terminal_replay_preserve_bytes() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let mut guard =
        DownloadJournalGuard::create(&layout, INTENT, request()).expect("retain intent");
    assert!(matches!(
        DownloadJournalGuard::open(&layout, INTENT),
        Err(DownloadJournalError::Lock(JournalLockError::Locked { .. }))
    ));
    assert_eq!(
        fs::metadata(guard.path())
            .expect("private journal")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(matches!(
        retain_verified_fixture(&mut guard, CANISTER, "wrong-snapshot"),
        Err(DownloadJournalError::Record(
            DownloadJournalRecordError::SnapshotMismatch
        ))
    ));
    retain_verified_fixture(&mut guard, "AAAAA-AA", SNAPSHOT).expect("retained complete download");
    let checksum = guard.record().expect("verified record").artifacts()[0]
        .checksum()
        .expect("checksum")
        .clone();
    guard
        .finalize_artifact(CANISTER, SNAPSHOT)
        .expect("durable publication");
    assert!(!root.join("artifacts/aaaaa-aa.tmp").exists());
    assert_eq!(
        fs::read(root.join("artifacts/aaaaa-aa/nested/heap.bin")).expect("bytes"),
        b"opaque snapshot bytes"
    );
    assert_eq!(
        guard.record().expect("durable").artifacts()[0].checksum(),
        Some(&checksum)
    );
    drop(guard);
    fs::remove_dir_all(root.join("artifacts")).expect("simulate later unavailable artifacts");
    let guard = DownloadJournalGuard::open(&layout, &INTENT.to_uppercase())
        .expect("local retained progress only");
    assert_eq!(
        guard.record().unwrap().artifacts()[0].state(),
        ArtifactStateRecord::Durable
    );
    assert!(matches!(
        DownloadJournalGuard::create(&layout, INTENT, request()),
        Err(DownloadJournalError::Lock(_))
    ));
    drop(guard);
    assert!(
        matches!(DownloadJournalGuard::create(&layout,INTENT,request()),Err(DownloadJournalError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        DownloadJournalGuard::open(&layout, &"cd".repeat(32)),
        Err(DownloadJournalError::IntentMismatch)
    ));
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn changed_bytes_reject_publication_without_resetting_evidence() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let mut guard = DownloadJournalGuard::create(&layout, INTENT, request()).expect("journal");
    retain_verified_fixture(&mut guard, CANISTER, SNAPSHOT).expect("download");
    let retained = fs::read(guard.path()).expect("verified evidence");
    fs::write(
        root.join("artifacts/aaaaa-aa.tmp/nested/heap.bin"),
        b"changed",
    )
    .expect("corruption");
    assert!(matches!(
        guard.finalize_artifact(CANISTER, SNAPSHOT),
        Err(DownloadJournalError::Persistence(
            PersistenceError::Checksum(ChecksumError::ChecksumMismatch { .. })
        ))
    ));
    assert!(matches!(
        guard.record(),
        Err(DownloadJournalError::IndeterminateWrite)
    ));
    assert_eq!(fs::read(guard.path()).expect("retained"), retained);
    assert!(root.join("artifacts/aaaaa-aa.tmp").is_dir());
    assert!(!root.join("artifacts/aaaaa-aa").exists());
    drop(guard);
    let guard =
        DownloadJournalGuard::open(&layout, INTENT).expect("reconcile from verified evidence");
    assert_eq!(
        guard.record().expect("state").artifacts()[0].state(),
        ArtifactStateRecord::ChecksumVerified
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn failed_write_keeps_previous_bytes_and_requires_retained_recovery() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let mut guard = DownloadJournalGuard::create(&layout, INTENT, request()).expect("journal");
    let original = fs::read(guard.path()).expect("retained identity");
    let next = guard
        .next(
            CANISTER,
            SNAPSHOT,
            ArtifactTransition::ChecksumVerified(ArtifactChecksumRecord::from_bytes(
                b"retained verified bytes",
            )),
        )
        .expect("transition");
    assert!(matches!(
        guard.store(next, |_, _| Err(
            io::Error::other("before publication").into()
        )),
        Err(DownloadJournalError::Persistence(PersistenceError::Io(_)))
    ));
    assert_eq!(fs::read(guard.path()).expect("previous bytes"), original);
    assert!(matches!(
        guard.record(),
        Err(DownloadJournalError::IndeterminateWrite)
    ));
    drop(guard);
    let mut guard = DownloadJournalGuard::open(&layout, INTENT).expect("previous retained state");
    assert_eq!(
        guard.record().expect("created").artifacts()[0].state(),
        ArtifactStateRecord::Created
    );
    retain_verified_fixture(&mut guard, CANISTER, SNAPSHOT)
        .expect("seed retained fixture after recovery");
    drop(guard);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn post_write_error_requires_reopen_and_adopts_exact_retained_state() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let mut guard = DownloadJournalGuard::create(&layout, INTENT, request()).expect("journal");
    let next = guard
        .next(
            CANISTER,
            SNAPSHOT,
            ArtifactTransition::ChecksumVerified(ArtifactChecksumRecord::from_bytes(
                b"retained verified bytes",
            )),
        )
        .expect("transition");
    let error = guard.store(next, |path, record| {
        write_encoded(path, record)?;
        Err(io::Error::other("lost local response").into())
    });
    assert!(matches!(
        error,
        Err(DownloadJournalError::Persistence(PersistenceError::Io(_)))
    ));
    assert!(matches!(
        guard.finalize_artifact(CANISTER, SNAPSHOT),
        Err(DownloadJournalError::IndeterminateWrite)
    ));
    drop(guard);
    let guard = DownloadJournalGuard::open(&layout, INTENT).expect("adopt retained state");
    assert_eq!(
        guard.record().expect("downloaded").artifacts()[0].state(),
        ArtifactStateRecord::ChecksumVerified
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn unsafe_paths_and_corrupt_journals_reject_without_overwriting() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let mut guard = DownloadJournalGuard::create(&layout, INTENT, request()).expect("journal");
    retain_verified_fixture(&mut guard, CANISTER, SNAPSHOT).unwrap();
    let original = fs::read(guard.path()).expect("original");
    fs::rename(root.join("artifacts"), root.join("retained-artifacts")).expect("retain original");
    symlink(root.join("retained-artifacts"), root.join("artifacts")).expect("unsafe parent");
    assert!(matches!(
        guard.finalize_artifact(CANISTER, SNAPSHOT),
        Err(DownloadJournalError::UnsafeArtifactParent { .. })
    ));
    assert_eq!(fs::read(guard.path()).expect("unchanged"), original);
    fs::remove_file(root.join("artifacts")).expect("remove fixture link");
    fs::rename(root.join("retained-artifacts"), root.join("artifacts")).expect("restore fixture");
    let stage = root.join("artifacts/aaaaa-aa.tmp");
    fs::rename(&stage, root.join("retained-staging")).expect("retain stage");
    symlink(root.join("retained-staging"), &stage).expect("unsafe leaf");
    assert!(matches!(
        guard.finalize_artifact(CANISTER, SNAPSHOT),
        Err(DownloadJournalError::Persistence(PersistenceError::Io(_)))
    ));
    assert_eq!(fs::read(guard.path()).unwrap(), original);
    let path = guard.path();
    drop(guard);
    fs::write(&path, b"corrupt retained journal").expect("malformed fixture");
    assert!(matches!(
        DownloadJournalGuard::open(&layout, INTENT),
        Err(DownloadJournalError::Persistence(PersistenceError::Json(_)))
    ));
    assert_eq!(
        fs::read(&path).expect("preserved"),
        b"corrupt retained journal"
    );
    fs::remove_file(&path).expect("move fixture journal");
    symlink(root.join("retained-staging/nested/heap.bin"), &path).expect("unsafe journal");
    assert!(DownloadJournalGuard::open(&layout, INTENT).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .expect("retained link")
            .file_type()
            .is_symlink()
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn byte_limits_and_replaced_layouts_reject_before_progress() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let mut guard = DownloadJournalGuard::create(&layout, INTENT, request()).expect("journal");
    let path = guard.path();
    fs::rename(&root, root.with_extension("retained")).expect("retain original root");
    fs::create_dir(&root).expect("replacement root");
    assert!(matches!(
        retain_verified_fixture(&mut guard, CANISTER, SNAPSHOT),
        Err(DownloadJournalError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert!(!path.exists());
    drop(guard);
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).expect("replacement layout");
    fs::write(
        &path,
        vec![b' '; usize::try_from(MAX_DOWNLOAD_JOURNAL_BYTES).expect("bound") + 1],
    )
    .expect("excessive retained bytes");
    assert!(matches!(
        DownloadJournalGuard::open(&layout, INTENT),
        Err(DownloadJournalError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_DOWNLOAD_JOURNAL_BYTES
            }
        ))
    ));
    drop(layout);
    fs::remove_dir_all(&root).expect("clean successful replacement");
    fs::remove_dir_all(root.with_extension("retained")).expect("clean successful original");
}

#[test]
fn owner_death_after_directory_publication_reconciles_without_download_retry() {
    const CHILD_ROOT: &str = "IC_BACKUP_DOWNLOAD_PUBLICATION_CHILD";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = PathBuf::from(root);
        let layout = BackupLayoutGuard::acquire(&root).expect("child layout");
        let mut guard = DownloadJournalGuard::open(&layout, INTENT).expect("child journal");
        guard
            .finalize_with(CANISTER, SNAPSHOT, |_, _| {
                hold_at_acknowledged_barrier(&root)
            })
            .expect("parent kills before journal advancement");
        unreachable!("crash barrier returned");
    }
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let mut guard = DownloadJournalGuard::create(&layout, INTENT, request()).expect("journal");
    retain_verified_fixture(&mut guard, CANISTER, SNAPSHOT).expect("download");
    let verified = fs::read(guard.path()).expect("original retained bytes");
    drop(guard);
    drop(layout);
    let mut child=Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact","ops::persistence::download_journal::tests::owner_death_after_directory_publication_reconciles_without_download_retry","--nocapture"])
        .env(CHILD_ROOT,&root).spawn().expect("spawn acknowledged owner");
    kill_child_at_acknowledged_barrier(&mut child, &root);
    assert!(!root.join("artifacts/aaaaa-aa.tmp").exists());
    assert!(root.join("artifacts/aaaaa-aa/nested/heap.bin").is_file());
    let layout = BackupLayoutGuard::acquire(&root).expect("recover released layout");
    let mut guard = DownloadJournalGuard::open(&layout, INTENT).expect("recover retained intent");
    assert_eq!(fs::read(guard.path()).expect("verified evidence"), verified);
    assert_eq!(
        guard.record().unwrap().artifacts()[0].state(),
        ArtifactStateRecord::ChecksumVerified
    );
    guard
        .finalize_artifact(CANISTER, SNAPSHOT)
        .expect("adopt exact published tree");
    assert_eq!(
        guard.record().expect("durable evidence").artifacts()[0].state(),
        crate::model::download_journal::ArtifactStateRecord::Durable
    );
    assert_eq!(
        fs::read(root.join("artifacts/aaaaa-aa/nested/heap.bin")).expect("exact recovered bytes"),
        b"opaque snapshot bytes"
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful acknowledged fixture");
}

#[test]
fn guard_preserves_send_sync_contract() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<DownloadJournalGuard<'_>>();
}

#[test]
fn retired_collection_and_downloaded_evidence_refuse_without_rewriting() {
    let root = fixture();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let guard = DownloadJournalGuard::create(&layout, INTENT, request()).unwrap();
    let path = guard.path();
    let original = serde_json::to_value(guard.record().unwrap()).unwrap();
    drop(guard);
    for collection in [false, true] {
        let mut retired = original.clone();
        if collection {
            let entry = retired["artifacts"][0].clone();
            retired["artifacts"].as_array_mut().unwrap().push(entry);
        } else {
            retired["artifacts"][0]["state"] = serde_json::json!("Downloaded");
        }
        let bytes = serde_json::to_vec(&retired).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            DownloadJournalGuard::open(&layout, INTENT),
            Err(DownloadJournalError::Persistence(PersistenceError::Json(_)))
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(root.join("artifacts/aaaaa-aa.tmp/nested/heap.bin").exists());
    }
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
