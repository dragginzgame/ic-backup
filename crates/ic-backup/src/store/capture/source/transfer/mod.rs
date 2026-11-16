//! Pure exact-length restore transfer schedule; no destination or dispatch authority.
use super::CaptureSource;
use crate::store::BackupDigest;
use crate::store::{BackupSummary, BackupTarget};
use ic_platform::management_canister_types::ReadCanisterSnapshotDataArgs;
use thiserror::Error;

/// Explicit same-target/release selection and bounds for a restore transfer calculation.
/// These are calculation limits, not persisted spending or provider admission.
#[derive(Clone, Debug)]
pub struct RestoreTransferRequest {
    /// Must equal the authenticated source network/canister.
    pub target: BackupTarget,
    /// Must equal the authenticated source release.
    pub release: BackupDigest,
    /// Region extent and whole-chunk ceiling, in 1..=1 MiB.
    pub extent_bytes: u32,
    /// Total actual payload ceiling, in 1..=1 GiB.
    pub maximum_payload_bytes: u64,
    /// Ceiling for the base `2*N + 7` management calls, in 1..=8192.
    pub maximum_management_calls: u32,
}

/// Deterministic source extents and base restore cost bound to one exact receipt.
///
/// An owned calculation, not a durable restore plan or authenticated destination.
/// Caller must retain/re-admit matching source custody. Metadata allocation,
/// payload request digests, provider overhead and restore intent remain separate.
#[derive(Debug)]
pub struct RestoreTransfer {
    source: BackupSummary,
    snapshot_id: Vec<u8>,
    sizes: [u64; 3],
    region_calls: [u64; 3],
    chunks: Vec<[u8; 32]>,
    extent: u64,
    data_calls: u32,
    management_calls: u32,
}

