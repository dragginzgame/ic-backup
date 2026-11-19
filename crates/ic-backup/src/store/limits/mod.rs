//! Shared declaration bounds; capture and restore retain independent approved values.

use serde::{Deserialize, Serialize};

// Complete envelopes/proofs include protocol overhead around a 1 MiB data extent.
pub(in crate::store) const REQUEST_BYTES: u32 = 2 * 1024 * 1024;
pub(in crate::store) const RESULT_BYTES: u32 = 4 * 1024 * 1024;
pub(in crate::store) const CERTIFICATE_BYTES: usize = 2 * 1024 * 1024;

/// Immutable finite operation ceilings. No defaults or inferred allowances.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationLimits {
    /// Maximum payload bytes, from 1 through 1 GiB.
    pub payload_bytes: u64,
    /// Maximum extent/whole-chunk bytes, from 1 through 1 MiB and at most payload.
    pub extent_bytes: u32,
    /// Total management, authentication and observation network allowance; 1..=8192.
    pub network_requests: u32,
    /// Observation subset of network allowance, never extra headroom.
    pub observations: u32,
    /// Maximum signing invocations, 1..=8192; lost output still consumes a slot.
    pub signing_requests: u32,
    /// Maximum retained operation evidence bytes, from 1 through 1 GiB.
    pub evidence_bytes: u64,
}

impl OperationLimits {
    pub(super) fn valid(self) -> bool {
        (1..=1024 * 1024 * 1024).contains(&self.payload_bytes)
            && (1..=1024 * 1024).contains(&self.extent_bytes)
            && u64::from(self.extent_bytes) <= self.payload_bytes
            && (1..=8192).contains(&self.network_requests)
            && self.observations <= self.network_requests
            && (1..=8192).contains(&self.signing_requests)
            && (1..=1024 * 1024 * 1024).contains(&self.evidence_bytes)
    }
}
