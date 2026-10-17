//! One passive reply-evidence framing owner; distinct adapter domains stay distinct.

use ic_agent::RequestId;
use ic_backup::model::artifacts::ArtifactChecksumRecord;
use ic_host_artifacts::artifact::HashingWriter;
use std::io::{self, Write};

pub(super) fn reply_evidence(
    domain: &[u8],
    authority: &ArtifactChecksumRecord,
    attempts: &[u32],
    request_id: &RequestId,
    reply: &[u8],
) -> ArtifactChecksumRecord {
    let mut evidence = HashingWriter::new(io::sink(), u64::MAX);
    let mut append = |bytes: &[u8]| {
        evidence
            .write_all(bytes)
            .expect("sink-backed reply digest writes cannot fail");
    };
    append(domain);
    append(authority.hash().as_bytes());
    for attempt in attempts {
        append(&attempt.to_le_bytes());
    }
    append(request_id.as_ref());
    append(ArtifactChecksumRecord::from_bytes(reply).hash().as_bytes());
    let (_, identity) = evidence.into_parts();
    ArtifactChecksumRecord::from_digest(*identity.sha256.as_bytes())
}

#[cfg(test)]
mod tests;
