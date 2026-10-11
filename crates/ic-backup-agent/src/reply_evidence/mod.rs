//! One passive reply-evidence framing owner; distinct adapter domains stay distinct.

use ic_agent::RequestId;
use ic_backup::model::artifacts::ArtifactChecksumRecord;

pub(super) fn reply_evidence(
    domain: &[u8],
    authority: &ArtifactChecksumRecord,
    attempts: &[u32],
    request_id: &RequestId,
    reply: &[u8],
) -> ArtifactChecksumRecord {
    let mut evidence = domain.to_vec();
    evidence.extend_from_slice(authority.hash().as_bytes());
    for attempt in attempts {
        evidence.extend_from_slice(&attempt.to_le_bytes());
    }
    evidence.extend_from_slice(request_id.as_ref());
    evidence.extend_from_slice(ArtifactChecksumRecord::from_bytes(reply).hash().as_bytes());
    ArtifactChecksumRecord::from_bytes(&evidence)
}

#[cfg(test)]
mod tests;
