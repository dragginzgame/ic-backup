//! Held payload bytes with ephemeral block commitments; no operation-evidence admission.
use super::{BackupStore, BackupSummary, PayloadDirectory, StoreError, unchanged_file};
use ic_host_artifacts::artifact::{ArtifactIdentity, Sha256Digest, hash_exact_reader};
use ic_platform::management_canister_types::{ReadCanisterSnapshotDataArgs, SnapshotDataKind};
use std::{
    fmt,
    fs::{File, Metadata},
    io::{self, Read, Seek, SeekFrom},
    num::NonZeroU64,
};
use thiserror::Error;

const BLOCK_BYTES: usize = 1024 * 1024;

/// Original payload descriptors and verified block commitments, not a restore handle.
///
/// Keeps the manifest, directory chain and every payload file open. The caller
/// must exclude namespace/content writers throughout its lifetime. Checks detect
/// observed changes but cannot fence writers or prove point-in-time immutability.
/// No plan, ledger, metadata evidence or application permission is admitted here.
pub struct PayloadSource<'store> {
    original: PayloadDirectory<'store>,
    members: Vec<HeldMember>,
    active: bool,
}

struct HeldMember {
    file: File,
    original: Metadata,
    blocks: Vec<Sha256Digest>,
}

impl BackupStore {
    /// Verify payload integrity and retain original files for bounded source reads.
    ///
    /// Uses the same trusted-manifest, byte-budget and closing admission as
    /// [`Self::verify_payload`]. During that scan, builds ephemeral SHA-256 values
    /// for 1 MiB blocks from the same bytes as each complete member hash. Nothing
    /// is written and the on-disk format is unchanged. Holds at most 1,027 payload
    /// descriptors plus the manifest/directory chain and temporary check handles;
    /// OS resource limits may refuse.
    /// Block hashes occupy less than 65 KiB under the existing 1 GiB/member caps;
    /// scan scratch is at most 1 MiB plus Host's fixed hashing buffer.
    ///
    /// The trusted manifest must come from original custody. Success establishes
    /// local payload integrity only, not authenticated capture or full bundle
    /// verification. Keep independent source evidence, restore intent and spending.
    ///
    /// ```no_run
    /// use ic_backup::store::BackupStore;
    /// use ic_host_artifacts::artifact::Sha256Digest;
    /// use ic_platform::management_canister_types::ReadCanisterSnapshotDataArgs;
    /// use std::num::NonZeroU64;
    ///
    /// fn read_extent(
    ///     store: &BackupStore, bundle: &str, trusted_manifest: Sha256Digest,
    ///     source_request: &ReadCanisterSnapshotDataArgs,
    /// ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    ///     let mut source = store.open_payload(
    ///         bundle, trusted_manifest, NonZeroU64::new(1 << 30).unwrap(),
    ///     )?;
    ///     // Keep source alive across successive reads in an actual transfer.
    ///     Ok(source.read_data(source_request)?)
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses the same malformed, excessive, unsafe, changed or corrupt sources
    /// as `verify_payload`, and descriptor/IO failures. Preserves all source files.
    pub fn open_payload(
        &self,
        bundle_id: &str,
        expected_manifest: Sha256Digest,
        maximum_payload_bytes: NonZeroU64,
    ) -> Result<PayloadSource<'_>, StoreError> {
        let original =
            PayloadDirectory::open(self, bundle_id, expected_manifest, maximum_payload_bytes)?;
        let members = original.verify_with(|file, length| {
            let metadata = file.metadata()?;
            let mut reader = IndexingReader {
                file,
                pending: Vec::with_capacity(BLOCK_BYTES.min(length as usize)),
                blocks: Vec::with_capacity(length.div_ceil(BLOCK_BYTES as u64) as usize),
            };
            let identity = hash_exact_reader(&mut reader, length)?;
            if !reader.pending.is_empty() {
                reader.blocks.push(Sha256Digest::compute(&reader.pending));
            }
            let held = HeldMember {
                file: reader.file.try_clone()?,
                original: metadata,
                blocks: reader.blocks,
            };
            Ok((identity, held))
        })?;
        let source = PayloadSource {
            original,
            members,
            active: true,
        };
        // Earlier files stay held while later files are scanned. Check their
        // original names again before returning this longer-lived capability.
        source.check_files()?;
        Ok(source)
    }
}

