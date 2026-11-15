//! Pure restore status prerequisites; no request authentication or spending.
use super::RestorePlan;
use crate::store::{
    CaptureStatusError, CaptureStatusReplyError, capture::status::StatusRequirements,
};
use ic_platform::management_canister_types::CanisterStatusResult;

impl RestorePlan {
    fn status_requirements(&self) -> StatusRequirements {
        StatusRequirements {
            module_sha256: self.source().module_sha256,
            sender: self.request().sender,
        }
    }

    /// Check stopped state, the original source module and this restore's direct sender.
    ///
    /// Shares capture's bounded controller and module policy, including its existing
    /// typed errors. Neither the capture sender nor the approving initiator can stand
    /// in for the new restore sender. The supplied DTO is unchanged.
    ///
    /// This pure check does not authenticate a request, target, network or timing,
    /// establish application rewind safety, consume authority or prove a load outcome.
    /// # Errors
    /// Refuses running/stopping state, missing/different module, duplicate/excessive
    /// controllers and absence of the restore sender.
    pub fn validate_status(
        &self,
        observed: &CanisterStatusResult,
    ) -> Result<(), CaptureStatusError> {
        self.status_requirements().validate_status(observed)
    }

    /// Apply restore status policy through the existing bounded Platform wire reader.
    ///
    /// Shares capture's exact single-argument framing, 1 MiB reply limit, bounded
    /// controllers/hash allocation and separate skipped-work allowance. Other fields
    /// are skipped, not semantically validated. Retain and authenticate original raw
    /// evidence separately; success grants no dispatch, reservation or settlement.
    /// # Errors
    /// Returns the existing status decoding/policy errors for malformed, excessive
    /// or policy-invalid replies. No file, provider, clock or journal is touched.
    pub fn validate_status_reply(&self, reply: &[u8]) -> Result<(), CaptureStatusReplyError> {
        self.status_requirements().validate_status_reply(reply)
    }
}
