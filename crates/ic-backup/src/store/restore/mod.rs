//! Immutable direct-host restore declarations; no persistence or dispatch authority.
use super::{
    BackupSummary, BackupTarget, CaptureSource, RestoreTransfer, RestoreTransferError,
    RestoreTransferRequest, manifest::lower_hex,
};
use ic_host_artifacts::artifact::{HashingWriter, Sha256Digest};
use ic_platform::{Principal, candid_codec};
use serde::{Deserialize, Serialize};
use std::io::{Write, sink};
use thiserror::Error;

const PLAN_LIMIT: usize = 1024 * 1024;
const FORMAT: &str = "ic-backup/local-restore-plan/1";
const DOMAIN: &[u8] = b"ic-backup/local-restore-plan/1\0";

/// Finite original restore ceilings; no defaults or inferred provider allowances.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreLimitsRecord {
    /// Maximum actual payload bytes, 1..=1 GiB.
    pub payload_bytes: u64,
    /// Region/whole-chunk ceiling, 1..=1 MiB and no greater than payload allowance.
    pub extent_bytes: u32,
    /// Aggregate network allowance including proof traffic/observations, 1..=8192.
    pub network_requests: u32,
    /// Observation subset of network allowance, never additional requests.
    pub observations: u32,
    /// Signing allowance, 1..=8192; actual spending requires a durable journal.
    pub signing_requests: u32,
    /// Retained evidence ceiling, 1..=1 GiB; later creation must admit its initial frame.
    pub evidence_bytes: u64,
}

/// Exact same-target/release direct-host restore selection; not live authority.
#[derive(Clone, Debug)]
pub struct RestoreRequest {
    /// Unique 32-character lowercase hexadecimal ID, distinct from the source capture.
    pub operation_id: String,
    /// Must equal the authenticated source network and existing canister.
    pub target: BackupTarget,
    /// Must equal the original provisioned capture trust binding.
    pub trust_root: Sha256Digest,
    /// Approving initiator, independent of the management sender.
    pub initiator: Principal,
    /// Must equal the authenticated source release.
    pub release: Sha256Digest,
    /// Explicit direct-host management sender; may differ from the capture sender.
    pub sender: Principal,
    /// Independent finite restore limits, never reused capture spending.
    pub limits: RestoreLimitsRecord,
}

/// Exact encoded source-bound intent and approval commitment, not an executable restore.
///
/// No owner, source reference, journal or destination snapshot is created. Restore
/// execution still requires original durable custody, provider/application admission
/// and retained allocation/request evidence. Source custody must be re-admitted on use.
#[derive(Debug)]
pub struct RestorePlan {
    request: RestoreRequest,
    transfer: RestoreTransfer,
    bytes: Vec<u8>,
    digest: Sha256Digest,
}

#[derive(Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct RestorePlanRecord {
    format: String,
    operation_id: String,
    network: String,
    canister: String,
    trust_root: String,
    initiator: String,
    release: String,
    module_sha256: String,
    sender: String,
    source_operation_id: String,
    source_plan_sha256: String,
    source_manifest_sha256: String,
    limits: RestoreLimitsRecord,
    data_calls: u32,
    minimum_management_calls: u32,
    source_requests_sha256: String,
}

