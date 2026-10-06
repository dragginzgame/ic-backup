//! Passive integration-qualified attribution of one original data upload.

use crate::model::artifacts::ArtifactChecksumRecord;

/// Integration-owned original-write attribution, never derived from byte equality.
///
/// These are passive claims. The integration authenticates and retains their
/// evidence; constructing a variant grants no receipt or dispatch authority.
#[derive(Clone, Debug)]
pub enum IcSnapshotUploadDataAttribution {
    /// The exact original write applied and remains represented by the exact readback.
    Applied {
        /// Exclusive original-request attribution, excluding preexisting bytes and
        /// independent writers, with stable destination custody through the read.
        attribution: ArtifactChecksumRecord,
    },
    /// Qualified exclusion establishes that this original write never applied.
    NotApplied {
        /// Excludes a transient write followed by overwrite; different bytes or
        /// matching initialized bytes alone cannot establish nonapplication.
        exclusion: ArtifactChecksumRecord,
    },
    /// A settled authenticated read cannot resolve this exact original write.
    Unresolved {
        /// Actual settled uncertainty evidence; unavailable or lost replies stay pending.
        uncertainty: ArtifactChecksumRecord,
    },
}

/// Passive settlement claims for an already accounted successful exact data read.
///
/// No serialized flag, provider, call allowance or progress owner is introduced.
/// Qualification uses retained evidence only; any additional remote call requires
/// independent prior accounting and is outside this local admission operation.
/// The integration binds all fields, actual context/target, original allocation,
/// authentication, chronology and attribution into `evidence`.
#[derive(Clone, Debug)]
pub struct IcSnapshotUploadDataSettlement {
    /// Exact full-plan-derived original operation authority and immutable allowances.
    pub authority: ArtifactChecksumRecord,
    /// Exact original still-pending write attempt.
    pub mutation_attempt: u32,
    /// Exact already reserved still-pending data-read attempt.
    pub observation_attempt: u32,
    /// Caller-owned fresh qualification challenge; equality alone proves no freshness.
    pub challenge: ArtifactChecksumRecord,
    /// Existing data-reply digest of exact destination metadata, read request and raw bytes.
    pub readback: ArtifactChecksumRecord,
    /// Exact opaque authenticated observation evidence retained with that response.
    pub observation_evidence: ArtifactChecksumRecord,
    /// Separately qualified write attribution/nonapplication/settled uncertainty.
    pub attribution: IcSnapshotUploadDataAttribution,
    /// Opaque retained evidence binding every settlement field and original context.
    pub evidence: ArtifactChecksumRecord,
}
