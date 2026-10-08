//! Native durable reservations and acknowledged process death; no IC effects.

use super::*;
use crate::{
    model::attempt_journal::{
        AttemptBudgetRecord, ObservationOutcomeRecord, OperationBindingRecord,
        OperationBindingRequest,
    },
    test_support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir},
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
    process::Command,
};

fn authority() -> AttemptAuthorityRecord {
    AttemptAuthorityRecord::new(
        OperationBindingRecord::new(&OperationBindingRequest {
            intent: "ab".repeat(32),
            operation_sequence: u64::MAX,
            network: "cd".repeat(32),
            caller: "2vxsx-fae".into(),
            target: "aaaaa-aa".into(),
            release: "ef".repeat(32),
            request: "01".repeat(32),
        })
        .expect("binding"),
        AttemptBudgetRecord::new(1, 1).expect("finite allowance"),
    )
}
fn root() -> PathBuf {
    let root = temp_dir("ic-backup-attempt-journal");
    fs::create_dir(&root).expect("native layout");
    root
}

#[test]
fn retained_limits_exact_identity_and_receipts_survive_reopen() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let selected = authority();
    let mut guard = AttemptJournalGuard::create(&layout, selected.clone()).expect("journal");
    assert_eq!(
        fs::metadata(guard.path())
            .expect("private file")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(matches!(
        AttemptJournalGuard::open(&layout, &selected),
        Err(AttemptJournalError::Lock(JournalLockError::Locked { .. }))
    ));
    assert_eq!(guard.reserve_mutation().expect("durable claim"), 1);
    let path = guard.path();
    drop(guard);
    let original = fs::read(&path).expect("retained bytes");
    let larger = AttemptAuthorityRecord::new(
        selected.binding().clone(),
        AttemptBudgetRecord::new(2, 1).expect("larger"),
    );
    assert!(matches!(
        AttemptJournalGuard::open(&layout, &larger),
        Err(AttemptJournalError::AuthorityMismatch)
    ));
    assert_eq!(fs::read(&path).expect("unchanged"), original);
    let mut guard = AttemptJournalGuard::open(&layout, &selected).expect("original limits");
    assert!(matches!(
        guard.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { attempt: 1 }
        ))
    ));
    assert_eq!(
        guard
            .reserve_observation(1, &"45".repeat(32))
            .expect("separate observation budget"),
        2
    );
    guard
        .record_observation(ObservationReceiptRequest {
            attempt: 2,
            request: "45".repeat(32),
            outcome: ObservationOutcomeRecord::Applied,
            evidence: "67".repeat(32),
        })
        .expect("resolve without mutation");
    assert_eq!(guard.record().expect("evidence").view().mutations_used, 1);
    drop(guard);
    let guard =
        AttemptJournalGuard::open(&layout, &selected).expect("local terminal accounting replay");
    assert!(guard.record().expect("retained view").view().applied);
    drop(guard);
    assert!(
        matches!(AttemptJournalGuard::create(&layout,larger),Err(AttemptJournalError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn failed_reservation_never_returns_a_slot_and_forces_exact_recovery() {
    for published in [false, true] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).expect("layout");
        let selected = authority();
        let mut guard = AttemptJournalGuard::create(&layout, selected.clone()).expect("journal");
        let path = guard.path();
        let original = fs::read(&path).expect("initial bytes");
        let result = guard.reserve_with(AttemptJournalRecord::reserve_mutation, |path, next| {
            if published {
                write_json_durable(path, next)?;
            }
            Err(io::Error::other("lost local reservation response").into())
        });
        assert!(matches!(
            result,
            Err(AttemptJournalError::Persistence(PersistenceError::Io(_)))
        ));
        assert!(matches!(
            guard.reserve_mutation(),
            Err(AttemptJournalError::IndeterminateWrite)
        ));
        assert!(matches!(
            guard.record(),
            Err(AttemptJournalError::IndeterminateWrite)
        ));
        drop(guard);
        let mut recovered =
            AttemptJournalGuard::open(&layout, &selected).expect("recover exact evidence");
        if published {
            assert_eq!(
                recovered.record().expect("pending").view().pending_mutation,
                Some(1)
            );
            assert!(matches!(
                recovered.reserve_mutation(),
                Err(AttemptJournalError::Record(
                    AttemptJournalRecordError::MutationPending { attempt: 1 }
                ))
            ));
        } else {
            assert_eq!(fs::read(&path).expect("unchanged bytes"), original);
            assert_eq!(
                recovered
                    .record()
                    .expect("no consumed slot")
                    .view()
                    .mutations_used,
                0
            );
            assert_eq!(
                recovered
                    .reserve_mutation()
                    .expect("previous call never returned a slot"),
                1
            );
        }
        drop(recovered);
        drop(layout);
        fs::remove_dir_all(root).expect("clean successful fixture");
    }
}

#[test]
fn unsafe_corrupt_excessive_and_replaced_evidence_rejects() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).expect("layout");
    let selected = authority();
    let guard = AttemptJournalGuard::create(&layout, selected.clone()).expect("journal");
    let path = guard.path();
    drop(guard);
    fs::write(&path, b"malformed retained evidence").expect("corrupt fixture");
    assert!(matches!(
        AttemptJournalGuard::open(&layout, &selected),
        Err(AttemptJournalError::Persistence(PersistenceError::Json(_)))
    ));
    assert_eq!(
        fs::read(&path).expect("preserved"),
        b"malformed retained evidence"
    );
    fs::write(
        &path,
        vec![b' '; usize::try_from(MAX_ATTEMPT_JOURNAL_BYTES).expect("bound") + 1],
    )
    .expect("excessive fixture");
    assert!(matches!(
        AttemptJournalGuard::open(&layout, &selected),
        Err(AttemptJournalError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_ATTEMPT_JOURNAL_BYTES
            }
        ))
    ));
    fs::rename(&path, root.join("retained.json")).expect("retain fixture");
    symlink(root.join("retained.json"), &path).expect("unsafe journal");
    assert!(AttemptJournalGuard::open(&layout, &selected).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .expect("retained link")
            .file_type()
            .is_symlink()
    );
    fs::rename(&root, root.with_extension("retained")).expect("retain root");
    fs::create_dir(&root).expect("replacement root");
    assert!(matches!(
        AttemptJournalGuard::create(&layout, selected),
        Err(AttemptJournalError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert!(!path.exists());
    drop(layout);
    fs::remove_dir_all(&root).expect("clean replacement");
    fs::remove_dir_all(root.with_extension("retained")).expect("clean original successful fixture");
}

#[test]
fn acknowledged_owner_death_selects_exact_reservation_recovery() {
    const CHILD_ROOT: &str = "IC_BACKUP_ATTEMPT_CHILD_ROOT";
    const CHILD_SIDE: &str = "IC_BACKUP_ATTEMPT_CHILD_SIDE";
    const SIDES: &[&str] = &["before_write", "after_write", "after_return"];
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = PathBuf::from(root);
        let side = std::env::var(CHILD_SIDE).expect("registered side");
        assert!(SIDES.contains(&side.as_str()));
        let layout = BackupLayoutGuard::acquire(&root).expect("child layout");
        let mut guard = AttemptJournalGuard::open(&layout, &authority()).expect("child journal");
        guard
            .reserve_with(AttemptJournalRecord::reserve_mutation, |path, next| {
                if side == "before_write" {
                    hold_at_acknowledged_barrier(&root);
                }
                write_json_durable(path, next)?;
                if side == "after_write" {
                    hold_at_acknowledged_barrier(&root);
                }
                Ok(())
            })
            .expect("durably returned slot");
        hold_at_acknowledged_barrier(&root);
    }
    for side in SIDES {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).expect("layout");
        let selected = authority();
        let guard = AttemptJournalGuard::create(&layout, selected.clone()).expect("journal");
        drop(guard);
        drop(layout);
        let mut child=Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact","ops::persistence::attempt_journal::tests::acknowledged_owner_death_selects_exact_reservation_recovery","--nocapture"])
            .env(CHILD_ROOT,&root).env(CHILD_SIDE,side).spawn().expect("spawn native reservation owner");
        kill_child_at_acknowledged_barrier(&mut child, &root);
        let layout = BackupLayoutGuard::acquire(&root).expect("recovered exclusion");
        let mut guard =
            AttemptJournalGuard::open(&layout, &selected).expect("exact retained history");
        if *side == "before_write" {
            assert_eq!(
                guard
                    .record()
                    .expect("initial accounting")
                    .view()
                    .mutations_used,
                0
            );
            assert_eq!(
                guard
                    .reserve_mutation()
                    .expect("no durable consumption or returned slot"),
                1
            );
        } else {
            assert_eq!(
                guard
                    .record()
                    .expect("pending accounting")
                    .view()
                    .mutations_used,
                1
            );
            assert_eq!(
                guard
                    .record()
                    .expect("pending accounting")
                    .view()
                    .pending_mutation,
                Some(1)
            );
            assert!(matches!(
                guard.reserve_mutation(),
                Err(AttemptJournalError::Record(
                    AttemptJournalRecordError::MutationPending { attempt: 1 }
                ))
            ));
        }
        drop(guard);
        drop(layout);
        fs::remove_dir_all(root).expect("clean successful acknowledged fixture");
    }
}
