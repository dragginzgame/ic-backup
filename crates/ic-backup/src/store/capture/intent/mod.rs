//! Strict immutable capture intent; direct profile authentication has one shared owner.
use super::{
    CapturePolicy, CaptureRequest, OperationLimits, OperationStoreError,
    plan::{PLAN_LIMIT, hash, principal},
};
pub(in crate::store) use crate::store::operation::intent::Error;
use crate::store::{
    BackupTarget,
    operation::{
        Journal,
        custody::Recorder,
        intent::{Intent, Plan, ProfileRecord},
    },
};
use ic_host_artifacts::artifact::Sha256Digest;
use serde::{Deserialize, Serialize};
const FORMAT: &str = "ic-backup/capture-intent/1";
const DOMAIN: &[u8] = b"ic-backup/capture-intent/1\0";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntentRecord {
    format: String,
    operation_id: String,
    network: String,
    canister: String,
    trust_root: String,
    initiator: String,
    release: String,
    module_sha256: String,
    sender: String,
    limits: OperationLimits,
    profile: ProfileRecord,
}

#[derive(Clone, Debug)]
/// Immutable capture approval bound to one direct connection, trust and finite limits.
pub struct CapturePlan {
    // Reuse the existing pure policy view, not its old encoding or approval digest.
    plan: CapturePolicy,
    profile: ProfileRecord,
    bytes: Vec<u8>,
    digest: Sha256Digest,
}

impl CapturePlan {
    /// Exact original declaration; editing a clone grants no continuation authority.
    #[must_use]
    pub fn request(&self) -> &CaptureRequest {
        self.plan.request()
    }
    pub(in crate::store) fn new(
        request: CaptureRequest,
        profile: ProfileRecord,
    ) -> Result<Self, Error> {
        let plan = CapturePolicy::new(request)?;
        let r = plan.request();
        let sender = r.sender;
        let record = IntentRecord {
            format: FORMAT.into(),
            operation_id: r.operation_id.clone(),
            network: r.target.network.to_string(),
            canister: r.target.canister.to_text(),
            trust_root: r.trust_root.to_string(),
            initiator: r.initiator.to_text(),
            release: r.release.to_string(),
            module_sha256: r.module_sha256.to_string(),
            sender: sender.to_text(),
            limits: r.limits,
            profile: profile.clone(),
        };
        let bytes = serde_json::to_vec(&record)?;
        if bytes.len() > PLAN_LIMIT {
            return Err(Error::Profile);
        }
        let digest = Sha256Digest::compute(&[DOMAIN, &bytes].concat());
        let intent = Self {
            plan,
            profile,
            bytes,
            digest,
        };
        intent.view().validate()?;
        Ok(intent)
    }

    pub(in crate::store) fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > PLAN_LIMIT {
            return Err(Error::Profile);
        }
        let r: IntentRecord = serde_json::from_slice(bytes)?;
        if r.format != FORMAT {
            return Err(Error::Profile);
        }
        let intent = Self::new(
            CaptureRequest {
                operation_id: r.operation_id,
                target: BackupTarget {
                    network: hash(&r.network)?,
                    canister: principal(&r.canister)?,
                },
                trust_root: hash(&r.trust_root)?,
                initiator: principal(&r.initiator)?,
                release: hash(&r.release)?,
                module_sha256: hash(&r.module_sha256)?,
                sender: principal(&r.sender)?,
                limits: r.limits,
            },
            r.profile,
        )?;
        // There is one canonical encoding, no old reader, normalization or renewed approval.
        if intent.bytes != bytes {
            return Err(Error::Profile);
        }
        Ok(intent)
    }

    pub(in crate::store) fn plan(&self) -> &CapturePolicy {
        &self.plan
    }
    pub(in crate::store) fn profile(&self) -> &ProfileRecord {
        &self.profile
    }
    /// Exact canonical approval bytes retained by operation creation.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.bytes
    }
    /// Domain-separated approval commitment to all original identities and limits.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        self.digest
    }
}

impl Intent for CapturePlan {
    fn view(&self) -> Plan<'_> {
        Plan::Capture(self)
    }
    fn decode(bytes: &[u8]) -> Result<Self, OperationStoreError> {
        Self::decode(bytes).map_err(|_| OperationStoreError::InvalidPlan)
    }
    fn check_layout(record: &Recorder<Self>, journal: &Journal) -> Result<(), OperationStoreError> {
        record.check_bundle_layout(journal)
    }
    fn published_binding(
        record: &Recorder<Self>,
        journal: &Journal,
    ) -> Result<bool, OperationStoreError> {
        record.published_binding(journal)
    }
}
#[cfg(test)]
pub(in crate::store) mod tests;
