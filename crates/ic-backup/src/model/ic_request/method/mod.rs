//! Closed method identities and semantic effect classes; no arbitrary method hook.

use serde::{Deserialize, Serialize};

/// Maintained closed IC management methods for the initial host-ingress codec.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IcManagementMethodRecord {
    /// Replicated lifecycle/controller/status observation.
    CanisterStatus,
    /// Replicated snapshot inventory observation, with distinct read permission.
    ListCanisterSnapshots,
    /// Load exact snapshot bytes into the same selected existing canister.
    LoadCanisterSnapshot,
    /// Start the selected existing canister; reviewed final state stays caller-owned.
    StartCanister,
    /// Stop the selected existing canister; application quiescence stays caller-owned.
    StopCanister,
    /// Create a new snapshot, retaining code and previously retained snapshots.
    TakeCanisterSnapshot,
}
impl IcManagementMethodRecord {
    /// Read the exact protocol method name, without aliases or arbitrary strings.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CanisterStatus => "canister_status",
            Self::ListCanisterSnapshots => "list_canister_snapshots",
            Self::LoadCanisterSnapshot => "load_canister_snapshot",
            Self::StartCanister => "start_canister",
            Self::StopCanister => "stop_canister",
            Self::TakeCanisterSnapshot => "take_canister_snapshot",
        }
    }
    /// Classify state observation versus mutation; both use replicated update ingress.
    #[must_use]
    pub const fn effect(self) -> IcRequestEffect {
        match self {
            Self::CanisterStatus | Self::ListCanisterSnapshots => IcRequestEffect::Observation,
            Self::LoadCanisterSnapshot
            | Self::StartCanister
            | Self::StopCanister
            | Self::TakeCanisterSnapshot => IcRequestEffect::Mutation,
        }
    }
}

/// Semantic attempt class, distinct from transport update/query mode or authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IcRequestEffect {
    /// Reconciliation/state observation, still potentially paid and separately bounded.
    Observation,
    /// Exact snapshot or lifecycle mutation requiring its original attempt allowance.
    Mutation,
}
