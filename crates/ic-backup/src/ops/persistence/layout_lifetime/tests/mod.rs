//! Fresh layout exclusion and retention regressions adapted from Canic.

use super::*;
use crate::{
    model::restore_references::RestoreReferenceError,
    test_support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir},
};
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    process::Command,
};

#[test]
fn removed_and_recreated_layout_keeps_the_same_lock_identity() {
    let parent = temp_dir("ic-backup-layout-recreate");
    let root = parent.join("backup");
    fs::create_dir_all(&root).expect("create backup");
    let guard = BackupLayoutGuard::acquire(&root).expect("own layout");
    fs::remove_dir_all(&root).expect("remove backup");
    fs::create_dir(&root).expect("recreate backup");
    assert!(matches!(
        BackupLayoutGuard::acquire(&root),
        Err(JournalLockError::Locked { .. })
    ));
    assert!(
        matches!(guard.restore_references(), Err(PersistenceError::LayoutChanged { path }) if path == root)
    );
    assert!(matches!(
        guard.retain_restore(&parent.join("journal.json"), &"ab".repeat(32)),
        Err(PersistenceError::LayoutChanged { .. })
    ));
    assert!(!root.join(REFERENCES_FILE).exists());
    drop(guard);
    let guard = BackupLayoutGuard::acquire(&root).expect("same sidecar released");
    assert_eq!(guard.root(), root);
    drop(guard);
    let sidecars: Vec<_> = fs::read_dir(&parent)
        .expect("read parent")
        .map(|entry| entry.expect("entry").file_name())
        .filter(|name| name.to_string_lossy().starts_with(".ic-backup-layout-"))
        .collect();
    assert_eq!(sidecars.len(), 1);
    fs::remove_dir_all(parent).expect("clean successful fixture");
}

