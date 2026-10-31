//! Passive native fixture replies and explicit receipts; no authentication or IC effects.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, MutationOutcomeRecord, MutationReceiptRequest},
    ic_update_response::IcUpdateResponseInput,
    operation_plan::PlanContextRecord,
};

pub(crate) fn input(
    authority: &AttemptAuthorityRecord,
    attempt: u32,
    context: &PlanContextRecord,
    reply: Vec<u8>,
) -> IcUpdateResponseInput {
    IcUpdateResponseInput {
        authority: authority.digest(),
        mutation_attempt: attempt,
        context: context.clone(),
        target: authority.binding().target().into(),
        evidence: ArtifactChecksumRecord::from_bytes(&reply),
        reply,
    }
}

pub(crate) fn applied_receipt(
    attempt: u32,
    request: &ArtifactChecksumRecord,
    evidence: &ArtifactChecksumRecord,
) -> MutationReceiptRequest {
    MutationReceiptRequest {
        attempt,
        request: request.hash().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: evidence.hash().into(),
    }
}
