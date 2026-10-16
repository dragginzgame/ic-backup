//! Ordered matching of declared operation context to a fresh observation.

use crate::model::{
    artifacts::ArtifactChecksumRecord, attempt_journal::OperationBindingRecord,
    operation_plan::PlanContextRecord,
};

pub(super) fn admit_request_context<E>(
    expected: &ArtifactChecksumRecord,
    observed_request: &ArtifactChecksumRecord,
    binding: &OperationBindingRecord,
    observed_context: &PlanContextRecord,
    request_error: E,
    context_error: impl FnOnce(&'static str) -> E,
) -> Result<(), E> {
    if observed_request != expected {
        return Err(request_error);
    }
    if let Some(field) = first_mismatch(binding, observed_context) {
        return Err(context_error(field));
    }
    Ok(())
}

pub(super) fn admit_target_limit<E>(
    binding: &OperationBindingRecord,
    observed_target: &str,
    limit: u32,
    reported: u32,
    target_error: E,
    limit_error: impl FnOnce(u32, u32) -> E,
) -> Result<(), E> {
    if observed_target != binding.target() {
        return Err(target_error);
    }
    admit_limit(limit, reported, limit_error)
}

pub(super) fn admit_limit<E>(
    limit: u32,
    reported: u32,
    limit_error: impl FnOnce(u32, u32) -> E,
) -> Result<(), E> {
    if reported > limit {
        return Err(limit_error(limit, reported));
    }
    Ok(())
}

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
