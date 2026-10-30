//! Closed host-ingress IC request codec; no transport, authority or effect admission.

use crate::model::evidence_digest::EvidenceDigest;
use ic_platform::candid::contract;
mod method;
pub use method::{IcManagementMethodRecord, IcRequestEffect};

use crate::model::{
    artifacts::ArtifactChecksumRecord, attempt_journal::OperationBindingRecord, bounded_sequence,
};
use ic_platform::management_canister_types::{
    CanisterIdRecord, LoadCanisterSnapshotArgs, TakeCanisterSnapshotArgs,
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Maximum raw snapshot identifier bytes, distinct from any backend's rendered token.
pub const MAX_IC_SNAPSHOT_ID_BYTES: usize = 256;
/// Maximum derived Candid argument bytes; checked before a request is admitted.
pub const MAX_IC_ARGUMENT_BYTES: usize = 4096;
/// Bound callers must use for encoded JSON input/output when retaining this record.
pub const MAX_IC_REQUEST_RECORD_BYTES: u64 = 8192;

/// Passive method/target/snapshot input; network, caller and permissions stay with their owners.
#[derive(Clone, Debug)]
pub struct IcManagementRequest {
    /// One of the closed supported management methods.
    pub method: IcManagementMethodRecord,
    /// Exact effective canister principal; normalized on admission.
    pub target: String,
    /// Raw nonempty snapshot bytes required only for load; otherwise exactly None.
    pub snapshot_id: Option<Vec<u8>>,
}

/// Immutable v1 request declaration with model-derived exact Candid bytes.
///
/// All supported methods use replicated update ingress, including observations.
/// Network/caller selection, signing, freshness, budgets, custody and lifecycle
/// safety are not granted by this record or its digest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "RequestFields")]
pub struct IcManagementRequestRecord {
    version: u16,
    method: IcManagementMethodRecord,
    target: String,
    snapshot_id: Option<Vec<u8>>,
    #[serde(skip)]
    target_bytes: Vec<u8>,
    #[serde(skip)]
    arguments: Vec<u8>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestFields {
    version: u16,
    method: IcManagementMethodRecord,
    target: String,
    #[serde(deserialize_with = "required_snapshot")]
    snapshot_id: Option<Vec<u8>>,
}
impl TryFrom<RequestFields> for IcManagementRequestRecord {
    type Error = IcRequestError;
    fn try_from(fields: RequestFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(IcRequestError::UnsupportedVersion(fields.version));
        }
        Self::new(IcManagementRequest {
            method: fields.method,
            target: fields.target,
            snapshot_id: fields.snapshot_id,
        })
    }
}
impl IcManagementRequestRecord {
    /// Validate the closed request and encode exact host-ingress arguments.
    ///
    /// Capture retains code and creates a new snapshot; sender canister version
    /// stays absent for host ingress. Load names exact raw bytes, with no identity remap.
    ///
    /// # Errors
    /// Rejects malformed principals, missing/unexpected/oversized snapshot IDs,
    /// Candid encoding failure and excessive derived argument bytes.
    pub fn new(request: IcManagementRequest) -> Result<Self, IcRequestError> {
        let (target, principal) = crate::model::principal::canonical_parts(&request.target)
            .ok_or(IcRequestError::InvalidTarget)?;
        let arguments = match (request.method, request.snapshot_id.as_deref()) {
            (IcManagementMethodRecord::LoadCanisterSnapshot, None) => {
                return Err(IcRequestError::SnapshotRequired);
            }
            (IcManagementMethodRecord::LoadCanisterSnapshot, Some(bytes)) => {
                if !admits_snapshot_id(bytes) {
                    return Err(IcRequestError::InvalidSnapshotId);
                }
                encode_arguments(LoadCanisterSnapshotArgs {
                    canister_id: principal.into(),
                    snapshot_id: bytes.to_vec(),
                    sender_canister_version: None,
                })
            }
            (_, Some(_)) => return Err(IcRequestError::UnexpectedSnapshot),
            (IcManagementMethodRecord::TakeCanisterSnapshot, None) => {
                encode_arguments(TakeCanisterSnapshotArgs {
                    canister_id: principal.into(),
                    replace_snapshot: None,
                    uninstall_code: Some(false),
                    sender_canister_version: None,
                })
            }
            (
                IcManagementMethodRecord::CanisterStatus
                | IcManagementMethodRecord::ListCanisterSnapshots
                | IcManagementMethodRecord::StartCanister
                | IcManagementMethodRecord::StopCanister,
                None,
            ) => encode_arguments(CanisterIdRecord {
                canister_id: principal.into(),
            }),
        }?;
        Ok(Self {
            version: 1,
            method: request.method,
            target,
            snapshot_id: request.snapshot_id,
            target_bytes: principal.as_slice().to_vec(),
            arguments,
        })
    }
    /// Read the exact closed method; its effect class does not grant spending authority.
    #[must_use]
    pub const fn method(&self) -> IcManagementMethodRecord {
        self.method
    }
    /// Read the canonical effective routing target, also encoded inside the arguments.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }
    /// Read exact raw load snapshot bytes, without backend token interpretation.
    #[must_use]
    pub fn snapshot_id(&self) -> Option<&[u8]> {
        self.snapshot_id.as_deref()
    }
    /// Read derived exact Candid bytes for a future qualified update-ingress transport.
    #[must_use]
    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }
    /// Read the fixed management receiver; the effective routing target is separate.
    #[must_use]
    pub const fn receiver(&self) -> &'static str {
        "aaaaa-aa"
    }
    /// Hash fixed receiver, effective target, update mode, exact method and Candid bytes.
    ///
    /// Network/caller/intent/release/budgets stay outside this nonrecursive payload
    /// digest and are bound by the existing operation plan/attempt authority.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        management_request_digest(&self.target_bytes, self.method.name(), &self.arguments)
    }
    /// Check exact payload target/digest and mutation class against original declared binding.
    ///
    /// This verifies bytes only; it does not authenticate the binding or admit dispatch.
    /// # Errors
    /// Rejects observation methods and changed target or request digest.
    pub fn validate_mutation_binding(
        &self,
        binding: &OperationBindingRecord,
    ) -> Result<(), IcRequestError> {
        self.require_effect(IcRequestEffect::Mutation)?;
        self.validate_identity(binding, binding.request())
    }
    /// Check an observation payload against its exact target and reserved observation digest.
    ///
    /// The original binding still names the mutation. The observation digest is
    /// separately retained by the existing journal's observation reservation.
    /// # Errors
    /// Rejects mutation methods, changed target or a different observation digest.
    pub fn validate_observation_binding(
        &self,
        binding: &OperationBindingRecord,
        request: &ArtifactChecksumRecord,
    ) -> Result<(), IcRequestError> {
        self.require_effect(IcRequestEffect::Observation)?;
        self.validate_identity(binding, request.hash())
    }
    fn require_effect(&self, expected: IcRequestEffect) -> Result<(), IcRequestError> {
        if self.method.effect() != expected {
            return Err(IcRequestError::EffectMismatch { expected });
        }
        Ok(())
    }
    fn validate_identity(
        &self,
        binding: &OperationBindingRecord,
        expected: &str,
    ) -> Result<(), IcRequestError> {
        if self.target != binding.target() {
            return Err(IcRequestError::TargetMismatch);
        }
        if self.digest().hash() != expected {
            return Err(IcRequestError::DigestMismatch);
        }
        Ok(())
    }
}

