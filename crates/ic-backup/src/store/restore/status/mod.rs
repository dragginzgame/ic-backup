//! Local restore status policy and signed-request authentication; no spending.
use super::RestorePlan;
use crate::store::{
    CaptureError, CaptureStatusError, CaptureStatusReplyError, PreparedStatus,
    StatusAdmissionRecord, StatusRequestError, capture::status::StatusRequirements,
};
use ic_platform::{
    candid_codec,
    management_canister_types::{CanisterIdRecord, CanisterStatusResult},
};

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

    /// Authenticate the exact signed initial status request for this restore.
    ///
    /// Checks the original single-call network/evidence ceilings, complete retained
    /// request framing, canonical Platform arguments, management destination, approved
    /// restore sender, ingress expiry and request ID. Reuses capture's strict direct
    /// Ed25519/low-S secp256k1 verification; delegation and sender-info are refused.
    /// Original envelope and observation bytes are unchanged.
    ///
    /// This local check reads no clock and consumes no allowance. It does not prove
    /// durable reservation, remaining cumulative spending, live application safety,
    /// routing/trust, observation policy or certified results. Those remain required
    /// before dispatch. An expired original request may authenticate historically;
    /// authentication never authorizes sending it again.
    /// # Errors
    /// Refuses invalid/oversized admission, altered target/method/arguments/identity,
    /// malformed envelope or unsupported/invalid signatures. Errors retain no secrets.
    pub fn authenticate_status_request(
        &self,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<(), StatusRequestError> {
        let initial = crate::store::capture::ledger::initial_bytes(
            self.encoded(),
            self.request().limits.evidence_bytes,
        )
        .map_err(|e| StatusRequestError::Admission(e.into()))?;
        admission
            .validate_limits(
                self.request().limits.network_requests,
                self.request().limits.evidence_bytes,
                initial.len(),
            )
            .map_err(StatusRequestError::Admission)?;
        let arguments = candid_codec::encode_one(
            &CanisterIdRecord {
                canister_id: self.request().target.canister,
            },
            512,
        )
        .map_err(|_| StatusRequestError::Admission(CaptureError::StatusAdmission))?;
        crate::store::capture::authenticate_direct_request(
            self.request().sender,
            admission.request_bytes,
            prepared,
            "canister_status",
            &arguments,
        )
    }
}