impl PayloadSource<'_> {
    pub(in crate::store) fn member_lengths(&self) -> Option<impl Iterator<Item = (&str, u64)>> {
        self.active.then(|| {
            self.original
                .decoded
                .members
                .iter()
                .map(|m| (m.name.as_str(), m.bytes))
        })
    }

    /// Manifest summary bound at opening; still not authenticated capture evidence.
    #[must_use]
    pub fn summary(&self) -> &BackupSummary {
        &self.original.decoded.summary
    }

    /// Read exact source bytes selected by a Platform capture-data request.
    ///
    /// Requires the original canister and snapshot ID. Regions are nonempty,
    /// within declared lengths and at most 1 MiB; named chunks are read whole,
    /// including empty chunks. The network/release and complete metadata/schedule
    /// association remain the integrating workflow's responsibility.
    ///
    /// Reads and rehashes at most two original blocks, then returns at most 1 MiB
    /// copied from those same verified bytes. Checks original names, manifest and
    /// directory custody before and after IO. Any IO/integrity/custody refusal
    /// invalidates this handle; an invalid request alone leaves it usable.
    /// Other members' contents are checked when read, not rescanned on each call.
    /// No provider call, progress record, completion or load authority is produced.
    ///
    /// # Errors
    /// Refuses wrong identities, unknown/malformed hashes, empty/overflowing/outside
    /// ranges, an invalidated handle, changed custody and length/hash/IO failures.
    pub fn read_data(
        &mut self,
        request: &ReadCanisterSnapshotDataArgs,
    ) -> Result<Vec<u8>, PayloadReadError> {
        if !self.active {
            return Err(PayloadReadError::Inactive);
        }
        let (index, offset, size) = self.select(request)?;
        self.active = false;
        let result = self.read_selected(index, offset, size)?;
        self.active = true;
        Ok(result)
    }

    fn select(
        &self,
        request: &ReadCanisterSnapshotDataArgs,
    ) -> Result<(usize, u64, usize), PayloadReadError> {
        if request.canister_id != self.summary().target.canister
            || !(1..=256).contains(&request.snapshot_id.len())
            || request
                .snapshot_id
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
                != self.summary().snapshot_id_hex
        {
            return Err(PayloadReadError::Request);
        }
        let (name, range) = match &request.kind {
            SnapshotDataKind::WasmModule { offset, size } => {
                ("module.wasm".to_owned(), Some((*offset, *size)))
            }
            SnapshotDataKind::WasmMemory { offset, size } => {
                ("heap.bin".to_owned(), Some((*offset, *size)))
            }
            SnapshotDataKind::StableMemory { offset, size } => {
                ("stable.bin".to_owned(), Some((*offset, *size)))
            }
            SnapshotDataKind::WasmChunk { hash } => {
                let hash = Sha256Digest::from_bytes(
                    hash.as_slice()
                        .try_into()
                        .map_err(|_| PayloadReadError::Request)?,
                );
                (format!("chunk-{hash}.bin"), None)
            }
        };
        let index = self
            .original
            .decoded
            .members
            .iter()
            .position(|member| member.name == name)
            .ok_or(PayloadReadError::Request)?;
        let member = &self.original.decoded.members[index];
        let (offset, size) = range.unwrap_or((0, member.bytes));
        if size > BLOCK_BYTES as u64
            || (range.is_some() && size == 0)
            || offset
                .checked_add(size)
                .is_none_or(|end| end > member.bytes)
        {
            return Err(PayloadReadError::Request);
        }
        Ok((index, offset, size as usize))
    }

    pub(in crate::store) fn check_files(&self) -> Result<(), StoreError> {
        self.original.check()?;
        for (member, held) in self.original.decoded.members.iter().zip(&self.members) {
            unchanged_file(
                &self.original.payload,
                &member.name,
                &held.file,
                &held.original,
            )?;
        }
        Ok(())
    }

    fn read_selected(
        &mut self,
        index: usize,
        offset: u64,
        size: usize,
    ) -> Result<Vec<u8>, StoreError> {
        self.check_files()?;
        let member = &self.original.decoded.members[index];
        let held = &mut self.members[index];
        let end = offset + size as u64; // Checked by select, bounded by the manifest.
        let mut output = Vec::with_capacity(size);
        if size != 0 {
            let first = offset / BLOCK_BYTES as u64;
            let last = (end - 1) / BLOCK_BYTES as u64;
            let mut block = Vec::with_capacity(BLOCK_BYTES);
            for ordinal in first..=last {
                let start = ordinal * BLOCK_BYTES as u64;
                let length = (member.bytes - start).min(BLOCK_BYTES as u64) as usize;
                block.resize(length, 0);
                held.file.seek(SeekFrom::Start(start))?;
                held.file.read_exact(&mut block)?;
                ArtifactIdentity {
                    bytes: length as u64,
                    sha256: Sha256Digest::compute(&block),
                }
                .verify(held.blocks[ordinal as usize])
                .map_err(|source| StoreError::Payload {
                    member: member.name.clone(),
                    source,
                })?;
                let from = offset.saturating_sub(start) as usize;
                let to = (end - start).min(length as u64) as usize;
                output.extend_from_slice(&block[from..to]);
            }
        }
        self.check_files()?;
        Ok(output)
    }
}

impl fmt::Debug for PayloadSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PayloadSource")
            .field("active", &self.active)
            .field("members", &self.members.len())
            .finish_non_exhaustive()
    }
}

// Observe exactly the bytes consumed by Host's whole-member hash. Only its
// successful digest admission can make these ephemeral block hashes trusted.
struct IndexingReader<'file> {
    file: &'file mut File,
    pending: Vec<u8>,
    blocks: Vec<Sha256Digest>,
}
impl Read for IndexingReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let count = self.file.read(output)?;
        let mut remaining = &output[..count];
        while !remaining.is_empty() {
            let take = remaining.len().min(BLOCK_BYTES - self.pending.len());
            self.pending.extend_from_slice(&remaining[..take]);
            remaining = &remaining[take..];
            if self.pending.len() == BLOCK_BYTES {
                self.blocks.push(Sha256Digest::compute(&self.pending));
                self.pending.clear();
            }
        }
        Ok(count)
    }
}

/// Source read refusal. Diagnostics contain no payload bytes or request contents.
#[derive(Debug, Error)]
pub enum PayloadReadError {
    /// The request does not select an admitted source identity and bounded member range.
    #[error("payload request does not match the admitted source")]
    Request,
    /// A previous filesystem/integrity check failed; reacquisition is explicit.
    #[error("payload source is invalidated")]
    Inactive,
    /// Original filesystem, manifest, custody or content refusal.
    #[error(transparent)]
    Store(#[from] StoreError),
}
