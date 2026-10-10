//! One originally reserved async replicated-update metadata/data read.

use crate::{
    model::attempt_journal::AttemptJournalRecord,
    model::ic_snapshot_transfer_read::{
        IcSnapshotTransferReadRequest, IcSnapshotTransferReadResponse,
    },
    ports::ic_observation::IcObservationProviderError,
};

/// Integration-owned authenticated transfer read under original durable accounting.
///
/// Retain the full original plan and exact payload before reserving its existing
/// mutation-lane replicated update. Qualify actual network/caller/target, fresh
/// method-specific snapshot read access, authentic metadata/raw-ID association and
/// exclusive command/dispatch custody proving this reservation was never dispatched.
/// A reconstructed pending request supplies no reissue permit. After interruption
/// retain pending spending and stop; loss is not a settled observation or a refund.
///
/// Submit exactly one host replicated update with the original management receiver,
/// effective routing target, method and argument bytes. No query/proxy substitution,
/// hidden retries, follow-up status/list/data/metadata calls, funding or mutation.
/// Further calls require their own prior original accounting and fresh admission.
/// Return exact bounded raw bytes and actual context/target/authority/attempt claims.
/// Integrations authenticate evidence independently and explicitly use the existing
/// journal owner to record a qualified successful read; passive association does not.
/// Keep failures/drop/death pending with source references/fences and evidence intact.
/// Read bytes alone establish no full transfer, immutable snapshot custody, durable
/// publication, restoration safety or terminal/reference/fence release. Terminal
/// replay invokes no provider. A configured Agent adapter requires explicit durable
/// signed-ingress retention; no default provider or application admission exists.
/// Recheck the supplied currently guarded original journal before submission. The
/// coordinator holds its exclusion across awaits; cancellation preserves spending.
/// This contract imposes no executor, runtime dependency or `Send` bound.
pub trait IcSnapshotTransferReadProvider {
    /// Read once, without hidden calls or originally reserved update reissues.
    /// # Errors
    /// Reuses the observation failure owner; indeterminate is not settled uncertainty.
    fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> impl Future<Output = Result<IcSnapshotTransferReadResponse, IcObservationProviderError>>;
}