/// Restore declaration/refusal changes no file, effect or allowance.
#[derive(Debug, Error)]
pub enum RestorePlanError {
    /// Invalid operation ID, format, canonical identity or finite ceiling.
    #[error("invalid restore plan declaration")]
    InvalidPlan,
    /// Trust, source receipt, schedule or derived fields differ from the admitted source.
    #[error("restore plan differs from its original source binding")]
    SourceBinding,
    /// Source schedule, same-target/release selection or minimum network allowance refused.
    #[error(transparent)]
    Transfer(#[from] RestoreTransferError),
    /// Base restore calls cannot fit the selected signing allowance.
    #[error("restore requires at least {required} signatures, exceeding {limit}")]
    SigningBudget {
        /// Fixed lifecycle plus complete upload/readback count.
        required: u32,
        /// Original signing allowance.
        limit: u32,
    },
    /// Strict record decode/encode failed.
    #[error("invalid restore plan record")]
    Record(#[from] serde_json::Error),
    /// Bounded source-request encoding/commitment failed.
    #[error("restore source request encoding refused")]
    Encoding,
}

impl RestorePlan {
    /// Bind a direct restore declaration to the authenticated source and exact schedule.
    ///
    /// Uses retained source facts only; no IO, provider, clock or persistence. The
    /// source manifest commits original metadata/payload; ordered Platform source
    /// request bytes commit the transfer template. Allocation-dependent destination
    /// arguments must be bound later in the original journal before submission.
    /// # Errors
    /// Refuses wrong identity/source, invalid limits, inactive source or insufficient
    /// base network/signing budgets. Provider overhead requires separate admission.
    pub fn new(
        source: &CaptureSource<'_, '_>,
        request: RestoreRequest,
    ) -> Result<Self, RestorePlanError> {
        validate(&request, &source.summary().bundle_id)?;
        if request.trust_root != source.original_plan().request().trust_root {
            return Err(RestorePlanError::SourceBinding);
        }
        let transfer = source.plan_restore_transfer(&RestoreTransferRequest {
            target: request.target,
            release: request.release,
            extent_bytes: request.limits.extent_bytes,
            maximum_payload_bytes: request.limits.payload_bytes,
            maximum_management_calls: request.limits.network_requests,
        })?;
        if transfer.minimum_management_calls() > request.limits.signing_requests {
            return Err(RestorePlanError::SigningBudget {
                required: transfer.minimum_management_calls(),
                limit: request.limits.signing_requests,
            });
        }
        let record = RestorePlanRecord {
            format: FORMAT.into(),
            operation_id: request.operation_id.clone(),
            network: request.target.network.to_string(),
            canister: request.target.canister.to_text(),
            trust_root: request.trust_root.to_string(),
            initiator: request.initiator.to_text(),
            release: request.release.to_string(),
            module_sha256: source.summary().module_sha256.to_string(),
            sender: request.sender.to_text(),
            source_operation_id: source.summary().bundle_id.clone(),
            source_plan_sha256: source.original_plan().digest().to_string(),
            source_manifest_sha256: source.summary().manifest_sha256.to_string(),
            limits: request.limits,
            data_calls: transfer.data_calls(),
            minimum_management_calls: transfer.minimum_management_calls(),
            source_requests_sha256: schedule_digest(&transfer)?.to_string(),
        };
        let bytes = serde_json::to_vec(&record)?;
        if bytes.len() > PLAN_LIMIT {
            return Err(RestorePlanError::InvalidPlan);
        }
        let digest = plan_digest(&bytes);
        Ok(Self {
            request,
            transfer,
            bytes,
            digest,
        })
    }

    /// Re-admit bounded original bytes against a currently authenticated held source.
    ///
    /// Unknown/duplicate fields and noncanonical identities refuse. Valid original
    /// JSON whitespace/order is retained exactly and remains part of the approval
    /// digest; decoding never substitutes regenerated bytes or grants authority.
    /// # Errors
    /// Refuses malformed/oversized records, invalid declarations and changed source,
    /// counts or request commitments. No source reads, writes or provider calls occur.
    pub fn decode(
        source: &CaptureSource<'_, '_>,
        bytes: Vec<u8>,
    ) -> Result<Self, RestorePlanError> {
        if bytes.len() > PLAN_LIMIT {
            return Err(RestorePlanError::InvalidPlan);
        }
        let record: RestorePlanRecord = serde_json::from_slice(&bytes)?;
        if record.format != FORMAT {
            return Err(RestorePlanError::InvalidPlan);
        }
        let request = RestoreRequest {
            operation_id: record.operation_id.clone(),
            target: BackupTarget {
                network: hash(&record.network)?,
                canister: principal(&record.canister)?,
            },
            trust_root: hash(&record.trust_root)?,
            initiator: principal(&record.initiator)?,
            release: hash(&record.release)?,
            sender: principal(&record.sender)?,
            limits: record.limits,
        };
        let mut plan = Self::new(source, request)?;
        let expected: RestorePlanRecord = serde_json::from_slice(plan.encoded())?;
        if record != expected {
            return Err(RestorePlanError::SourceBinding);
        }
        plan.digest = plan_digest(&bytes);
        plan.bytes = bytes;
        Ok(plan)
    }
    /// Original direct-host declaration; a changed clone cannot alter this plan.
    #[must_use]
    pub fn request(&self) -> &RestoreRequest {
        &self.request
    }
    /// Exact authenticated source receipt required by this plan.
    #[must_use]
    pub fn source(&self) -> &BackupSummary {
        self.transfer.source()
    }
    /// Original fixed-order transfer calculation; not upload/load authority.
    #[must_use]
    pub fn transfer(&self) -> &RestoreTransfer {
        &self.transfer
    }
    /// Exact original bytes used by the approval commitment.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.bytes
    }
    /// Domain-separated commitment to every original plan byte.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        self.digest
    }
}