/// Pure restore transfer refusal; no attempt, record or file is changed.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum RestoreTransferError {
    /// Target/network or release does not match the authenticated source.
    #[error("restore target and release must match the original capture")]
    Selection,
    /// A calculation limit is zero or outside the supported profile.
    #[error("restore transfer limits are outside the supported profile")]
    Limits,
    /// The held payload reader was invalidated or its admitted shape is inconsistent.
    #[error("restore source is inactive or inconsistent")]
    Source,
    /// Actual source payload exceeds the caller's ceiling.
    #[error("restore payload requires {required} bytes, exceeding {limit}")]
    Payload {
        /// Exact authenticated source bytes.
        required: u64,
        /// Caller-selected total payload ceiling.
        limit: u64,
    },
    /// A whole stored chunk cannot be split to meet the selected extent ceiling.
    #[error("stored chunk has {bytes} bytes, exceeding extent ceiling {extent}")]
    Chunk {
        /// Exact whole-chunk length.
        bytes: u64,
        /// Caller-selected whole-chunk ceiling.
        extent: u32,
    },
    /// Upload/readback and fixed lifecycle calls exceed the selected ceiling.
    #[error("restore requires at least {required} management calls, exceeding {limit}")]
    Calls {
        /// Upload, readback and fixed lifecycle calls.
        required: u64,
        /// Caller-selected base management-call ceiling.
        limit: u32,
    },
}
impl CaptureSource<'_, '_> {
    /// Calculate exact source extents and base restore management-call count.
    ///
    /// No IO or providers: uses authenticated member lengths retained at admission.
    /// Shares capture's module/heap/stable-then-sorted-chunks ordering. Empty regions
    /// produce no requests; each whole chunk, including an empty one, produces one.
    /// Count is `2*N + 7`, including upload and full readback; additional signing,
    /// executor hops, observations and proof traffic require separate admission.
    /// No destination snapshot, restore intent or source reference is created.
    /// # Errors
    /// Refuses wrong target/release, invalid limits, inactive source, oversized
    /// whole chunks and insufficient actual payload or base call allowance.
    pub fn plan_restore_transfer(
        &self,
        request: &RestoreTransferRequest,
    ) -> Result<RestoreTransfer, RestoreTransferError> {
        plan_payload(&self.payload, request)
    }
}
pub(in crate::store) fn plan_payload(
    payload: &crate::store::PayloadSource<'_>,
    request: &RestoreTransferRequest,
) -> Result<RestoreTransfer, RestoreTransferError> {
    if request.target != payload.summary().target || request.release != payload.summary().release {
        return Err(RestoreTransferError::Selection);
    }
    if !(1..=1_048_576).contains(&request.extent_bytes)
        || !(1..=1_073_741_824).contains(&request.maximum_payload_bytes)
        || !(1..=8192).contains(&request.maximum_management_calls)
    {
        return Err(RestoreTransferError::Limits);
    }
    let source = payload.summary();
    if source.payload_bytes > request.maximum_payload_bytes {
        return Err(RestoreTransferError::Payload {
            required: source.payload_bytes,
            limit: request.maximum_payload_bytes,
        });
    }
    let mut sizes = [0; 3];
    let mut chunks = Vec::new();
    for (name, bytes) in payload
        .member_lengths()
        .ok_or(RestoreTransferError::Source)?
    {
        match name {
            "module.wasm" => sizes[0] = bytes,
            "heap.bin" => sizes[1] = bytes,
            "stable.bin" => sizes[2] = bytes,
            _ => {
                if bytes > u64::from(request.extent_bytes) {
                    return Err(RestoreTransferError::Chunk {
                        bytes,
                        extent: request.extent_bytes,
                    });
                }
                let hash: BackupDigest = name
                    .strip_prefix("chunk-")
                    .and_then(|s| s.strip_suffix(".bin"))
                    .ok_or(RestoreTransferError::Source)?
                    .parse()
                    .map_err(|_| RestoreTransferError::Source)?;
                chunks.push(*hash.as_bytes());
            }
        }
    }
    chunks.sort_unstable();
    // Source admission already bounds three regions to 1 GiB total and 1024 chunks.
    let extent = u64::from(request.extent_bytes);
    let region_calls = sizes.map(|size| size.div_ceil(extent));
    let data_calls = region_calls.into_iter().sum::<u64>() + chunks.len() as u64;
    let management_calls = 2 * data_calls + 7;
    if management_calls > u64::from(request.maximum_management_calls) {
        return Err(RestoreTransferError::Calls {
            required: management_calls,
            limit: request.maximum_management_calls,
        });
    }
    let snapshot_id = source
        .snapshot_id_hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|hex| {
            u8::from_str_radix(
                std::str::from_utf8(hex).map_err(|_| RestoreTransferError::Source)?,
                16,
            )
            .map_err(|_| RestoreTransferError::Source)
        })
        .collect::<Result<_, _>>()?;
    Ok(RestoreTransfer {
        source: source.clone(),
        snapshot_id,
        sizes,
        region_calls,
        chunks,
        extent,
        data_calls: u32::try_from(data_calls).map_err(|_| RestoreTransferError::Source)?,
        management_calls: u32::try_from(management_calls)
            .map_err(|_| RestoreTransferError::Source)?,
    })
}

impl RestoreTransfer {
    /// Exact authenticated receipt underlying the calculation, not restore permission.
    #[must_use]
    pub fn source(&self) -> &BackupSummary {
        &self.source
    }
    /// Extent/chunk count for each upload or readback pass.
    #[must_use]
    pub fn data_calls(&self) -> u32 {
        self.data_calls
    }
    /// Base upload/readback and lifecycle count; no provider or observation overhead.
    #[must_use]
    pub fn minimum_management_calls(&self) -> u32 {
        self.management_calls
    }
    /// Generate original-source read DTOs lazily. These are not upload requests.
    pub fn source_requests(
        &self,
    ) -> impl ExactSizeIterator<Item = ReadCanisterSnapshotDataArgs> + '_ {
        (0..self.data_calls).map(|n| ReadCanisterSnapshotDataArgs {
            canister_id: self.source.target.canister,
            snapshot_id: self.snapshot_id.clone(),
            kind: super::super::transfer::data_kind(
                self.sizes,
                self.region_calls,
                self.extent,
                &self.chunks,
                u64::from(n),
            ),
        })
    }
}
