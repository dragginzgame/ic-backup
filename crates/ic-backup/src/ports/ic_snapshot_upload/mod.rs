//! Single reserved source-bound snapshot upload contract; no installed transport.

use crate::{
    model::{ic_mutation::IcMutationAcknowledgement, ic_snapshot_upload::IcSnapshotUploadAttempt},
    ports::ic_mutation::IcMutationProviderError,
};

/// Integration-owned submission of one originally accounted metadata or data update.
///
/// Retain the full original source and upload plans, exact metadata/tree evidence,
/// encoded payload and pending mutation reservation before invocation. Qualify
/// authentic complete source bytes, same-network/release/target association, fresh
/// actual network/caller/target/controller permissions and all original prerequisites.
/// Data additionally requires exclusive attribution of the new destination to the
/// original metadata allocation; a passive returned ID or inventory cardinality
/// supplies no such proof. Each data payload needs its own original plan/reservation,
/// after binding that destination and exact source range/chunk. Metadata spending
/// cannot pay for data, and neither payload permits replacing/deleting the source.
///
/// Qualify stable source/payload bytes and exclusive command/dispatch custody,
/// including proof that this exact reservation has never been dispatched. Local
/// verification and a reconstructed pending attempt supply no dispatch permit.
/// After interruption reconcile the original effect instead of submitting again.
/// Retain source references and application/fence obligations independently; an
/// upload acknowledgement establishes no complete transfer or load/start safety.
///
/// Send exactly the payload's management receiver, effective routing target, method
/// and Candid bytes as one host replicated update. No query/proxy substitution,
/// implicit funding, allocation plus data batching, hidden retries or observations
/// may occur. Further calls require independent prior durable accounting and fresh
/// qualified admission. The port does not allocate or mutate that accounting.
///
/// Return the existing bounded passive acknowledgement with exact raw bytes and
/// actual context/target/original authority/attempt claims. Use
/// [`crate::policy::ic_snapshot_upload::validate_acknowledgement`] to recheck the
/// current journal and tighter method-specific wire bounds. Authentication,
/// chronology and attribution remain integration-owned; association produces no
/// Applied/NotApplied/Uncertain receipt. All failures/drop/death retain pending
/// spending, source references, obligations and recovery evidence without refunds
/// or automatic retry. Terminal replay invokes no provider. No default exists.
pub trait IcSnapshotUploadProvider {
    /// Submit one exact original upload update without retry or follow-up calls.
    /// # Errors
    /// Reuses the IC update failure owner; no variant grants retry or nonapplication.
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError>;
}

#[cfg(test)]
mod tests;
