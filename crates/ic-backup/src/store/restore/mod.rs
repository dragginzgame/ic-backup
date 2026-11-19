//! Source-bound same-canister restore through the common direct recorder.

pub(in crate::store) mod persistence;
pub(in crate::store) mod procedure;
#[cfg(test)]
pub(in crate::store::restore) mod tests;

use super::{
    BackupTarget, CaptureSource, OperationLimits, RestoreTransfer, RestoreTransferError,
    RestoreTransferRequest, manifest::lower_hex,
};
use crate::store::{
    BackupDigest, OperationStoreError,
    operation::{
        Journal,
        custody::Recorder,
        intent::{Error as IntentError, Intent, Plan, ProfileRecord},
    },
};
use ic_platform_types::Principal;
pub use persistence::{RestoreOperation, RestorePersistenceError};
pub use procedure::Error as RestorePolicyError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const PLAN_LIMIT: usize = 1024 * 1024;
const FORMAT: &str = "ic-backup/restore-intent/1";
const DOMAIN: &[u8] = b"ic-backup/restore-intent/1\0";

/// Exact same-target/release direct-host restore selection; not live authority.
#[derive(Clone, Debug)]
pub struct RestoreRequest {
    /// Unique 32-character lowercase hexadecimal ID, distinct from the source capture.
    pub operation_id: String,
    /// Must equal the authenticated source network and existing canister.
    pub target: BackupTarget,
    /// Must equal the original provisioned capture trust binding.
    pub trust_root: BackupDigest,
    /// Approving initiator, independent of the management sender.
    pub initiator: Principal,
    /// Must equal the authenticated source release.
    pub release: BackupDigest,
    /// Explicit direct-host management sender; may differ from the capture sender.
    pub sender: Principal,
    /// Independent finite restore limits, never reused capture spending.
    pub limits: OperationLimits,
}

