//! Shared exact historical certificate-window arithmetic.
use super::StatusAdmissionRecord;
use ic_platform::certification::historical::{TimePolicy, TimeWindow};
const MAX_LIVE_WINDOW_NS: u64 = 300_000_000_000;
/// Explicit original certificate policy; there are no implicit freshness defaults.
#[derive(Clone, Copy, Debug)]
pub struct CaptureTiming {
    /// Maximum certificate age relative to the start of preparation.
    pub certificate_max_age_ns: u64,
    /// Separate maximum root-delegation age relative to that same reference.
    pub delegation_max_age_ns: u64,
    /// Extra future allowance beyond the original submission deadline.
    pub max_clock_skew_ns: u64,
}

impl CaptureTiming {
    // Shared exact time-window arithmetic for direct capture and restore preparation.
    pub(in crate::store) fn policy(
        &self,
        admission: &StatusAdmissionRecord,
        reference: u64,
    ) -> Option<TimePolicy> {
        let future = admission
            .submit_deadline_ns
            .checked_sub(reference)
            .filter(|span| *span <= MAX_LIVE_WINDOW_NS)?
            .checked_add(self.max_clock_skew_ns)?;
        if reference >= admission.prepare_deadline_ns {
            return None;
        }
        let window = |max_age_ns| TimeWindow {
            reference_ns: reference,
            max_age_ns,
            max_future_skew_ns: future,
        };
        Some(TimePolicy {
            certificate: window(self.certificate_max_age_ns),
            delegation: window(self.delegation_max_age_ns),
        })
    }
}
