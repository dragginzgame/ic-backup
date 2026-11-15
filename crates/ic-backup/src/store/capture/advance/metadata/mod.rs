//! Exact metadata authentication and one durable metadata acquisition.
use super::{
    CaptureError, CapturePlan, PreparedStatus, StatusAdmissionRecord, StatusCertificateError,
    StatusEvidence, StatusRequestError, certificate, request,
};
use crate::store::TransferMetadataError;
use ic_platform::{
    candid_codec,
    management_canister_types::{
        ReadCanisterSnapshotMetadataArgs, ReadCanisterSnapshotMetadataResult, Snapshot,
    },
};
use thiserror::Error;

const METHOD: &str = "read_canister_snapshot_metadata";

/// Metadata request, authenticated result or transfer-profile refusal.
#[derive(Debug, Error)]
pub enum MetadataReadError {
    /// Snapshot IDs must contain 1 through 256 exact opaque bytes.
    #[error("invalid metadata snapshot identity")]
    SnapshotId,
    /// Original bounds or argument encoding refused.
    #[error(transparent)]
    Admission(#[from] CaptureError),
    /// Original direct-key signature or exact request binding refused.
    #[error(transparent)]
    Request(#[from] StatusRequestError),
    /// Original trust, certificate windows or exact response refused.
    #[error(transparent)]
    Certificate(#[from] StatusCertificateError),
    /// Certified rejection is retained evidence, not usable metadata.
    #[error("metadata request was rejected")]
    Rejected,
    /// The metadata's timestamp differs from the selected original take result.
    #[error("metadata differs from the original snapshot timestamp")]
    SnapshotMismatch,
    /// Bounded decoding or the selected transfer profile refused.
    #[error(transparent)]
    Metadata(#[from] TransferMetadataError),
}

impl CapturePlan {
    /// Encode the fixed metadata-read arguments for this target and exact snapshot ID.
    ///
    /// The integration must authenticate the original take result and preserve its
    /// custody. This pure helper neither proves that provenance nor reserves a call.
    /// Receiver is the management canister; effective routing is this plan's target.
    /// # Errors
    /// Refuses empty/oversized snapshot IDs or arguments exceeding 512 bytes.
    pub fn metadata_read_arguments(
        &self,
        snapshot: &Snapshot,
    ) -> Result<Vec<u8>, MetadataReadError> {
        if !(1..=256).contains(&snapshot.id.len()) {
            return Err(MetadataReadError::SnapshotId);
        }
        candid_codec::encode_one(
            &ReadCanisterSnapshotMetadataArgs {
                canister_id: self.request().target.canister,
                snapshot_id: snapshot.id.clone(),
            },
            512,
        )
        .map_err(|_| MetadataReadError::Admission(CaptureError::StatusAdmission))
    }
}

/// Authenticate and decode metadata for the exact original snapshot under retained timing.
///
/// Checks the complete bounded original request, direct Ed25519/low-S secp256k1
/// signature, sender, management receiver, fixed method, exact target/snapshot
/// arguments, request ID and expiry. Authenticates the certificate under the plan's
/// trust root, effective target and original time windows, then compares the exact
/// retained reply/proof outcome. Existing bounded metadata decoding and transfer
/// policy apply; the source timestamp must equal the selected take's timestamp.
/// Snapshot `total_size` is not a payload-length promise and is not compared here.
///
/// This local helper performs no clock, signer, network or persistence work. It
/// does not establish original take provenance, live permission, finite prior
/// spending or durable schedule custody. Callers retain exact request/reply/proof
/// bytes. Success neither completes a backup nor authorizes another request.
///
/// ```no_run
/// use ic_backup::store::{CapturePlan, PreparedStatus, StatusAdmissionRecord,
///     StatusEvidence, MetadataReadError, authenticate_metadata_reply};
/// use ic_platform::management_canister_types::{Snapshot, ReadCanisterSnapshotMetadataResult};
/// fn inspect(plan: &CapturePlan, snapshot: &Snapshot, admission: &StatusAdmissionRecord,
///     request: &PreparedStatus, evidence: &StatusEvidence, root: &[u8])
///     -> Result<ReadCanisterSnapshotMetadataResult, MetadataReadError> {
///     authenticate_metadata_reply(plan, snapshot, admission, request, evidence, root)
/// }
/// ```
/// # Errors
/// Refuses altered identity/signature, bounds, certificate timing/trust, response,
/// rejection, snapshot timestamp and unsupported metadata or transfer profiles.
pub fn authenticate_metadata_reply(
    plan: &CapturePlan,
    snapshot: &Snapshot,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    evidence: &StatusEvidence,
    root_key_der: &[u8],
) -> Result<ReadCanisterSnapshotMetadataResult, MetadataReadError> {
    let arguments = plan.metadata_read_arguments(snapshot)?;
    request::authenticate_expected_request(plan, admission, prepared, METHOD, &arguments)?;
    evidence.encode(admission.result_bytes)?;
    certificate::authenticate_expected_evidence(
        plan,
        admission,
        prepared,
        evidence,
        root_key_der,
        METHOD,
        &arguments,
    )?;
    if evidence.rejected {
        return Err(MetadataReadError::Rejected);
    }
    let metadata = plan.decode_metadata(&snapshot.id, &evidence.response)?;
    if metadata.taken_at_timestamp != snapshot.taken_at_timestamp {
        return Err(MetadataReadError::SnapshotMismatch);
    }
    Ok(metadata)
}

mod workflow;
pub use workflow::{
    CaptureMetadataAdmissionRecord, CaptureMetadataAdvanceError, CaptureMetadataPhase,
    CaptureMetadataProgress, CaptureMetadataSigner, CaptureMetadataView,
};
