//! Bounded completion declarations. Decoding grants no restore authority.

use super::{BackupSummary, BackupTarget};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::Principal;
use serde::{
    Deserialize, Deserializer,
    de::{self, SeqAccess, Visitor},
};
use std::{collections::BTreeSet, fmt};
use thiserror::Error;

pub(super) const FORMAT: &str = "ic-backup/local-manifest/1";
const MAX_MEMBERS: usize = 1027; // Three regions plus at most 1,024 whole chunks.
const MAX_PAYLOAD: u64 = 1024 * 1024 * 1024;

#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManifestRecord {
    pub format: String,
    pub operation_id: String,
    pub network: String,
    pub canister: String,
    pub snapshot_id_hex: String,
    pub captured_at_ns: u64,
    pub completed_at_ns: u64,
    pub release: String,
    pub module_sha256: String,
    pub payload_bytes: u64,
    pub plan: CommitmentRecord,
    pub ledger: CommitmentRecord,
    pub metadata_arguments: CommitmentRecord,
    pub metadata_reply: CommitmentRecord,
    #[serde(deserialize_with = "members")]
    pub members: Vec<MemberRecord>,
}

#[derive(Deserialize, serde::Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct CommitmentRecord {
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Deserialize, serde::Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct MemberRecord {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
}

pub(super) struct DecodedManifest {
    pub summary: BackupSummary,
    pub members: Vec<MemberRecord>,
    pub plan: CommitmentRecord,
    pub ledger: CommitmentRecord,
}

pub(super) fn lower_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn digest(value: &str) -> Result<Sha256Digest, ManifestError> {
    if !lower_hex(value, 64) {
        return Err(ManifestError::Invalid("digest"));
    }
    value.parse().map_err(|_| ManifestError::Invalid("digest"))
}

pub(super) fn decode(id: &str, bytes: &[u8]) -> Result<DecodedManifest, ManifestError> {
    let record: ManifestRecord = serde_json::from_slice(bytes)?;
    if record.format != FORMAT {
        return Err(ManifestError::UnsupportedFormat);
    }
    if record.operation_id != id || !lower_hex(id, 32) {
        return Err(ManifestError::Invalid("operation identity"));
    }
    let canister =
        Principal::from_text(&record.canister).map_err(|_| ManifestError::Invalid("canister"))?;
    if canister.to_text() != record.canister {
        return Err(ManifestError::Invalid("canonical canister"));
    }
    let snapshot_len = record.snapshot_id_hex.len();
    if !(2..=512).contains(&snapshot_len)
        || !snapshot_len.is_multiple_of(2)
        || !lower_hex(&record.snapshot_id_hex, snapshot_len)
    {
        return Err(ManifestError::Invalid("snapshot identity"));
    }
    for (commitment, maximum) in [
        (&record.plan, 1024 * 1024),
        (&record.ledger, 128 * 1024 * 1024),
        (&record.metadata_arguments, 4096),
        (&record.metadata_reply, 1024 * 1024),
    ] {
        if commitment.bytes == 0 || commitment.bytes > maximum {
            return Err(ManifestError::Invalid("evidence length"));
        }
        digest(&commitment.sha256)?;
    }
    let mut names = BTreeSet::new();
    let mut total = 0_u64;
    let module_hash = digest(&record.module_sha256)?;
    for member in &record.members {
        if !names.insert(member.name.as_str()) {
            return Err(ManifestError::Invalid("duplicate member"));
        }
        let hash = digest(&member.sha256)?;
        match member.name.as_str() {
            "module.wasm" => {
                if member.bytes == 0 || hash != module_hash {
                    return Err(ManifestError::Invalid("module binding"));
                }
            }
            "heap.bin" | "stable.bin" => {}
            name => {
                let hash_name = name
                    .strip_prefix("chunk-")
                    .and_then(|s| s.strip_suffix(".bin"))
                    .ok_or(ManifestError::Invalid("member name"))?;
                if digest(hash_name)? != hash || member.bytes > 1024 * 1024 {
                    return Err(ManifestError::Invalid("chunk binding"));
                }
            }
        }
        total = total
            .checked_add(member.bytes)
            .filter(|n| *n <= MAX_PAYLOAD)
            .ok_or(ManifestError::Invalid("payload length"))?;
    }
    if total != record.payload_bytes
        || !["module.wasm", "heap.bin", "stable.bin"]
            .iter()
            .all(|n| names.contains(n))
    {
        return Err(ManifestError::Invalid("payload coverage"));
    }
    let summary = BackupSummary {
        bundle_id: record.operation_id,
        target: BackupTarget {
            network: digest(&record.network)?,
            canister,
        },
        snapshot_id_hex: record.snapshot_id_hex,
        captured_at_ns: record.captured_at_ns,
        completed_at_ns: record.completed_at_ns,
        release: digest(&record.release)?,
        module_sha256: module_hash,
        payload_bytes: total,
        manifest_sha256: Sha256Digest::compute(bytes),
    };
    Ok(DecodedManifest {
        summary,
        members: record.members,
        plan: record.plan,
        ledger: record.ledger,
    })
}

fn members<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<MemberRecord>, D::Error> {
    struct Members;
    impl<'de> Visitor<'de> for Members {
        type Value = Vec<MemberRecord>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("at most 1027 payload members")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut rows = Vec::new();
            while rows.len() < MAX_MEMBERS {
                let Some(row) = seq.next_element()? else {
                    return Ok(rows);
                };
                rows.push(row);
            }
            if seq.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom("payload member limit exceeded"));
            }
            Ok(rows)
        }
    }
    deserializer.deserialize_seq(Members)
}

/// Manifest decoding/admission failure; successful decoding is still passive metadata.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// Invalid JSON, duplicate/unknown/missing fields, depth or collection limits.
    #[error("malformed completion record")]
    Encoding(#[from] serde_json::Error),
    /// The manifest is from an unsupported generation.
    #[error("unsupported completion format")]
    UnsupportedFormat,
    /// A required semantic binding or numeric bound failed.
    #[error("invalid completion {0}")]
    Invalid(&'static str),
}
