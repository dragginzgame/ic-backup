//! Immutable capture intent; declaration admission grants no effect authority.

use super::CaptureError;
use crate::store::{BackupTarget, manifest::lower_hex};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::Principal;
use serde::{Deserialize, Serialize};

pub(super) const PLAN_LIMIT: usize = 1024 * 1024;
const DOMAIN: &[u8] = b"ic-backup/local-capture-plan/1\0";

/// Immutable finite ceilings. No defaults or inferred provider allowances.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureLimitsRecord {
    /// Maximum payload bytes, from 1 through 1 GiB.
    pub payload_bytes: u64,
    /// Maximum extent/whole-chunk bytes, from 1 through 1 MiB.
    pub extent_bytes: u32,
    /// Total network requests across management, authentication and observations; 1..=8192.
    pub network_requests: u32,
    /// Observation subset of the network allowance, never extra headroom.
    pub observations: u32,
    /// Maximum signing preparations, 1..=8192; lost signing output still consumes a slot.
    pub signing_requests: u32,
    /// Maximum retained operation evidence bytes, from 1 through 1 GiB.
    pub evidence_bytes: u64,
}

/// Exact management sender and route declaration, not qualification of that route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureExecution {
    /// The host submits directly as this sender.
    Direct {
        /// Management sender, distinct from the approving initiator.
        sender: Principal,
    },
    /// A Canic-owned executor submits management calls under its own authority.
    Canic {
        /// Exact executor canister.
        executor: Principal,
        /// Commitment to the executor's durable receipt protocol.
        protocol: Sha256Digest,
        /// Executor authority generation; cannot change during continuation.
        generation: u64,
    },
}

/// Passive input for a single stopped-canister capture plan.
#[derive(Clone, Debug)]
pub struct CaptureRequest {
    /// Caller-selected unique operation ID: exactly 32 lowercase hexadecimal characters.
    pub operation_id: String,
    /// Exact network/canister selection.
    pub target: BackupTarget,
    /// Provisioned network trust-root commitment, independent of routing URLs.
    pub trust_root: Sha256Digest,
    /// Approving operator identity.
    pub initiator: Principal,
    /// Integration-owned release commitment.
    pub release: Sha256Digest,
    /// Expected installed module SHA-256.
    pub module_sha256: Sha256Digest,
    /// Original sender/route declaration.
    pub execution: CaptureExecution,
    /// Explicit original finite ceilings.
    pub limits: CaptureLimitsRecord,
}

