//! One retained, single-dispatch update shared by the configured port adapters.

use crate::{AgentTransport, PreparedUpdate, ReservedUpdate, UpdateOutcome};
use ic_agent::RequestId;
use ic_backup::model::attempt_journal::AttemptJournalRecord;

pub(crate) async fn submit_once<E>(
    transport: &AgentTransport,
    request: ReservedUpdate<'_>,
    journal: &AttemptJournalRecord,
    mut retain_ingress: impl FnMut(&PreparedUpdate<'_>) -> Result<(), E>,
    mut retain_reply: impl FnMut(&RequestId, &[u8]) -> Result<(), E>,
    unsupported: fn() -> E,
    indeterminate: fn() -> E,
) -> Result<(RequestId, Vec<u8>), E> {
    let prepared = transport
        .prepare(request, journal)
        .map_err(|_| unsupported())?;
    retain_ingress(&prepared)?;
    let outcome = prepared.submit().await.map_err(|_| indeterminate())?;
    let UpdateOutcome::Replied { request_id, reply } = outcome else {
        return Err(indeterminate());
    };
    // A failed post-dispatch retainer must never turn an uncertain effect into a retry.
    retain_reply(&request_id, &reply).map_err(|_| indeterminate())?;
    Ok((request_id, reply))
}
