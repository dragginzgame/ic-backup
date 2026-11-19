//! Immutable direct capture declaration and private bounded policy.

use super::{OperationLimits, OperationStoreError};
use crate::store::{BackupDigest, BackupTarget, manifest::lower_hex};
use ic_platform_types::Principal;

pub(in crate::store) const PLAN_LIMIT: usize = 1024 * 1024;
/// Passive input for a single stopped-canister capture plan.
#[derive(Clone, Debug)]
pub struct CaptureRequest {
    /// Caller-selected unique operation ID: exactly 32 lowercase hexadecimal characters.
    pub operation_id: String,
    /// Exact network/canister selection.
    pub target: BackupTarget,
    /// Provisioned network trust-root commitment, independent of routing URLs.
    pub trust_root: BackupDigest,
    /// Approving operator identity.
    pub initiator: Principal,
    /// Integration-owned release commitment.
    pub release: BackupDigest,
    /// Expected installed module SHA-256.
    pub module_sha256: BackupDigest,
    /// Actual direct-host management sender. Canic execution requires a separate qualified adapter.
    pub sender: Principal,
    /// Explicit original finite ceilings.
    pub limits: OperationLimits,
}

#[derive(Clone, Debug)]
pub(in crate::store) struct CapturePolicy {
    request: CaptureRequest,
}
impl CapturePolicy {
    pub(in crate::store) fn new(request: CaptureRequest) -> Result<Self, OperationStoreError> {
        validate(&request)?;
        Ok(Self { request })
    }

    pub(in crate::store) fn request(&self) -> &CaptureRequest {
        &self.request
    }
    // A literal native fixture input, not a production store/approval reader.
    #[cfg(test)]
    pub(in crate::store) fn decode(bytes: &[u8]) -> Result<Self, OperationStoreError> {
        let r: serde_json::Value = serde_json::from_slice(bytes)?;
        let text = |key: &str| r[key].as_str().ok_or(OperationStoreError::InvalidPlan);
        Self::new(CaptureRequest {
            operation_id: text("operation_id")?.into(),
            target: BackupTarget {
                network: hash(text("network")?)?,
                canister: principal(text("canister")?)?,
            },
            trust_root: hash(text("trust_root")?)?,
            initiator: principal(text("initiator")?)?,
            release: hash(text("release")?)?,
            module_sha256: hash(text("module_sha256")?)?,
            sender: principal(
                r["execution"]["sender"]
                    .as_str()
                    .ok_or(OperationStoreError::InvalidPlan)?,
            )?,
            limits: serde_json::from_value(r["limits"].clone())?,
        })
    }
}

fn validate(request: &CaptureRequest) -> Result<(), OperationStoreError> {
    if !lower_hex(&request.operation_id, 32) || !request.limits.valid() {
        return Err(OperationStoreError::InvalidPlan);
    }
    Ok(())
}

pub(super) fn principal(text: &str) -> Result<Principal, OperationStoreError> {
    let p = Principal::from_text(text).map_err(|_| OperationStoreError::InvalidPlan)?;
    if p.to_text() != text {
        return Err(OperationStoreError::InvalidPlan);
    }
    Ok(p)
}

pub(super) fn hash(text: &str) -> Result<BackupDigest, OperationStoreError> {
    if !lower_hex(text, 64) {
        return Err(OperationStoreError::InvalidPlan);
    }
    text.parse().map_err(|_| OperationStoreError::InvalidPlan)
}