/// Validated capture declaration retaining its exact encoding and approval digest.
///
/// It supplies no controller, application, signer or transport authority. Provider
/// cost admission and durable schedule binding belong to the operation workflow.
/// [`Self::plan_transfer`] supplies pure metadata-dependent scheduling only.
#[derive(Debug)]
pub struct CapturePlan {
    request: CaptureRequest,
    bytes: Vec<u8>,
    digest: Sha256Digest,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CapturePlanRecord {
    format: String,
    operation_id: String,
    network: String,
    canister: String,
    trust_root: String,
    initiator: String,
    release: String,
    module_sha256: String,
    execution: ExecutionRecord,
    limits: CaptureLimitsRecord,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ExecutionRecord {
    Direct {
        sender: String,
    },
    Canic {
        executor: String,
        protocol: String,
        generation: u64,
    },
}

impl CapturePlan {
    /// Validate and encode exact capture intent without IO or providers.
    ///
    /// # Errors
    /// Refuses malformed IDs and inconsistent/out-of-range ceilings.
    pub fn new(request: CaptureRequest) -> Result<Self, CaptureError> {
        validate(&request)?;
        let execution = match &request.execution {
            CaptureExecution::Direct { sender } => ExecutionRecord::Direct {
                sender: sender.to_text(),
            },
            CaptureExecution::Canic {
                executor,
                protocol,
                generation,
            } => ExecutionRecord::Canic {
                executor: executor.to_text(),
                protocol: protocol.to_string(),
                generation: *generation,
            },
        };
        let record = CapturePlanRecord {
            format: "ic-backup/local-capture-plan/1".into(),
            operation_id: request.operation_id.clone(),
            network: request.target.network.to_string(),
            canister: request.target.canister.to_text(),
            trust_root: request.trust_root.to_string(),
            initiator: request.initiator.to_text(),
            release: request.release.to_string(),
            module_sha256: request.module_sha256.to_string(),
            execution,
            limits: request.limits,
        };
        let bytes = serde_json::to_vec(&record)?;
        Self::admitted(request, bytes)
    }

    /// Read the immutable declaration; changing a clone grants no continuation authority.
    #[must_use]
    pub const fn request(&self) -> &CaptureRequest {
        &self.request
    }

    /// Original approval commitment, including every identity and limit.
    #[must_use]
    pub const fn digest(&self) -> Sha256Digest {
        self.digest
    }

    /// Exact retained plan bytes, never reconstructed on reopen.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.bytes
    }

    pub(super) fn decode(bytes: Vec<u8>) -> Result<Self, CaptureError> {
        if bytes.len() > PLAN_LIMIT {
            return Err(CaptureError::InvalidPlan);
        }
        let r: CapturePlanRecord = serde_json::from_slice(&bytes)?;
        if r.format != "ic-backup/local-capture-plan/1" {
            return Err(CaptureError::InvalidPlan);
        }
        let execution = match r.execution {
            ExecutionRecord::Direct { sender } => CaptureExecution::Direct {
                sender: principal(&sender)?,
            },
            ExecutionRecord::Canic {
                executor,
                protocol,
                generation,
            } => CaptureExecution::Canic {
                executor: principal(&executor)?,
                protocol: hash(&protocol)?,
                generation,
            },
        };
        let request = CaptureRequest {
            operation_id: r.operation_id,
            target: BackupTarget {
                network: hash(&r.network)?,
                canister: principal(&r.canister)?,
            },
            trust_root: hash(&r.trust_root)?,
            initiator: principal(&r.initiator)?,
            release: hash(&r.release)?,
            module_sha256: hash(&r.module_sha256)?,
            execution,
            limits: r.limits,
        };
        Self::admitted(request, bytes)
    }

    fn admitted(request: CaptureRequest, bytes: Vec<u8>) -> Result<Self, CaptureError> {
        validate(&request)?;
        if bytes.len() > PLAN_LIMIT {
            return Err(CaptureError::InvalidPlan);
        }
        let digest = Sha256Digest::compute(&[DOMAIN, &bytes].concat());
        Ok(Self {
            request,
            bytes,
            digest,
        })
    }
}

fn validate(request: &CaptureRequest) -> Result<(), CaptureError> {
    let b = request.limits;
    if !lower_hex(&request.operation_id, 32)
        || !(1..=1024 * 1024 * 1024).contains(&b.payload_bytes)
        || !(1..=1024 * 1024).contains(&b.extent_bytes)
        || u64::from(b.extent_bytes) > b.payload_bytes
        || !(1..=8192).contains(&b.network_requests)
        || b.observations > b.network_requests
        || !(1..=8192).contains(&b.signing_requests)
        || !(1..=1024 * 1024 * 1024).contains(&b.evidence_bytes)
    {
        return Err(CaptureError::InvalidPlan);
    }
    Ok(())
}

fn principal(text: &str) -> Result<Principal, CaptureError> {
    let p = Principal::from_text(text).map_err(|_| CaptureError::InvalidPlan)?;
    if p.to_text() != text {
        return Err(CaptureError::InvalidPlan);
    }
    Ok(p)
}
fn hash(text: &str) -> Result<Sha256Digest, CaptureError> {
    if !lower_hex(text, 64) {
        return Err(CaptureError::InvalidPlan);
    }
    text.parse().map_err(|_| CaptureError::InvalidPlan)
}
