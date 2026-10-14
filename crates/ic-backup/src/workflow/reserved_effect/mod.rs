//! One provider invocation under an already consumed, caller-held journal reservation.

use crate::{
    model::attempt_journal::AttemptJournalRecord,
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, ExecutionStageGuard,
        ExecutionWorkflowPersistenceError,
    },
};

pub(super) async fn submit_once<R, A, E>(
    journal: &AttemptJournalGuard<'_>,
    request: &R,
    admit: impl AsyncFnOnce(&R) -> Result<(), E>,
    recheck_before_dispatch: impl FnOnce() -> Result<(), E>,
    submit: impl AsyncFnOnce(&R, &AttemptJournalRecord) -> Result<A, E>,
    journal_error: fn(AttemptJournalError) -> E,
) -> Result<A, E> {
    admit(request).await?;
    recheck_before_dispatch()?;
    submit(request, journal.record().map_err(journal_error)?).await
}

pub(super) enum ReturnedError<E> {
    Journal(AttemptJournalError),
    Association(E),
    Stage(ExecutionWorkflowPersistenceError),
}

pub(super) fn check_returned<'stage, R, A, E>(
    stage: &'stage ExecutionStageGuard<'_>,
    journal: &AttemptJournalGuard<'_>,
    request: &R,
    reply: &A,
    associate: impl FnOnce(&R, &AttemptJournalRecord, &A) -> Result<(), E>,
) -> Result<&'stage BackupLayoutGuard, ReturnedError<E>> {
    let record = journal.record().map_err(ReturnedError::Journal)?;
    associate(request, record, reply).map_err(ReturnedError::Association)?;
    stage.layout().map_err(ReturnedError::Stage)
}
