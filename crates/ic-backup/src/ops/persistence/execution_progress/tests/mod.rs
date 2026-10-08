//! Real local original-evidence admission; fixtures authenticate no remote effects.

use super::*;
use crate::{
    model::attempt_journal::{
        AttemptJournalRecordError, MutationOutcomeRecord, MutationReceiptRequest,
    },
    ops::persistence::{
        AttemptJournalGuard, attempt_journal::read_original_journals, create_operation_plan,
    },
    policy::execution_progress::OperationProgressState,
    test_support::{
        membership::{hash, plan},
        temp_dir,
    },
};
use std::{fs, path::PathBuf};

fn prepare() -> (PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-retained-progress");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    create_operation_plan(&layout, &plan).unwrap();
    for authority in plan.attempt_authorities().unwrap() {
        drop(AttemptJournalGuard::create(&layout, authority).unwrap());
    }
    (root, layout)
}

fn receipt(journal: &mut AttemptJournalGuard<'_>, outcome: MutationOutcomeRecord) {
    journal
        .record_mutation(MutationReceiptRequest {
            attempt: journal.record().unwrap().view().pending_mutation.unwrap(),
            request: journal
                .record()
                .unwrap()
                .authority()
                .binding()
                .request()
                .into(),
            outcome,
            evidence: hash("12").hash().into(),
        })
        .unwrap();
}

