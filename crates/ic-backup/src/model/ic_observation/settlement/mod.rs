//! Passive attribution for one original stop/start/load effect.

use crate::model::artifacts::ArtifactChecksumRecord;

/// Independently qualified original lifecycle outcome; status alone supplies no claim.
#[derive(Clone, Debug)]
pub enum IcLifecycleAttribution {
    /// Exclusive attribution to the original stop/start/load request.
    Applied {
        /// Excludes preexisting state and independent effects. For load, qualifies
        /// the exact original snapshot state, not merely status or module hash.
        attribution: ArtifactChecksumRecord,
    },
    /// The original mutation never applied, including no transient application.
    NotApplied {
        /// Qualified exclusion; later opposite lifecycle state is insufficient.
        exclusion: ArtifactChecksumRecord,
    },
    /// An actually settled authenticated status observation cannot resolve the mutation.
    Unresolved {
        /// Settled uncertainty; lost, unavailable or malformed replies stay pending.
        uncertainty: ArtifactChecksumRecord,
    },
}

/// Passive claims for an already reserved exact successful lifecycle observation.
///
/// Integrations authenticate the actual context/target, qualify chronology,
/// permissions, challenge freshness, command custody and original attribution.
/// Evidence binds every field. No serializable proof flag or spending owner exists.
#[derive(Clone, Debug)]
pub struct IcLifecycleSettlement {
    /// Complete original plan/operation/immutable allowance digest.
    pub authority: ArtifactChecksumRecord,
    /// Exact original pending mutation attempt.
    pub mutation_attempt: u32,
    /// Exact already consumed pending status observation attempt.
    pub observation_attempt: u32,
    /// Current caller-owned qualification challenge, never a spending allowance.
    pub challenge: ArtifactChecksumRecord,
    /// Existing lifecycle digest of exact status request and raw reply bytes.
    pub status: ArtifactChecksumRecord,
    /// Exact opaque evidence retained with the passive status response.
    pub observation_evidence: ArtifactChecksumRecord,
    /// Independently qualified original outcome, never inferred from status values.
    pub attribution: IcLifecycleAttribution,
    /// Complete retained qualification evidence for explicit integration settlement.
    pub evidence: ArtifactChecksumRecord,
}
