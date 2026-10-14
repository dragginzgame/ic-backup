//! Ordered matching of declared operation context to a fresh observation.

use crate::model::{attempt_journal::OperationBindingRecord, operation_plan::PlanContextRecord};

pub(super) fn first_mismatch(
    binding: &OperationBindingRecord,
    observed: &PlanContextRecord,
) -> Option<&'static str> {
    if observed.network() != binding.network() {
        Some("network")
    } else if observed.caller() != binding.caller() {
        Some("caller")
    } else if observed.release() != binding.release() {
        Some("release")
    } else {
        None
    }
}