/// Immutable restore approval bound to one authenticated source and direct profile.
#[derive(Clone, Debug)]
pub struct RestorePlan {
    request: RestoreRequest,
    source: SourceRecord,
    profile: ProfileRecord,
    bytes: Vec<u8>,
    digest: BackupDigest,
}
#[derive(Clone, Debug)]
struct SourceRecord {
    operation_id: String,
    capture_plan: BackupDigest,
    manifest: BackupDigest,
    module: BackupDigest,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanRecord {
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
    limits: OperationLimits,
    profile: ProfileRecord,
}
/// Restore declaration/profile/source refusal; no failure grants new effect authority.
#[derive(Debug, Error)]
pub enum RestorePlanError {
    /// Invalid identity, schema or finite limits.
    #[error("invalid restore intent")]
    InvalidPlan,
    /// Selected source differs from the authenticated original.
    #[error("restore source binding refused")]
    SourceBinding,
    /// Source geometry or finite base schedule refused.
    #[error(transparent)]
    Transfer(#[from] RestoreTransferError),
    /// Known restore calls exceed original signing allowance.
    #[error("restore requires {required} signatures, exceeding {limit}")]
    SigningBudget {
        /// Known complete restore count.
        required: u32,
        /// Original finite ceiling.
        limit: u32,
    },
    /// Strict original record refused.
    #[error("invalid restore intent encoding")]
    Record(#[from] serde_json::Error),
    /// Original direct profile refused.
    #[error(transparent)]
    Profile(#[from] IntentError),
    /// Known metadata/payload cannot fit the approved complete signed-envelope ceiling.
    #[error("restore arguments exceed original request capacity")]
    RequestCapacity,
}
impl RestorePlan {
    pub(in crate::store) fn new(
        source: &CaptureSource<'_, '_>,
        request: RestoreRequest,
        profile: ProfileRecord,
    ) -> Result<Self, RestorePlanError> {
        let original = source.original_plan();
        let summary = source.summary();
        let plan = Self::from_parts(
            request,
            SourceRecord {
                operation_id: summary.bundle_id.clone(),
                capture_plan: original.digest(),
                manifest: summary.manifest_sha256,
                module: summary.module_sha256,
            },
            profile,
        )?;
        plan.validate_source(source)?;
        Ok(plan)
    }

    fn from_parts(
        request: RestoreRequest,
        source: SourceRecord,
        profile: ProfileRecord,
    ) -> Result<Self, RestorePlanError> {
        if !lower_hex(&request.operation_id, 32)
            || !lower_hex(&source.operation_id, 32)
            || request.operation_id == source.operation_id
            || !request.limits.valid()
        {
            return Err(RestorePlanError::InvalidPlan);
        }
        let record = PlanRecord {
            format: FORMAT.into(),
            operation_id: request.operation_id.clone(),
            network: request.target.network.to_string(),
            canister: request.target.canister.to_text(),
            trust_root: request.trust_root.to_string(),
            initiator: request.initiator.to_text(),
            release: request.release.to_string(),
            module_sha256: source.module.to_string(),
            sender: request.sender.to_text(),
            source_operation_id: source.operation_id.clone(),
            source_plan_sha256: source.capture_plan.to_string(),
            source_manifest_sha256: source.manifest.to_string(),
            limits: request.limits,
            profile: profile.clone(),
        };
        let bytes = serde_json::to_vec(&record)?;
        if bytes.len() > PLAN_LIMIT {
            return Err(RestorePlanError::InvalidPlan);
        }
        let digest = BackupDigest::compute(&[DOMAIN, &bytes].concat());
        let plan = Self {
            request,
            source,
            profile,
            bytes,
            digest,
        };
        plan.view().validate()?;
        Ok(plan)
    }

    pub(in crate::store) fn decode(bytes: &[u8]) -> Result<Self, RestorePlanError> {
        if bytes.len() > PLAN_LIMIT {
            return Err(RestorePlanError::InvalidPlan);
        }
        let r: PlanRecord = serde_json::from_slice(bytes)?;
        if r.format != FORMAT {
            return Err(RestorePlanError::InvalidPlan);
        }
        let plan = Self::from_parts(
            RestoreRequest {
                operation_id: r.operation_id,
                target: BackupTarget {
                    network: hash(&r.network)?,
                    canister: principal(&r.canister)?,
                },
                trust_root: hash(&r.trust_root)?,
                initiator: principal(&r.initiator)?,
                release: hash(&r.release)?,
                sender: principal(&r.sender)?,
                limits: r.limits,
            },
            SourceRecord {
                operation_id: r.source_operation_id,
                capture_plan: hash(&r.source_plan_sha256)?,
                manifest: hash(&r.source_manifest_sha256)?,
                module: hash(&r.module_sha256)?,
            },
            r.profile,
        )?;
        if plan.bytes != bytes {
            return Err(RestorePlanError::InvalidPlan);
        }
        Ok(plan)
    }

    pub(in crate::store) fn validate_source(
        &self,
        source: &CaptureSource<'_, '_>,
    ) -> Result<RestoreTransfer, RestorePlanError> {
        let summary = source.summary();
        let original = source.original_plan();
        if summary.bundle_id != self.source.operation_id
            || summary.manifest_sha256 != self.source.manifest
            || summary.module_sha256 != self.source.module
            || original.digest() != self.source.capture_plan
            || original.request().trust_root != self.request.trust_root
            || summary.target != self.request.target
            || summary.release != self.request.release
        {
            return Err(RestorePlanError::SourceBinding);
        }
        let transfer = source.plan_restore_transfer(&self.transfer_request())?;
        if transfer.minimum_management_calls() > self.request.limits.signing_requests {
            return Err(RestorePlanError::SigningBudget {
                required: transfer.minimum_management_calls(),
                limit: self.request.limits.signing_requests,
            });
        }
        let room = self.profile.request_bytes.saturating_sub(4096) as u64;
        if source
            .maximum_member_bytes()
            .min(u64::from(self.request.limits.extent_bytes))
            > room
            || procedure::upload_metadata_arguments(
                self.request.target.canister,
                source.metadata(),
                room as usize,
            )
            .is_err()
        {
            return Err(RestorePlanError::RequestCapacity);
        }
        Ok(transfer)
    }

    pub(in crate::store) fn transfer_request(&self) -> RestoreTransferRequest {
        RestoreTransferRequest {
            target: self.request.target,
            release: self.request.release,
            extent_bytes: self.request.limits.extent_bytes,
            maximum_payload_bytes: self.request.limits.payload_bytes,
            maximum_management_calls: self.request.limits.network_requests,
        }
    }

    /// Exact original declaration; editing a clone grants no authority.
    #[must_use]
    pub fn request(&self) -> &RestoreRequest {
        &self.request
    }

    /// Original capture operation selected as source.
    #[must_use]
    pub fn source_operation_id(&self) -> &str {
        &self.source.operation_id
    }

    /// Exact original capture approval.
    #[must_use]
    pub fn source_plan_sha256(&self) -> BackupDigest {
        self.source.capture_plan
    }

    /// Exact trusted source manifest.
    #[must_use]
    pub fn source_manifest_sha256(&self) -> BackupDigest {
        self.source.manifest
    }

    /// Original source module, required before and after load.
    #[must_use]
    pub fn module_sha256(&self) -> BackupDigest {
        self.source.module
    }

    /// Exact canonical original approval bytes.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.bytes
    }

    /// Domain-separated original approval.
    #[must_use]
    pub fn digest(&self) -> BackupDigest {
        self.digest
    }

    pub(in crate::store) fn profile(&self) -> &ProfileRecord {
        &self.profile
    }
}
impl Intent for RestorePlan {
    fn attach_codec(&mut self, codec: Option<&'static dyn crate::store::RootCodec>) {
        self.profile.codec = codec;
    }

    fn view(&self) -> Plan<'_> {
        Plan::Restore(self)
    }

    fn decode(bytes: &[u8]) -> Result<Self, OperationStoreError> {
        Self::decode(bytes).map_err(|_| OperationStoreError::InvalidPlan)
    }

    fn check_layout(record: &Recorder<Self>, _: &Journal) -> Result<(), OperationStoreError> {
        let bundles = crate::store::capture::child(&record.store, "bundles", false)?;
        if bundles
            .open_directory(
                std::ffi::OsStr::new(&record.plan().request().operation_id),
                crate::store::PRIVATE,
            )?
            .is_some()
        {
            return Err(OperationStoreError::Changed);
        }
        Ok(())
    }

    fn published_binding(
        _: &Recorder<Self>,
        journal: &Journal,
    ) -> Result<bool, OperationStoreError> {
        Ok(journal.completed_at_ns.is_some())
    }
}

fn hash(text: &str) -> Result<BackupDigest, RestorePlanError> {
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
