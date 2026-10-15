//! Exact domain-prefixed evidence framing for admitted IC replies.

use crate::model::artifacts::ArtifactChecksumRecord;

pub(crate) fn reply<const N: usize>(
    domain: &[u8],
    checksums: [&ArtifactChecksumRecord; N],
) -> ArtifactChecksumRecord {
    let mut bytes = Vec::with_capacity(domain.len() + N * 64);
    bytes.extend_from_slice(domain);
    for checksum in checksums {
        bytes.extend_from_slice(checksum.hash().as_bytes());
    }
    ArtifactChecksumRecord::from_bytes(&bytes)
}