fn validate(r: &RestoreRequest, source_id: &str) -> Result<(), RestorePlanError> {
    let b = r.limits;
    if !lower_hex(&r.operation_id, 32)
        || r.operation_id == source_id
        || !(1..=1_073_741_824).contains(&b.payload_bytes)
        || !(1..=1_048_576).contains(&b.extent_bytes)
        || u64::from(b.extent_bytes) > b.payload_bytes
        || !(1..=8192).contains(&b.network_requests)
        || b.observations > b.network_requests
        || !(1..=8192).contains(&b.signing_requests)
        || !(1..=1_073_741_824).contains(&b.evidence_bytes)
    {
        return Err(RestorePlanError::InvalidPlan);
    }
    Ok(())
}
fn hash(text: &str) -> Result<Sha256Digest, RestorePlanError> {
    if !lower_hex(text, 64) {
        return Err(RestorePlanError::InvalidPlan);
    }
    text.parse().map_err(|_| RestorePlanError::InvalidPlan)
}
fn principal(text: &str) -> Result<Principal, RestorePlanError> {
    let p = Principal::from_text(text).map_err(|_| RestorePlanError::InvalidPlan)?;
    if p.to_text() != text {
        return Err(RestorePlanError::InvalidPlan);
    }
    Ok(p)
}
fn plan_digest(bytes: &[u8]) -> Sha256Digest {
    Sha256Digest::compute(&[DOMAIN, bytes].concat())
}
fn schedule_digest(transfer: &RestoreTransfer) -> Result<Sha256Digest, RestorePlanError> {
    let mut writer = HashingWriter::new(sink(), 1024 * 1024);
    writer
        .write_all(b"ic-backup/restore-source-requests/1\0")
        .map_err(|_| RestorePlanError::Encoding)?;
    for (ordinal, request) in transfer.source_requests().enumerate() {
        let bytes =
            candid_codec::encode_one(&request, 4096).map_err(|_| RestorePlanError::Encoding)?;
        writer
            .write_all(&(ordinal as u32).to_le_bytes())
            .map_err(|_| RestorePlanError::Encoding)?;
        writer
            .write_all(&(bytes.len() as u32).to_le_bytes())
            .map_err(|_| RestorePlanError::Encoding)?;
        writer
            .write_all(Sha256Digest::compute(&bytes).as_bytes())
            .map_err(|_| RestorePlanError::Encoding)?;
    }
    Ok(writer.identity().sha256)
}

#[cfg(test)]
mod tests;
