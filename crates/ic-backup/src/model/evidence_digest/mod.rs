//! Exact domain-prefixed framing for canonical records and IC evidence.

use crate::model::artifacts::ArtifactChecksumRecord;
use ic_host_artifacts::artifact::HashingWriter;
use std::io::{self, Write};

pub(crate) struct EvidenceDigest {
    writer: HashingWriter<io::Sink>,
}

impl EvidenceDigest {
    pub(crate) fn new(domain: &[u8]) -> Self {
        let mut digest = Self {
            writer: HashingWriter::new(io::sink(), u64::MAX),
        };
        digest.append(domain);
        digest
    }

    pub(crate) fn checksum(&mut self, checksum: &ArtifactChecksumRecord) {
        self.append(checksum.hash().as_bytes());
    }

    pub(crate) fn u64_be(&mut self, value: u64) {
        self.append(&value.to_be_bytes());
    }

    pub(crate) fn u32_be(&mut self, value: u32) {
        self.append(&value.to_be_bytes());
    }

    pub(crate) fn byte(&mut self, value: u8) {
        self.append(&[value]);
    }

    // Callers supply already bounded model fields; the prefix width is part of each v1 contract.
    pub(crate) fn bytes_u8(&mut self, bytes: &[u8]) {
        self.byte(u8::try_from(bytes.len()).expect("admitted digest field fits u8"));
        self.append(bytes);
    }

    pub(crate) fn bytes_u32(&mut self, bytes: &[u8]) {
        self.u32_be(u32::try_from(bytes.len()).expect("admitted digest field fits u32"));
        self.append(bytes);
    }

    pub(crate) fn bytes_u64(&mut self, bytes: &[u8]) {
        self.u64_be(bytes.len() as u64);
        self.append(bytes);
    }

    pub(crate) fn append(&mut self, bytes: &[u8]) {
        self.writer
            .write_all(bytes)
            .expect("sink-backed evidence digest writes cannot fail");
    }

    pub(crate) fn finish(self) -> ArtifactChecksumRecord {
        let (_, identity) = self.writer.into_parts();
        ArtifactChecksumRecord::from_digest(*identity.sha256.as_bytes())
    }
}

pub(crate) fn reply<const N: usize>(
    domain: &[u8],
    checksums: [&ArtifactChecksumRecord; N],
) -> ArtifactChecksumRecord {
    let mut framing = EvidenceDigest::new(domain);
    for checksum in checksums {
        framing.checksum(checksum);
    }
    framing.finish()
}
