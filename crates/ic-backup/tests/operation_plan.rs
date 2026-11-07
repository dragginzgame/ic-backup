//! Public immutable plan-to-journal binding, with local evidence and no management backend.

#[expect(
    dead_code,
    reason = "this integration target uses only a subset of the shared fixture helpers"
)]
mod support;

use ic_backup::{
    model::attempt_journal::AttemptJournalRecordError,
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_operation_plan,
        read_operation_plan,
    },
};
use std::fs;

#[test]
fn original_plan_authority_reopens_consumed_attempts_without_rebinding_or_replenishment() {
    let root = support::temp_root("ic-backup-public-plan");
    let layout = BackupLayoutGuard::acquire(&root).expect("exclusion");
    let plan = support::single_operation_plan("aaaaa-aa", 7, &"ef".repeat(32));
    let intent = plan.digest();
    create_operation_plan(&layout, &plan).expect("retain exact intent");
    let original = fs::read(root.join("operation-plan.json")).expect("original bytes");
    let authority = plan
        .attempt_authority(7)
        .expect("derive exact identity and original limits");
    let mut guard =
        AttemptJournalGuard::create(&layout, authority.clone()).expect("one original journal");
    assert_eq!(
        guard
            .reserve_mutation()
            .expect("consume locally before caller-owned call"),
        1
    );
    drop(guard);
    let retained = read_operation_plan(&layout, &intent).expect("original reviewed digest");
    let repeated = retained
        .attempt_authority(7)
        .expect("repeat declaration only");
    assert_eq!(authority, repeated);
    let mut guard =
        AttemptJournalGuard::open(&layout, &repeated).expect("original pending journal");
    let view = guard.record().expect("retained accounting").view();
    assert_eq!(view.mutations_used, 1);
    assert_eq!(view.mutations_remaining, 0);
    assert_eq!(view.pending_mutation, Some(1));
    assert!(matches!(
        guard.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { attempt: 1 }
        ))
    ));
    drop(guard);
    assert!(AttemptJournalGuard::create(&layout, repeated).is_err());
    assert_eq!(
        fs::read(root.join("operation-plan.json")).expect("unchanged plan"),
        original
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}
