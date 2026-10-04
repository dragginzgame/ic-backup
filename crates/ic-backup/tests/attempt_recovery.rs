//! External public-API accounting journey using local receipts; no IC backend.

use ic_backup::{
    model::attempt_journal::{
        AttemptAuthorityRecord, AttemptBudgetRecord, AttemptJournalRecordError,
        ObservationOutcomeRecord, ObservationReceiptRequest, OperationBindingRecord,
        OperationBindingRequest,
    },
    ops::persistence::{AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard},
};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn exhausted_pending_attempt_reconciles_under_the_original_public_authority() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("parent")
        .join(format!(
            "ic-backup-public-attempt-{}-{nonce}",
            std::process::id()
        ));
    fs::create_dir(&root).expect("layout");
    let layout = BackupLayoutGuard::acquire(&root).expect("exclude other layout users");
    let authority = AttemptAuthorityRecord::new(
        OperationBindingRecord::new(&OperationBindingRequest {
            intent: "ab".repeat(32),
            operation_sequence: 3,
            network: "cd".repeat(32),
            caller: "2vxsx-fae".into(),
            target: "aaaaa-aa".into(),
            release: "ef".repeat(32),
            request: "01".repeat(32),
        })
        .expect("exact selected binding"),
        AttemptBudgetRecord::new(1, 1).expect("original finite ceilings"),
    );
    let mut guard =
        AttemptJournalGuard::create(&layout, authority.clone()).expect("retain original authority");
    let mutation = guard
        .reserve_mutation()
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
        guard.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { attempt: 1 }
        ))
    ));
    let request = "45".repeat(32);
    let observation = guard
        .reserve_observation(mutation, &request)
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
