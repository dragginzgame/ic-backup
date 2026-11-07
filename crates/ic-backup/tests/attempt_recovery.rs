//! External public-API accounting journey using local receipts; no IC backend.

#[expect(
    dead_code,
    reason = "this integration target uses only a subset of the shared fixture helpers"
)]
mod support;

use ic_backup::ops::persistence::ExecutionProgressPersistenceError as ProgressError;
use ic_backup::{
    model::attempt_journal::{
        AttemptJournalRecordError, ObservationOutcomeRecord, ObservationReceiptRequest,
    },
    ops::persistence::{AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard},
};
use std::fs;

#[test]
fn exhausted_pending_attempt_reconciles_under_the_original_public_authority() {
    let root = support::temp_root("ic-backup-public-attempt");
    let layout = BackupLayoutGuard::acquire(&root).expect("exclude other layout users");
    let plan = support::single_operation_plan("aaaaa-aa", 3, &"01".repeat(32));
    ic_backup::ops::persistence::create_operation_plan(&layout, &plan).unwrap();
    let authority = plan.attempt_authority(3).unwrap();
    let mut guard =
        AttemptJournalGuard::create(&layout, authority.clone()).expect("retain original authority");
    let mutation = guard
        .reserve_planned_mutation(&plan.digest())
        .expect("consume before caller-owned effect");
    drop(guard);
    let mut guard =
        AttemptJournalGuard::open(&layout, &authority).expect("recover after lost response");
    assert_eq!(
        guard
            .record()
            .expect("accounting")
            .view()
            .mutations_remaining,
        0
    );
    assert!(matches!(
        guard.reserve_planned_mutation(&plan.digest()),
        Err(ProgressError::Journal(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { attempt: 1 }
        )))
    ));
    let request = "45".repeat(32);
    let observation = guard
        .reserve_planned_observation(&plan.digest(), mutation, &request)
        .expect("consume separate observation authority");
    guard
        .record_observation(ObservationReceiptRequest {
            attempt: observation,
            request,
            outcome: ObservationOutcomeRecord::Applied,
            evidence: "67".repeat(32),
        })
        .expect("retain local fixture receipt");
    let retained = guard.record().expect("applied evidence").view();
    assert!(retained.applied);
    assert_eq!(retained.mutations_used, 1);
    assert_eq!(retained.observations_used, 1);
    drop(guard);
    let guard = AttemptJournalGuard::open(&layout, &authority).expect("local applied replay");
    assert_eq!(
        guard.record().expect("retained accounting").view(),
        retained
    );
    drop(guard);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}
