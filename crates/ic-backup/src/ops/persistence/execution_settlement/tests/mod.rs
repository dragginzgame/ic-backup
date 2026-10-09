use super::*;
use crate::ops::persistence::AttemptJournalGuard;
use crate::{
    model::attempt_journal::{
        MutationOutcomeRecord, MutationReceiptRequest, ObservationOutcomeRecord,
        ObservationReceiptRequest,
    },
    test_support::{
        execution_settlement::{applied, record},
        hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier,
        membership::plan,
        temp_dir,
    },
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    process::Command,
};

fn prepare() -> (PathBuf, BackupLayoutGuard, ExecutionSettlementRecord) {
    let root = temp_dir("ic-backup-execution-settlement");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    super::super::create_operation_plan(&layout, &plan).unwrap();
    for operation in plan.operations() {
        let mut guard = AttemptJournalGuard::create(
            &layout,
            plan.attempt_authority(operation.operation_sequence())
                .unwrap(),
        )
        .unwrap();
        let attempt = guard.reserve_mutation().unwrap();
        guard
            .record_mutation(MutationReceiptRequest {
                attempt,
                request: operation.request().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: "12".repeat(32),
            })
            .unwrap();
    }
    let record = record(&plan, &applied(&plan));
    (root, layout, record)
}
#[test]
fn derived_checkpoint_preserves_original_histories_and_replays_without_replacement() {
    let (root, layout, expected) = prepare();
    let before: Vec<_> = [0, 7]
        .map(|sequence| fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap())
        .into();
    let actual = checkpoint_execution_settlement(&layout, expected.plan_intent()).unwrap();
    assert_eq!(actual, expected);
    let path = root.join("execution-settlement.json");
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        read_execution_settlement(&layout, actual.plan_intent(), &actual.digest()).unwrap(),
        actual
    );
    assert!(checkpoint_execution_settlement(&layout, expected.plan_intent()).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    for (sequence, original) in [0, 7].into_iter().zip(before) {
        assert_eq!(
            fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap(),
            original
        );
    }
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn derivation_requires_complete_unheld_originals_and_exact_plan_identity() {
    let (root, layout, expected) = prepare();
    let plan = plan();
    let held = AttemptJournalGuard::open(&layout, &plan.attempt_authority(7).unwrap()).unwrap();
    assert!(matches!(
        checkpoint_execution_settlement(&layout, expected.plan_intent()),
        Err(ExecutionSettlementCheckpointError::Persistence(
            ExecutionSettlementPersistenceError::Journal(AttemptJournalError::Lock(_))
        ))
    ));
    drop(held);
    fs::rename(root.join("attempt-7.json"), root.join("retained-journal")).unwrap();
    assert!(matches!(
        checkpoint_execution_settlement(&layout, expected.plan_intent()),
        Err(ExecutionSettlementCheckpointError::Persistence(
            ExecutionSettlementPersistenceError::Journal(_)
        ))
    ));
    assert!(matches!(
        checkpoint_execution_settlement(
            &layout,
            &ArtifactChecksumRecord::from_bytes(b"other plan")
        ),
        Err(ExecutionSettlementCheckpointError::Persistence(
            ExecutionSettlementPersistenceError::Plan(_)
        ))
    ));
    assert!(!root.join("execution-settlement.json").exists());
    assert!(root.join("retained-journal").exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn pending_negative_and_uncertain_histories_cannot_be_checkpointed_or_reset() {
    for state in ["pending", "not-applied", "uncertain"] {
        let (root, layout, expected) = prepare();
        fs::rename(
            root.join("attempt-7.json"),
            root.join("retained-applied-journal"),
        )
        .unwrap();
        let authority = plan().attempt_authority(7).unwrap();
        let mut journal = AttemptJournalGuard::create(&layout, authority).unwrap();
        let attempt = journal.reserve_mutation().unwrap();
        if state == "not-applied" {
            journal
                .record_mutation(MutationReceiptRequest {
                    attempt,
                    request: journal
                        .record()
                        .unwrap()
                        .authority()
                        .binding()
                        .request()
                        .into(),
                    outcome: MutationOutcomeRecord::NotApplied,
                    evidence: "34".repeat(32),
                })
                .unwrap();
        } else if state == "uncertain" {
            let request = "56".repeat(32);
            let observation = journal.reserve_observation(attempt, &request).unwrap();
            journal
                .record_observation(ObservationReceiptRequest {
                    attempt: observation,
                    request,
                    outcome: ObservationOutcomeRecord::Uncertain,
                    evidence: "78".repeat(32),
                })
                .unwrap();
        }
        drop(journal);
        let original = fs::read(root.join("attempt-7.json")).unwrap();
        assert!(
            matches!(
                checkpoint_execution_settlement(&layout, expected.plan_intent()),
                Err(ExecutionSettlementCheckpointError::Persistence(
                    ExecutionSettlementPersistenceError::Policy(
                        ExecutionSettlementPolicyError::UnsettledOperation(7)
                    )
                ))
            ),
            "{state}"
        );
        assert!(!root.join("execution-settlement.json").exists());
        assert_eq!(fs::read(root.join("attempt-7.json")).unwrap(), original);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn immutable_private_publication_reopens_exact_originals_and_rejects_contention() {
    let (root, layout, record) = prepare();
    let plan = plan();
    let held = AttemptJournalGuard::open(&layout, &plan.attempt_authority(0).unwrap()).unwrap();
    assert!(matches!(
        create_execution_settlement(&layout, &record),
        Err(ExecutionSettlementPersistenceError::Journal(
            AttemptJournalError::Lock(_)
        ))
    ));
    assert!(!root.join("execution-settlement.json").exists());
    drop(held);
    create_execution_settlement(&layout, &record).unwrap();
    let path = root.join("execution-settlement.json");
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        read_execution_settlement(&layout, record.plan_intent(), &record.digest()).unwrap(),
        record
    );
    assert!(
        matches!(create_execution_settlement(&layout,&record),Err(ExecutionSettlementPersistenceError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(&path).unwrap(), bytes);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn lost_publication_response_before_or_after_write_reconciles_exact_local_evidence() {
    for published in [false, true] {
        let (root, layout, record) = prepare();
        let original = fs::read(root.join("attempt-0.json")).unwrap();
        let error = create_with(&layout, &record, |path, record| {
            if published {
                create_json_durable(path, record)?;
            }
            Err(io::Error::other("lost local publication response").into())
        })
        .unwrap_err();
        assert!(matches!(
            error,
            ExecutionSettlementPersistenceError::Persistence(_)
        ));
        assert_eq!(fs::read(root.join("attempt-0.json")).unwrap(), original);
        if published {
            assert_eq!(
                read_execution_settlement(&layout, record.plan_intent(), &record.digest()).unwrap(),
                record
            );
        } else {
            assert!(!root.join("execution-settlement.json").exists());
            create_execution_settlement(&layout, &record).unwrap();
        }
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn unsafe_excessive_missing_and_changed_originals_reject_without_repair() {
    let (root, layout, record) = prepare();
    create_execution_settlement(&layout, &record).unwrap();
    let path = root.join("execution-settlement.json");
    let bytes = fs::read(&path).unwrap();
    let journal = root.join("attempt-0.json");
    let original = fs::read(&journal).unwrap();
    let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
    changed["events"][1]["evidence"] = serde_json::json!("34".repeat(32));
    fs::write(&journal, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
    assert!(matches!(
        read_execution_settlement(&layout, record.plan_intent(), &record.digest()),
        Err(ExecutionSettlementPersistenceError::Policy(
            ExecutionSettlementPolicyError::HistoryMismatch(0)
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::write(&journal, &original).unwrap();
    fs::rename(&journal, root.join("retained-journal")).unwrap();
    assert!(matches!(
        read_execution_settlement(&layout, record.plan_intent(), &record.digest()),
        Err(ExecutionSettlementPersistenceError::Journal(_))
    ));
    symlink(root.join("retained-journal"), &journal).unwrap();
    assert!(read_execution_settlement(&layout, record.plan_intent(), &record.digest()).is_err());
    fs::remove_file(&journal).unwrap();
    fs::rename(root.join("retained-journal"), &journal).unwrap();
    fs::write(
        &path,
        vec![b' '; usize::try_from(MAX_EXECUTION_SETTLEMENT_BYTES).unwrap() + 1],
    )
    .unwrap();
    assert!(matches!(
        read_execution_settlement(&layout, record.plan_intent(), &record.digest()),
        Err(ExecutionSettlementPersistenceError::Persistence(
            PersistenceError::RecordTooLarge { .. }
        ))
    ));
    fs::write(&path, &bytes).unwrap();
    fs::rename(&path, root.join("retained-settlement")).unwrap();
    symlink(root.join("retained-settlement"), &path).unwrap();
    assert!(read_execution_settlement(&layout, record.plan_intent(), &record.digest()).is_err());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn held_layout_replacement_preserves_retained_checkpoint() {
    let (root, layout, record) = prepare();
    create_execution_settlement(&layout, &record).unwrap();
    let moved = root.with_extension("retained");
    fs::rename(&root, &moved).unwrap();
    fs::create_dir(&root).unwrap();
    let bytes = fs::read(moved.join("execution-settlement.json")).unwrap();
    assert!(matches!(
        read_execution_settlement(&layout, record.plan_intent(), &record.digest()),
        Err(ExecutionSettlementPersistenceError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert_eq!(
        fs::read(moved.join("execution-settlement.json")).unwrap(),
        bytes
    );
    assert!(!root.join("execution-settlement.json").exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(moved).unwrap();
}

#[test]
fn acknowledged_process_death_before_publication_or_after_sync_replays_locally() {
    const ROOT: &str = "IC_BACKUP_SETTLEMENT_CHILD_ROOT";
    const BARRIER: &str = "IC_BACKUP_SETTLEMENT_CHILD_PUBLISHED";
    const HANDSHAKE: &str = "IC_BACKUP_SETTLEMENT_CHILD_HANDSHAKE";
    if let Some(root) = std::env::var_os(ROOT) {
        let root = PathBuf::from(root);
        let handshake = PathBuf::from(std::env::var_os(HANDSHAKE).unwrap());
        let published = std::env::var(BARRIER).unwrap() == "yes";
        let plan = plan();
        let record = record(&plan, &applied(&plan));
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        create_with(&layout, &record, |path, record| {
            super::super::json::create_json_durable_at_barriers(
                path,
                record,
                || {
                    if !published {
                        hold_at_acknowledged_barrier(&handshake);
                    }
                },
                || {
                    if published {
                        hold_at_acknowledged_barrier(&handshake);
                    }
                },
            )
        })
        .unwrap();
        panic!("crash child passed its acknowledged publication barrier");
    }
    for published in [false, true] {
        let (root, layout, record) = prepare();
        let original = fs::read(root.join("attempt-0.json")).unwrap();
        drop(layout);
        let handshake = temp_dir("ic-backup-settlement-handshake");
        fs::create_dir(&handshake).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ops::persistence::execution_settlement::tests::acknowledged_process_death_before_publication_or_after_sync_replays_locally", "--nocapture"])
            .env(ROOT, &root).env(BARRIER, if published {"yes"} else {"no"}).env(HANDSHAKE, &handshake)
            .spawn().unwrap();
        kill_child_at_acknowledged_barrier(&mut child, &handshake);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        assert_eq!(root.join("execution-settlement.json").exists(), published);
        assert_eq!(fs::read(root.join("attempt-0.json")).unwrap(), original);
        if !published {
            create_execution_settlement(&layout, &record).unwrap();
        }
        assert_eq!(
            read_execution_settlement(&layout, record.plan_intent(), &record.digest()).unwrap(),
            record
        );
        assert_eq!(fs::read(root.join("attempt-0.json")).unwrap(), original);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(handshake).unwrap();
    }
}
