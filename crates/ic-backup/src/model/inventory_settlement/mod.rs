//! Passive independently qualified original snapshot-creation claims.

use crate::model::artifacts::ArtifactChecksumRecord;
use std::fmt;

/// Independent original capture or allocation attribution; candidate cardinality supplies no claim.
#[derive(Clone)]
pub enum IcInventoryAttribution {
    /// Exclusive attribution to the exact original capture or allocation, retained through observation.
    Applied {
        /// Exact new raw ID, distinct from every retained baseline ID.
        snapshot_id: Vec<u8>,
        /// Qualified evidence excluding independent creation by other writers.
        attribution: ArtifactChecksumRecord,
    },
    /// The original capture or allocation never applied, including no transient creation then deletion.
    NotApplied {
        /// Qualified exclusion; an empty inventory delta is insufficient.
        exclusion: ArtifactChecksumRecord,
    },
    /// An actually settled authenticated list cannot resolve the original capture or allocation.
    Unresolved {
        /// Settled uncertainty; lost, unavailable or malformed replies stay pending.
        uncertainty: ArtifactChecksumRecord,
    },
}

impl fmt::Debug for IcInventoryAttribution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Applied {
                snapshot_id,
                attribution,
            } => formatter
                .debug_struct("Applied")
                .field("snapshot_id_bytes", &snapshot_id.len())
                .field("attribution", attribution)
                .finish(),
            Self::NotApplied { exclusion } => formatter
                .debug_struct("NotApplied")
                .field("exclusion", exclusion)
                .finish(),
            Self::Unresolved { uncertainty } => formatter
                .debug_struct("Unresolved")
                .field("uncertainty", uncertainty)
                .finish(),
        }
    }
}

/// Passive claims bound to original reservations and independently retained inventories.
///
/// Integrations retain the full original baseline before mutation and authenticate
/// actual context/target, chronology, freshness, permissions, custody and attribution.
/// `evidence` binds all fields and their qualification. No persisted proof flag,
/// spending owner or receipt is introduced.
#[derive(Clone, Debug)]
pub struct IcInventorySettlement {
    /// Complete original plan/operation/immutable allowance digest.
    pub authority: ArtifactChecksumRecord,
    /// Exact original pending capture or allocation attempt.
    pub mutation_attempt: u32,
    /// Exact already consumed pending list observation attempt.
    pub observation_attempt: u32,
    /// Current caller-owned qualification challenge, never a spending allowance.
    pub challenge: ArtifactChecksumRecord,
    /// Existing exact original list request/raw baseline digest.
    pub baseline: ArtifactChecksumRecord,
    /// Existing exact current list request/raw inventory digest.
    pub inventory: ArtifactChecksumRecord,
    /// Exact opaque evidence retained with the passive list response.
    pub observation_evidence: ArtifactChecksumRecord,
    /// Independently qualified outcome, never inferred from candidate count.
    pub attribution: IcInventoryAttribution,
    /// Complete retained integration qualification evidence.
    pub evidence: ArtifactChecksumRecord,
}
