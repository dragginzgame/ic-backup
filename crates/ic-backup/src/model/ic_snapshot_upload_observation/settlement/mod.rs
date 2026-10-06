//! Passive independently qualified original metadata-allocation claims.
use crate::model::artifacts::ArtifactChecksumRecord;
use std::fmt;

/// Independent allocation attribution; inventory cardinality supplies no claim.
#[derive(Clone)]
pub enum IcSnapshotUploadAttribution {
    /// Exclusive original-request attribution, stable through the retained list.
    Applied {
        /// Exact newly allocated raw ID; never the original source ID.
        snapshot_id: Vec<u8>,
        /// Qualified attribution excluding independent allocation by other writers.
        attribution: ArtifactChecksumRecord,
    },
    /// Exclusion of original allocation, including transient creation then deletion.
    NotApplied {
        /// Qualified nonapplication evidence; absence alone is insufficient.
        exclusion: ArtifactChecksumRecord,
    },
    /// A settled authenticated successful list cannot resolve original allocation.
    Unresolved {
        /// Settled uncertainty evidence; lost or unavailable replies remain pending.
        uncertainty: ArtifactChecksumRecord,
    },
}

impl fmt::Debug for IcSnapshotUploadAttribution {
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
/// Integrations retain the original baseline before mutation and authenticate actual
/// context/target, chronology, freshness, custody and attribution. `evidence` binds
/// every field and this qualification. No schema, spending owner or receipt is added.
#[derive(Clone, Debug)]
pub struct IcSnapshotUploadSettlement {
    /// Complete original plan/operation/immutable allowances.
    pub authority: ArtifactChecksumRecord,
    /// Exact original pending allocation attempt.
    pub mutation_attempt: u32,
    /// Exact already reserved pending list attempt.
    pub observation_attempt: u32,
    /// Current integration-owned qualification challenge.
    pub challenge: ArtifactChecksumRecord,
    /// Original list request/raw baseline digest; equality proves no chronology.
    pub baseline: ArtifactChecksumRecord,
    /// Current exact list request/raw inventory digest.
    pub inventory: ArtifactChecksumRecord,
    /// Exact retained opaque observation evidence.
    pub observation_evidence: ArtifactChecksumRecord,
    /// Independently qualified outcome, never inferred from candidate count.
    pub attribution: IcSnapshotUploadAttribution,
    /// Complete retained integration qualification evidence.
    pub evidence: ArtifactChecksumRecord,
}
