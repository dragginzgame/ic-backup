//! Pure metadata-upload inventory association; no allocation outcome or receipt.

use crate::{
    model::{
        attempt_journal::AttemptJournalRecord, ic_observation::IcObservationResponse,
        ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest,
    },
    policy::ic_observation::{
        IcObservationAssociationError, IcObservationResponseView, ObservationAssociation,
        validate_association,
    },
};

/// Recheck original pending attempts and associate bounded exact list evidence.
///
/// Reuses the existing passive claims, inventory decoder and raw-evidence identity.
/// No authentication, freshness, exclusive metadata-allocation attribution or
/// outcome follows from wire shape, timestamp, size or zero/one/many snapshots.
/// This performs no IO, provider call, spending or journal transition.
/// # Errors
/// Rejects changed original reservations, actual claims or invalid bounded wire.
pub fn validate_response<'a>(
    request: &IcSnapshotUploadObservationRequest<'a, '_>,
    journal: &AttemptJournalRecord,
    response: &'a IcObservationResponse,
) -> Result<IcObservationResponseView<'a>, IcObservationAssociationError> {
    request.validate_journal(journal)?;
    validate_association(
        &ObservationAssociation {
            authority: request.authority(),
            mutation_attempt: request.mutation_attempt(),
            observation_attempt: request.observation_attempt(),
            payload: request.payload(),
            context: request.plan().context(),
        },
        response,
    )
}

#[cfg(test)]
mod tests;
