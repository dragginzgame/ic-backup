//! Exact scheduled data ingress and certified reply admission; no dispatch or progress.
use super::{CaptureTransfer, TransferDataError};
use crate::store::capture::{
    CaptureError, PreparedStatus, StatusAdmissionRecord, StatusCertificateError, StatusEvidence,
    StatusRequestError,
    advance::{authenticate_expected_evidence, authenticate_expected_request},
};
use ic_platform::{candid_codec, management_canister_types::ReadCanisterSnapshotDataArgs};
use thiserror::Error;

const METHOD: &str = "read_canister_snapshot_data";

/// Signed request, certified evidence or scheduled payload refusal.
#[derive(Debug, Error)]
pub enum DataReadError {
    /// Original request/result bounds or fixed argument encoding refused.
    #[error(transparent)]
    Admission(#[from] CaptureError),
    /// Exact request identity or direct-key signature refused.
    #[error(transparent)]
    Request(#[from] StatusRequestError),
    /// Original root, time policy or exact certified response refused.
    #[error(transparent)]
    Certificate(#[from] StatusCertificateError),
    /// Certified rejection is evidence, not snapshot payload.
    #[error("snapshot data request was rejected")]
    Rejected,
    /// Ordinal, bounded Candid decoding, region length or chunk hash refused.
    #[error(transparent)]
    Data(#[from] TransferDataError),
}
impl CaptureTransfer<'_> {
    /// Encode one exact scheduled request without iterating preceding extents.
    ///
    /// Uses the plan target, original snapshot ID and zero-based data ordinal.
    /// This pure helper grants no signing or dispatch authority.
    /// # Errors
    /// Refuses an ordinal outside the schedule or arguments exceeding 1 KiB.
    pub fn data_arguments(&self, ordinal: u32) -> Result<Vec<u8>, DataReadError> {
        if ordinal >= self.data_calls() {
            return Err(TransferDataError::Ordinal.into());
        }
        candid_codec::encode_one(
            &ReadCanisterSnapshotDataArgs {
                canister_id: self.plan.request().target.canister,
                snapshot_id: self.snapshot_id.to_vec(),
                kind: self.kind(u64::from(ordinal)),
            },
            1024,
        )
        .map_err(|_| CaptureError::StatusAdmission.into())
    }

    /// Authenticate one certified data reply against the exact scheduled ordinal.
    ///
    /// Verifies the original direct signature, management receiver, method, target,
    /// snapshot, range/hash, sender, expiry and request ID. Checks the original
    /// trust root and retained certificate windows and compares exact response
    /// bytes and rejection flag before bounded decoding and length/hash validation.
    ///
    /// The schedule's source metadata still requires independent authentication
    /// and retained binding. This pure check does not prove reservation, fresh
    /// read permission, custody, persistence or complete coverage. No clock,
    /// signing, HTTP, journal write or retry is performed. Preserve raw evidence.
    /// ```no_run
    /// use ic_backup::store::{CaptureTransfer, StatusAdmissionRecord, PreparedStatus,
    ///     StatusEvidence, DataReadError};
    /// fn check(transfer: &CaptureTransfer<'_>, ordinal: u32,
    ///     admission: &StatusAdmissionRecord, request: &PreparedStatus,
    ///     evidence: &StatusEvidence, root: &[u8]) -> Result<Vec<u8>, DataReadError> {
    ///     transfer.authenticate_data_reply(ordinal, admission, request, evidence, root)
    /// }
    /// ```
    /// # Errors
    /// Refuses wrong request/signature, bounds, root/time/proof/outcome, rejection,
    /// malformed payload, wrong region length or a mismatched whole-chunk hash.
    pub fn authenticate_data_reply(
        &self,
        ordinal: u32,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
        root_key_der: &[u8],
    ) -> Result<Vec<u8>, DataReadError> {
        let arguments = self.data_arguments(ordinal)?;
        authenticate_expected_request(self.plan, admission, prepared, METHOD, &arguments)?;
        evidence.encode(admission.result_bytes)?;
        authenticate_expected_evidence(
            self.plan,
            admission,
            prepared,
            evidence,
            root_key_der,
            METHOD,
            &arguments,
        )?;
        if evidence.rejected {
            return Err(DataReadError::Rejected);
        }
        self.decode_data(ordinal, &evidence.response)
            .map_err(Into::into)
    }
}
