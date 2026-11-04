//! Single original mutation reservation and dispatch under continuous journal exclusion.

use crate::{
    model::{
        attempt_journal::AttemptAuthorityRecord,
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest, IcMutationRequestError},
        ic_request::IcManagementRequestRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionProgressPersistenceError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError,
    },
    ports::ic_mutation::{IcMutationProvider, IcMutationProviderError},
};

// Domain callbacks retain their admission/re-admission order and post-reply errors.
// The journal drops on return, before a caller can publish a stage checkpoint.
pub(super) async fn submit_with<E, T>(
    stage: &ExecutionStageGuard<'_>,
    authority: &AttemptAuthorityRecord,
    payload: &IcManagementRequestRecord,
    provider: &mut impl IcMutationProvider,
    admit: impl AsyncFnOnce(&IcMutationRequest<'_>) -> Result<(), E>,
    complete: impl AsyncFnOnce(
        &mut AttemptJournalGuard<'_>,
        &IcMutationRequest<'_>,
        &IcMutationAcknowledgement,
    ) -> T,
) -> Result<(IcMutationAcknowledgement, T), E>
where
    E: From<IcMutationRequestError>
        + From<ExecutionWorkflowPersistenceError>
        + From<AttemptJournalError>
        + From<ExecutionProgressPersistenceError>
        + From<IcMutationProviderError>,
{
    payload
        .validate_mutation_binding(authority.binding())
        .map_err(IcMutationRequestError::from)
        .map_err(E::from)?;
    execute_reserved(
        stage,
        authority,
        |_| Ok(()),
        |journal| {
            IcMutationRequest::new(
                stage.plan(),
                authority.binding().operation_sequence(),
                journal,
                payload,
            )
            .map_err(E::from)
        },
        admit,
        async |request, journal| {
            provider
                .submit_mutation(request, journal)
                .await
                .map_err(E::from)
        },
        complete,
    )
    .await
}

// The sole open -> original validation -> reserve -> bind -> admit -> submit
// sequence. Request/reply semantics stay with each typed caller. Keep the same
// journal through both awaits and completion; checkpointing happens after return.
pub(super) async fn execute_reserved<Q, R, T, E>(
    stage: &ExecutionStageGuard<'_>,
    authority: &AttemptAuthorityRecord,
    validate: impl FnOnce(&crate::model::attempt_journal::AttemptJournalRecord) -> Result<(), E>,
    bind: impl FnOnce(&crate::model::attempt_journal::AttemptJournalRecord) -> Result<Q, E>,
    admit: impl AsyncFnOnce(&Q) -> Result<(), E>,
    submit: impl AsyncFnOnce(&Q, &crate::model::attempt_journal::AttemptJournalRecord) -> Result<R, E>,
    complete: impl AsyncFnOnce(&mut AttemptJournalGuard<'_>, &Q, &R) -> T,
) -> Result<(R, T), E>
where
    E: From<ExecutionWorkflowPersistenceError>
        + From<AttemptJournalError>
        + From<ExecutionProgressPersistenceError>,
{
    let mut journal =
        AttemptJournalGuard::open(stage.layout().map_err(E::from)?, authority).map_err(E::from)?;
    validate(journal.record().map_err(E::from)?)?;
    journal
        .reserve_planned_mutation(&stage.plan().digest())
        .map_err(E::from)?;
    let request = bind(journal.record().map_err(E::from)?)?;
    admit(&request).await?;
    let reply = submit(&request, journal.record().map_err(E::from)?).await?;
    let result = complete(&mut journal, &request, &reply).await;
    Ok((reply, result))
}