#[test]
fn exact_authority_survives_reopen_and_missing_external_journals() {
    let parent = temp_dir("ic-backup-layout-authority");
    let root = parent.join("backup");
    fs::create_dir_all(&root).expect("create backup");
    let guard = BackupLayoutGuard::acquire(&root).expect("lock layout");
    assert!(!guard.has_restore_references().expect("empty dependencies"));
    let journal = parent.join("external.json");
    let hash = "ab".repeat(32);
    let reference = guard
        .retain_restore(&journal, &hash.to_uppercase())
        .expect("retain exact restore");
    assert_eq!(reference.journal(), journal);
    assert_eq!(reference.authority(), hash);
    let bytes = fs::read(root.join(REFERENCES_FILE)).expect("reference bytes");
    assert_eq!(
        guard
            .retain_restore(&journal, &hash)
            .expect("adopt exact reference"),
        reference
    );
    assert_eq!(
        fs::read(root.join(REFERENCES_FILE)).expect("unchanged reference bytes"),
        bytes
    );
    assert!(
        matches!(guard.retain_restore(&journal, &"cd".repeat(32)), Err(PersistenceError::RestoreReference(RestoreReferenceError::AuthorityConflict { journal: actual })) if actual == journal)
    );
    assert_eq!(
        fs::read(root.join(REFERENCES_FILE)).expect("preserved conflicting intent"),
        bytes
    );
    assert_eq!(
        fs::metadata(root.join(REFERENCES_FILE))
            .expect("private reference metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    drop(guard);
    let guard = BackupLayoutGuard::acquire(&root).expect("reopen backup");
    assert!(!journal.exists());
    assert!(
        guard
            .has_restore_references()
            .expect("missing journal stays retained")
    );
    assert_eq!(
        guard
            .restore_references()
            .expect("read exact references")
            .entries(),
        &[reference]
    );
    drop(guard);
    fs::remove_dir_all(parent).expect("clean successful fixture");
}

#[test]
fn root_and_journal_parent_aliases_resolve_to_one_identity() {
    let parent = temp_dir("ic-backup-layout-alias");
    let root = parent.join("backup");
    fs::create_dir_all(&root).expect("create root");
    let alias = parent.join("alias");
    symlink(&root, &alias).expect("explicit root alias");
    let guard = BackupLayoutGuard::acquire(&alias).expect("resolve operator root");
    assert_eq!(guard.root(), root);
    assert!(matches!(
        BackupLayoutGuard::acquire(&root),
        Err(JournalLockError::Locked { .. })
    ));
    let first = guard
        .retain_restore(&alias.join("journal.json"), &"ab".repeat(32))
        .expect("resolve journal parent");
    let second = guard
        .retain_restore(&root.join("journal.json"), &"ab".repeat(32))
        .expect("same resolved journal");
    assert_eq!(first, second);
    assert_eq!(
        guard.restore_references().expect("one identity").entries(),
        &[first]
    );
    let linked_journal = parent.join("linked-journal.json");
    symlink(root.join("journal.json"), &linked_journal).expect("dangling journal leaf alias");
    assert!(
        matches!(guard.retain_restore(&linked_journal, &"ab".repeat(32)), Err(PersistenceError::InvalidRestoreReferences { path }) if path == linked_journal)
    );
    drop(guard);
    fs::remove_dir_all(parent).expect("clean successful fixture");
}

#[test]
fn unsafe_reference_entries_fail_closed_without_following_links() {
    let parent = temp_dir("ic-backup-layout-unsafe-reference");
    let root = parent.join("backup");
    fs::create_dir_all(&root).expect("create root");
    let guard = BackupLayoutGuard::acquire(&root).expect("lock backup");
    let outside = parent.join("outside.json");
    let bytes = br#"{"version":1,"restores":[]}"#;
    fs::write(&outside, bytes).expect("outside document");
    let path = root.join(REFERENCES_FILE);
    symlink(&outside, &path).expect("unsafe record link");
    assert!(
        matches!(guard.has_restore_references(), Err(PersistenceError::InvalidRestoreReferences { path: actual }) if actual == path)
    );
    assert!(
        guard
            .retain_restore(&parent.join("journal.json"), &"ab".repeat(32))
            .is_err()
    );
    assert_eq!(fs::read(&outside).expect("retained outside bytes"), bytes);
    fs::remove_file(&path).expect("remove successful test link");
    fs::create_dir(&path).expect("unsafe record directory");
    assert!(matches!(
        guard.has_restore_references(),
        Err(PersistenceError::InvalidRestoreReferences { .. })
    ));
    fs::remove_dir(&path).expect("remove test directory");
    rustix::fs::mknodat(
        rustix::fs::CWD,
        &path,
        rustix::fs::FileType::Fifo,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        0,
    )
    .expect("unsafe record FIFO");
    assert!(matches!(
        guard.has_restore_references(),
        Err(PersistenceError::InvalidRestoreReferences { .. })
    ));
    drop(guard);
    fs::remove_dir_all(parent).expect("clean successful fixture");
}

#[test]
fn missing_layouts_and_corrupt_or_excessive_records_cannot_authorize_empty_retention() {
    let parent = temp_dir("ic-backup-layout-invalid-reference");
    let root = parent.join("backup");
    fs::create_dir_all(&parent).expect("create parent");
    assert!(
        matches!(BackupLayoutGuard::acquire(&root), Err(JournalLockError::Io(error)) if error.kind() == io::ErrorKind::NotFound)
    );
    assert!(!root.exists());
    fs::create_dir(&root).expect("create root");
    let guard = BackupLayoutGuard::acquire(&root).expect("lock backup");
    let path = root.join(REFERENCES_FILE);
    for invalid in [
        b"malformed".as_slice(),
        br#"{"version":2,"restores":[]}"#,
        br#"{"version":1,"restores":[],"unknown":true}"#,
    ] {
        fs::write(&path, invalid).expect("inject rejected document");
        assert!(matches!(
            guard.has_restore_references(),
            Err(PersistenceError::Json(_))
        ));
        assert!(
            guard
                .retain_restore(&parent.join("journal.json"), &"ab".repeat(32))
                .is_err()
        );
        assert_eq!(fs::read(&path).expect("retain rejected record"), invalid);
    }
    let excessive =
        vec![b' '; usize::try_from(MAX_RESTORE_REFERENCE_BYTES).expect("native bound") + 1];
    fs::write(&path, &excessive).expect("oversized document");
    assert!(matches!(
        guard.restore_references(),
        Err(PersistenceError::RecordTooLarge {
            limit: MAX_RESTORE_REFERENCE_BYTES
        })
    ));
    assert_eq!(
        fs::read(&path).expect("retain oversized evidence"),
        excessive
    );
    drop(guard);
    fs::remove_dir_all(parent).expect("clean successful fixture");
}

#[test]
fn publication_byte_bound_preserves_existing_dependencies() {
    use crate::model::restore_references::{MAX_JOURNAL_PATH_BYTES, MAX_RESTORE_REFERENCES};
    let parent = temp_dir("ic-backup-layout-write-bound");
    let root = parent.join("backup");
    fs::create_dir_all(&root).expect("create root");
    let guard = BackupLayoutGuard::acquire(&root).expect("lock layout");
    let hash = "ab".repeat(32);
    let entries: Vec<_> = (0..MAX_RESTORE_REFERENCES - 1)
        .map(|index| serde_json::json!({"journal":format!("/external/{index:04}-{}", "x".repeat(700)),"authority":hash}))
        .collect();
    let mut value = serde_json::json!({"version":1,"restores":entries});
    let limit = usize::try_from(MAX_RESTORE_REFERENCE_BYTES).expect("native byte bound");
    let mut remaining = limit
        - serde_json::to_vec_pretty(&value)
            .expect("initial bytes")
            .len();
    for entry in value["restores"].as_array_mut().expect("entries") {
        let location = entry["journal"].as_str().expect("journal location");
        let added = remaining.min(MAX_JOURNAL_PATH_BYTES - location.len());
        entry["journal"] = serde_json::Value::String(format!("{location}{}", "x".repeat(added)));
        remaining -= added;
        if remaining == 0 {
            break;
        }
    }
    assert_eq!(remaining, 0);
    let retained: RestoreReferencesRecord =
        serde_json::from_value(value).expect("admitted dependencies");
    let bytes = serde_json::to_vec_pretty(&retained).expect("canonical record bytes");
    assert_eq!(bytes.len(), limit);
    let path = root.join(REFERENCES_FILE);
    fs::write(&path, &bytes).expect("exact-limit retained record");
    assert_eq!(
        guard.restore_references().expect("exact limit accepts"),
        retained
    );
    assert!(matches!(
        guard.retain_restore(&parent.join("new.json"), &hash),
        Err(PersistenceError::RecordTooLarge {
            limit: MAX_RESTORE_REFERENCE_BYTES
        })
    ));
    assert_eq!(fs::read(&path).expect("original bytes remain"), bytes);
    assert_eq!(
        guard
            .restore_references()
            .expect("original dependencies remain"),
        retained
    );
    drop(guard);
    fs::remove_dir_all(parent).expect("clean successful fixture");
}

#[test]
fn process_death_before_or_after_reference_publication_retains_exact_evidence() {
    use crate::ops::persistence::json::{DurableWriteBarrier, write_json_durable_at_barriers};
    const ROOT_ENV: &str = "IC_BACKUP_LAYOUT_CRASH_ROOT";
    const SIDE_ENV: &str = "IC_BACKUP_LAYOUT_CRASH_SIDE";
    let hash = "ab".repeat(32);
    if let Some(parent) = std::env::var_os(ROOT_ENV) {
        let parent = PathBuf::from(parent);
        let side = std::env::var(SIDE_ENV).expect("child barrier");
        let guard = BackupLayoutGuard::acquire(&parent.join("backup")).expect("child guard");
        guard
            .retain_with(&parent.join("external.json"), &hash, |path, references| {
                write_json_durable_at_barriers(path, references, |barrier| {
                    if matches!(
                        (side.as_str(), barrier),
                        ("before", DurableWriteBarrier::BeforeRename)
                            | ("after", DurableWriteBarrier::AfterDirectorySync)
                    ) {
                        hold_at_acknowledged_barrier(&parent);
                    }
                })
            })
            .expect("child retain");
        panic!("crash barrier did not run");
    }
    for side in ["before", "after"] {
        let parent = temp_dir(&format!("ic-backup-layout-crash-{side}"));
        let root = parent.join("backup");
        fs::create_dir_all(&root).expect("create root");
        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", "ops::persistence::layout_lifetime::tests::process_death_before_or_after_reference_publication_retains_exact_evidence", "--nocapture"])
            .env(ROOT_ENV, &parent).env(SIDE_ENV, side).spawn().expect("spawn owner");
        crate::test_support::wait_for_child_path(
            &mut child,
            &parent.join("barrier-ready"),
            "reference publication",
        );
        assert!(matches!(
            BackupLayoutGuard::acquire(&root),
            Err(JournalLockError::Locked { .. })
        ));
        kill_child_at_acknowledged_barrier(&mut child, &parent);
        let guard = BackupLayoutGuard::acquire(&root).expect("owner death releases lock");
        assert_eq!(
            guard
                .has_restore_references()
                .expect("read retained references"),
            side == "after"
        );
        assert!(!parent.join("external.json").exists());
        if side == "before" {
            assert!(!root.join(REFERENCES_FILE).exists());
            assert!(
                fs::read_dir(&root)
                    .expect("retained directory")
                    .any(|entry| entry
                        .expect("entry")
                        .file_name()
                        .to_string_lossy()
                        .contains(".ic-backup-tmp-"))
            );
        }
        let reference = guard
            .retain_restore(&parent.join("external.json"), &hash)
            .expect("continue or adopt exact intent");
        assert_eq!(
            guard
                .restore_references()
                .expect("one exact dependency")
                .entries(),
            &[reference]
        );
        drop(guard);
        fs::remove_dir_all(parent).expect("clean successful fixture");
    }
}