pub(super) fn encode_arguments(
    value: impl contract::CandidType,
) -> Result<Vec<u8>, IcRequestError> {
    let arguments =
        contract::encode_one(value).map_err(|error| IcRequestError::Encoding(error.to_string()))?;
    if arguments.len() > MAX_IC_ARGUMENT_BYTES {
        return Err(IcRequestError::ArgumentsTooLarge);
    }
    Ok(arguments)
}

// Shared scalar admission; callers retain method-specific absence/source-ID rules
// and JSON/Candid visitors retain their separate bounded decoding behavior.
pub(super) fn admits_snapshot_id(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.len() <= MAX_IC_SNAPSHOT_ID_BYTES
}

// Callers own canonical principal bytes, a fixed ASCII method and bounded arguments.
pub(super) fn management_request_digest(
    target_bytes: &[u8],
    method: &str,
    arguments: &[u8],
) -> ArtifactChecksumRecord {
    let mut bytes = EvidenceDigest::new(b"ic-backup/ic-management-request/v1\0");
    bytes.byte(0); // Management receiver principal has zero raw bytes.
    bytes.bytes_u8(target_bytes);
    bytes.byte(1); // Replicated update ingress.
    bytes.bytes_u8(method.as_bytes());
    bytes.bytes_u32(arguments);
    bytes.finish()
}

fn required_snapshot<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<u8>>, D::Error> {
    Ok(Option::<SnapshotBytes>::deserialize(deserializer)?.map(|bytes| bytes.0))
}
struct SnapshotBytes(Vec<u8>);
impl<'de> Deserialize<'de> for SnapshotBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bounded_sequence::deserialize::<_, u8, MAX_IC_SNAPSHOT_ID_BYTES>(
            deserializer,
            "at most 256 exact snapshot bytes",
            IcRequestError::InvalidSnapshotId,
        )
        .map(Self)
    }
}

/// Typed closed IC request, exact byte binding or codec admission failure.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum IcRequestError {
    /// Other product generations are not maintained.
    #[error("unsupported IC request version {0}")]
    UnsupportedVersion(u16),
    /// Target text is not a bounded canonicalizable principal.
    #[error("invalid IC request target principal")]
    InvalidTarget,
    /// Load requires exact raw snapshot bytes.
    #[error("load snapshot request requires snapshot_id")]
    SnapshotRequired,
    /// A method that has no snapshot argument received one.
    #[error("snapshot_id is not admitted for this IC request method")]
    UnexpectedSnapshot,
    /// Snapshot bytes are empty or exceed their finite bound.
    #[error("snapshot_id must contain 1..={MAX_IC_SNAPSHOT_ID_BYTES} raw bytes")]
    InvalidSnapshotId,
    /// The pinned upstream Candid encoder rejected the typed arguments.
    #[error("IC request Candid encoding failed: {0}")]
    Encoding(String),
    /// Derived argument size exceeds the codec bound.
    #[error("IC request arguments exceed {MAX_IC_ARGUMENT_BYTES} bytes")]
    ArgumentsTooLarge,
    /// A mutation and observation request cannot substitute for each other.
    #[error("IC request must have effect class {expected:?}")]
    EffectMismatch {
        /// Required semantic class, independent of replicated call mode.
        expected: IcRequestEffect,
    },
    /// The payload/routing target differs from the original operation target.
    #[error("IC request target differs from original binding")]
    TargetMismatch,
    /// Exact method/argument digest differs from its original reservation.
    #[error("IC request digest differs from original binding")]
    DigestMismatch,
}

#[cfg(test)]
mod tests;
