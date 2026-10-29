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
    let plan = stage.plan();
    payload
        .validate_mutation_binding(authority.binding())
        .map_err(IcMutationRequestError::from)
        .map_err(E::from)?;
    let mut journal =
        AttemptJournalGuard::open(stage.layout().map_err(E::from)?, authority).map_err(E::from)?;
    journal
        .reserve_planned_mutation(&plan.digest())
        .map_err(E::from)?;
    let request = IcMutationRequest::new(
        plan,
        authority.binding().operation_sequence(),
        journal.record().map_err(E::from)?,
        payload,
    )
    .map_err(E::from)?;
    admit(&request).await?;
    let acknowledgement = provider
        .submit_mutation(&request, journal.record().map_err(E::from)?)
        .await
        .map_err(E::from)?;
    let result = complete(&mut journal, &request, &acknowledgement).await;
    Ok((acknowledgement, result))
}
