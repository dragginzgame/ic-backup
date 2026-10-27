//! Exact original Applied receipt admission; callers keep domain work before recording.

use crate::model::attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest};

pub(super) fn admit_applied(
    receipt: MutationReceiptRequest,
    expected_attempt: u32,
    expected_request: &str,
) -> Option<MutationReceiptRequest> {
    (receipt.outcome == MutationOutcomeRecord::Applied
        && receipt.attempt == expected_attempt
        && receipt.request == expected_request)
        .then_some(receipt)
}