#[test]
fn dependency_gate_preserves_bytes_then_reserves_under_original_applied_evidence() {
    let (root, layout) = prepare();
    let plan = plan();
    let mut child =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let original = fs::read(child.path()).unwrap();
    assert!(matches!(
        child.reserve_planned_mutation(&plan.digest()),
        Err(ExecutionProgressPersistenceError::DependenciesUnapplied(7))
    ));
    assert_eq!(fs::read(child.path()).unwrap(), original);
    drop(child);
    let mut parent =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(0).unwrap()).unwrap();
    assert_eq!(parent.reserve_planned_mutation(&plan.digest()).unwrap(), 1);
    receipt(&mut parent, MutationOutcomeRecord::Applied);
    drop(parent);
    let mut child =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(7).unwrap()).unwrap();
    assert_eq!(child.reserve_planned_mutation(&plan.digest()).unwrap(), 1);
    receipt(&mut child, MutationOutcomeRecord::NotApplied);
    assert_eq!(child.reserve_planned_mutation(&plan.digest()).unwrap(), 2);
    receipt(&mut child, MutationOutcomeRecord::Applied);
    assert!(matches!(
        child.reserve_planned_mutation(&plan.digest()),
        Err(ExecutionProgressPersistenceError::Journal(
            AttemptJournalError::Record(_)
        ))
    ));
    drop(child);
    let view = read_execution_progress(&layout, &plan.digest()).unwrap();
    assert_eq!(view.applied_operations, 2);
    assert_eq!(view.attempts.mutations_used, 3);
    assert_eq!(view.attempts.mutations_remaining, 0);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn original_complete_coverage_and_selected_identity_are_required_before_spending() {
    let (root, layout) = prepare();
    let plan = plan();
    let mut parent =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(0).unwrap()).unwrap();
    let original = fs::read(parent.path()).unwrap();
    assert!(matches!(
        parent.reserve_planned_mutation(&hash("56")),
        Err(ExecutionProgressPersistenceError::Plan(
            OperationPlanPersistenceError::DigestMismatch
        ))
    ));
    fs::rename(
        root.join("attempt-7.json"),
        root.join("retained-attempt-7.json"),
    )
    .unwrap();
    assert!(matches!(
        parent.reserve_planned_mutation(&plan.digest()),
        Err(ExecutionProgressPersistenceError::Journal(_))
    ));
    assert_eq!(fs::read(parent.path()).unwrap(), original);
    assert!(!root.join("attempt-7.json").exists());
    fs::rename(
        root.join("retained-attempt-7.json"),
        root.join("attempt-7.json"),
    )
    .unwrap();
    let mut changed = serde_json::to_value(&plan).unwrap();
    changed["context"]["release"] = serde_json::json!("56".repeat(32));
    let changed =
        serde_json::from_value::<crate::model::operation_plan::OperationPlanRecord>(changed)
            .unwrap();
    assert!(matches!(
        read_original_journals(
            &layout,
            &changed.attempt_authorities().unwrap(),
            Some(parent.record().unwrap())
        ),
        Err(AttemptJournalError::AuthorityMismatch)
    ));
    assert_eq!(fs::read(parent.path()).unwrap(), original);
    drop(parent);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn held_originals_fail_without_releasing_custody_or_consuming_allowance() {
    let (root, layout) = prepare();
    let plan = plan();
    let parent = AttemptJournalGuard::open(&layout, &plan.attempt_authority(0).unwrap()).unwrap();
    let mut child =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let original = fs::read(child.path()).unwrap();
    assert!(matches!(
        child.reserve_planned_mutation(&plan.digest()),
        Err(ExecutionProgressPersistenceError::Journal(
            AttemptJournalError::Lock(_)
        ))
    ));
    assert!(matches!(
        read_execution_progress(&layout, &plan.digest()),
        Err(ExecutionProgressPersistenceError::Journal(
            AttemptJournalError::Lock(_)
        ))
    ));
    assert_eq!(fs::read(child.path()).unwrap(), original);
    drop(child);
    drop(parent);
    assert_eq!(
        read_execution_progress(&layout, &plan.digest())
            .unwrap()
            .attempts
            .mutations_used,
        0
    );
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lost_observation_reopens_pending_original_spending_without_writes_or_reissue() {
    let (root, layout) = prepare();
    let plan = plan();
    let mut parent =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(0).unwrap()).unwrap();
    let mutation = parent.reserve_planned_mutation(&plan.digest()).unwrap();
    assert_eq!(
        parent
            .reserve_planned_observation(&plan.digest(), mutation, hash("56").hash())
            .unwrap(),
        2
    );
    let original = fs::read(parent.path()).unwrap();
    drop(parent);
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    for _ in 0..2 {
        let view = read_execution_progress(&layout, &plan.digest()).unwrap();
        assert_eq!(
            view.operations[0].state,
            OperationProgressState::ObservationUnresolved
        );
        assert_eq!(view.attempts.mutations_used, 1);
        assert_eq!(view.attempts.observations_used, 1);
        assert_eq!(fs::read(root.join("attempt-0.json")).unwrap(), original);
    }
    let mut parent =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(0).unwrap()).unwrap();
    assert!(matches!(
        parent.reserve_planned_observation(&plan.digest(), mutation, hash("56").hash()),
        Err(ExecutionProgressPersistenceError::Journal(
            AttemptJournalError::Record(_)
        ))
    ));
    assert!(matches!(
        parent.reserve_planned_mutation(&plan.digest()),
        Err(ExecutionProgressPersistenceError::Journal(
            AttemptJournalError::Record(AttemptJournalRecordError::ObservationPending {
                attempt: 2
            })
        ))
    ));
    assert_eq!(fs::read(parent.path()).unwrap(), original);
    drop(parent);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn premature_retained_attempts_reject_without_changing_original_accounting() {
    let (root, layout) = prepare();
    let plan = plan();
    let mut child =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let mutation = child.reserve_mutation().unwrap();
    let original = fs::read(child.path()).unwrap();
    assert!(matches!(
        child.reserve_planned_observation(&plan.digest(), mutation, hash("56").hash()),
        Err(ExecutionProgressPersistenceError::Policy(
            ExecutionProgressError::PrematureAttempt {
                operation_sequence: 7,
                prerequisite: 0
            }
        ))
    ));
    assert_eq!(fs::read(child.path()).unwrap(), original);
    drop(child);
    assert!(matches!(
        read_execution_progress(&layout, &plan.digest()),
        Err(ExecutionProgressPersistenceError::Policy(
            ExecutionProgressError::PrematureAttempt { .. }
        ))
    ));
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
