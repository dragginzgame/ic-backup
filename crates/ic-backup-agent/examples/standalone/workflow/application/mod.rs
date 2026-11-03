//! Application authority and evidence stay explicit at the complete workflow boundary.

use super::{
    Digest, ExecutionWorkflowRecord, MutationOutcomeRecord, MutationReceiptRequest,
    OperationPlanRecord, Path, PreparedUpdate, ReservedUpdate, Stage, io, retain,
};
use ic_backup::{
    model::{
        ic_mutation::IcMutationRequest,
        ic_snapshot_metadata::IcSnapshotMetadataReply,
        restore_safety::{
            RestoreSafetyObservation, RestoreSafetyRequest, RestoreSafetyRequirementRecord,
        },
    },
    view::restore_reference_release::TerminalRestoreEvidence,
};
use std::future::Future;

/// Exact returned bytes and original operation identities under stage custody.
/// This passive input does not authenticate application meaning or grant a receipt.
pub struct Reply<'a> {
    /// Held stage's private evidence directory.
    pub root: &'a Path,
    /// Original stage kind.
    pub stage: Stage,
    /// Full original operation plan and allowances.
    pub plan: &'a OperationPlanRecord,
    /// Original opaque operation identity.
    pub sequence: u64,
    /// Already reserved mutation attempt.
    pub attempt: u32,
    /// Exact payload commitment.
    pub request: &'a Digest,
    /// Exact original arguments.
    pub arguments: &'a [u8],
    /// Complete authenticated transport reply; semantics still require qualification.
    pub bytes: &'a [u8],
}

/// Required application decisions; no permissive authority or safety default exists.
/// Remote checks need their own original accounting. Async methods impose no Send bound.
pub trait Application {
    /// Qualify fresh membership, release, direct authority, read access, consistency
    /// and exclusive never-dispatched custody for the actual original stage.
    fn admit(
        &self,
        stage: Stage,
        plan: &OperationPlanRecord,
    ) -> impl Future<Output = io::Result<Digest>>;

    /// Durably retain signed bytes/ID before dispatch, refusing prior/uncertain witnesses.
    /// No remote calls or cancellable work are permitted here.
    /// # Errors
    /// Rejects unqualified custody, duplicate witnesses or failed durable retention.
    fn retain_ingress(
        &self,
        stage: Stage,
        root: &Path,
        request: &ReservedUpdate<'_>,
        prepared: &PreparedUpdate<'_>,
    ) -> io::Result<()>;

    /// Independently attribute and durably retain exact reply/argument bytes before
    /// awaiting cancellable qualification work. Return an explicit original receipt.
    /// `retain_applied_reply` is available after actual application qualification.
    fn qualify_reply(
        &self,
        reply: Reply<'_>,
    ) -> impl Future<Output = io::Result<MutationReceiptRequest>>;

    /// Declare the actual original restore safety lane and any original fence binding.
    /// The returned requirement must bind both original plans and this manifest.
    /// # Errors
    /// Rejects an unsupported or unqualified application safety lane.
    fn restore_requirement(
        &self,
        restore: &OperationPlanRecord,
        source: &OperationPlanRecord,
        manifest: &Digest,
    ) -> io::Result<RestoreSafetyRequirementRecord>;

    /// Qualify current lifecycle, obligations/fence and whole-selection acceptance.
    fn restore_safety(
        &self,
        original: &IcMutationRequest<'_>,
        request: &RestoreSafetyRequest<'_>,
        acceptance: Option<&Digest>,
    ) -> impl Future<Output = io::Result<RestoreSafetyObservation>>;

    /// Freshly admit independently retained stopped post-load application acceptance.
    /// This local check also gates preparation of the start stage. Any remote
    /// verification must have been separately accounted; replay makes no IC call.
    /// # Errors
    /// Rejects missing, changed, stale or unqualified application acceptance.
    fn acceptance(
        &self,
        restore: &ExecutionWorkflowRecord,
        metadata: &IcSnapshotMetadataReply<'_>,
        destination: &[u8],
    ) -> io::Result<Digest>;

    /// Qualify complete terminal application/effect evidence and all retained command
    /// identities. Complete stage replay alone never establishes this authority.
    /// This callback is effect-free and must also qualify already-released replay.
    /// # Errors
    /// Rejects incomplete application/effect settlement or command custody.
    fn terminal(
        &self,
        restore: &ExecutionWorkflowRecord,
        acceptance: &Digest,
    ) -> io::Result<TerminalRestoreEvidence>;
}

/// Durably retain an independently qualified Applied reply and build its explicit receipt.
/// Call only after authenticating original attribution and application meaning.
/// # Errors
/// Refuses occupied evidence paths or IO failures; no journal is changed here.
pub fn retain_applied_reply(
    reply: &Reply<'_>,
    evidence: &Digest,
) -> io::Result<MutationReceiptRequest> {
    retain(
        &reply
            .root
            .join(format!("arguments-{}.candid", reply.sequence)),
        reply.arguments,
    )?;
    retain(
        &reply.root.join(format!("reply-{}.candid", reply.sequence)),
        reply.bytes,
    )?;
    Ok(MutationReceiptRequest {
        attempt: reply.attempt,
        request: reply.request.hash().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: evidence.hash().into(),
    })
}
