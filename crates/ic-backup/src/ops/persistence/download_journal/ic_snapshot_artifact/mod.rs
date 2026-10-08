//! Bounded local snapshot streaming joined to the existing durable download owner.

mod upload;
mod verification;
pub use upload::IcSnapshotUploadArtifactError;

use super::{DownloadJournalError, DownloadJournalGuard};
use crate::{
    hash::hex_bytes,
    model::{
        artifacts::{ArtifactChecksumRecord, ChecksumError},
        download_journal::ArtifactStateRecord,
        ic_snapshot_coverage::{IcSnapshotDataCoverage, IcSnapshotDataCoverageError},
        ic_snapshot_data::IcSnapshotDataReply,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, MAX_IC_SNAPSHOT_METADATA_BYTES},
    },
    ops::{
        artifacts::{ArtifactError, checksum_directory, checksum_relative_files},
        persistence::write_json_durable,
    },
};
use ic_management_canister_types::SnapshotDataKind;
use rustix::fs::{self as unix_fs, AtFlags, Dir, FileType, Mode, OFlags};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fmt,
    fs::{self, File},
    io::{self, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use thiserror::Error;

const FORMAT: &[u8] = b"ic-backup/ic-snapshot-artifact/v1\n";
const REGIONS: [&str; 3] = ["wasm-module.bin", "wasm-memory.bin", "stable-memory.bin"];

/// Exclusive ephemeral writer for one original journal artifact and exact IC metadata.
///
/// Each successful append retains decoded bytes in private descriptor-opened files.
/// It reuses the model's coverage owner and keeps only three hash states plus at most
/// 1,024 chunk checksums. It borrows the original layout/journal for its whole lifetime.
/// Appends consume the writer: any rejection or partial IO failure closes it and retains
/// the partial tree, without a completion transition or permission to repeat reads.
/// There is no partial-transfer reconstruction or automatic cleanup.
///
/// The integration must qualify the generic token's association with the raw IC ID,
/// authentic capture/metadata/data, original per-call spending, fresh read permission,
/// complete backend transfer and stable noncooperating byte/command custody. This writer
/// performs no provider calls and grants no upload/load/start, settlement or release.
pub struct IcSnapshotArtifactWriter<'journal, 'layout, 'metadata> {
    journal: &'journal mut DownloadJournalGuard<'layout>,
    coverage: IcSnapshotDataCoverage<'metadata>,
    snapshot: String,
    path: PathBuf,
    parent: File,
    directory: File,
    regions: [File; 3],
    hashes: [Sha256; 3],
    checksums: Vec<(PathBuf, ArtifactChecksumRecord)>,
}

impl<'layout> DownloadJournalGuard<'layout> {
    /// Create private staging for an existing Created artifact and exact decoded metadata.
    ///
    /// The exact generic backend token remains unchanged and is not parsed as a raw IC ID.
    /// The integration supplies their authoritative association. Target and timestamp must
    /// match the retained entry; its independently observed total snapshot size is preserved,
    /// never inferred from a sum of metadata regions. Exact original metadata wire bytes and
    /// request arguments are retained alongside fixed region/chunk files in a distinct v1
    /// artifact layout. Existing staging/canonical destinations reject without adoption.
    /// # Errors
    /// Rejects wrong identity/state/raw metadata, occupied or unsafe paths and IO failure.
    /// Failure retains partial bytes and leaves the original journal unchanged.
    pub fn stage_ic_snapshot_artifact<'journal, 'metadata>(
        &'journal mut self,
        snapshot: &str,
        metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
        raw_metadata: &[u8],
    ) -> Result<IcSnapshotArtifactWriter<'journal, 'layout, 'metadata>, IcSnapshotArtifactError>
    {
        self.check_usable()?;
        let entry = self
            .record
            .artifact(metadata.request().target(), snapshot)
            .map_err(DownloadJournalError::from)?;
        if entry.state() != ArtifactStateRecord::Created
            || entry.snapshot_taken_at_timestamp() != metadata.metadata().taken_at_timestamp
            || raw_metadata.len() > MAX_IC_SNAPSHOT_METADATA_BYTES
            || ArtifactChecksumRecord::from_bytes(raw_metadata) != *metadata.payload_checksum()
        {
            return Err(IcSnapshotArtifactError::OriginalMismatch);
        }
        self.check_artifact_parent()?;
        let path = self.layout.root().join(entry.staging_path());
        let canonical = self.layout.root().join(entry.artifact_path());
        match fs::symlink_metadata(canonical) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
            Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists).into()),
        }
        let parent = open_directory(&self.layout.root().join("artifacts"))?;
        let name = path
            .file_name()
            .ok_or(IcSnapshotArtifactError::CustodyChanged)?;
        unix_fs::mkdirat(&parent, name, Mode::from_bits_truncate(0o700)).map_err(errno_to_io)?;
        let directory = File::from(
            unix_fs::openat(
                &parent,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(errno_to_io)?,
        );
        let mut checksums = Vec::new();
        for (name, bytes) in [
            ("format", FORMAT),
            ("metadata.candid", raw_metadata),
            ("metadata-arguments.candid", metadata.request().arguments()),
        ] {
            create_file(&directory, name)?.write_all(bytes)?;
            checksums.push((name.into(), ArtifactChecksumRecord::from_bytes(bytes)));
        }
        let regions = [
            create_file(&directory, REGIONS[0])?,
            create_file(&directory, REGIONS[1])?,
            create_file(&directory, REGIONS[2])?,
        ];
        let writer = IcSnapshotArtifactWriter {
            journal: self,
            coverage: IcSnapshotDataCoverage::new(metadata),
            snapshot: snapshot.to_owned(),
            path,
            parent,
            directory,
            regions,
            hashes: std::array::from_fn(|_| Sha256::new()),
            checksums,
        };
        writer.check_custody()?;
        Ok(writer)
    }
}

impl<'layout> IcSnapshotArtifactWriter<'_, 'layout, '_> {
    /// Read the original declared coverage without changing it.
    #[must_use]
    pub const fn coverage(&self) -> &IcSnapshotDataCoverage<'_> {
        &self.coverage
    }

    /// Read the fixed private staging path; stable external custody remains required.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one exact admitted reply without retaining its memory buffer.
    ///
    /// Regions may interleave; each region must remain contiguous from zero. Chunk files
    /// use exact metadata hash names and exclusive creation. No fsync or journal transition
    /// occurs until explicit finish. Any error consumes this owner and retains all bytes.
    /// Before writing, all region names must still select the held regular files at
    /// their exact already-covered lengths. This sequential check is not a byte fence.
    /// # Errors
    /// Rejects changed custody, coverage conflicts and filesystem failures.
    pub fn append(
        mut self,
        reply: &IcSnapshotDataReply<'_, '_>,
    ) -> Result<Self, IcSnapshotArtifactError> {
        self.check_custody()?;
        self.check_region_custody()?;
        self.coverage.admit(reply)?;
        let index = match reply.request().kind() {
            SnapshotDataKind::WasmModule { .. } => 0,
            SnapshotDataKind::WasmMemory { .. } => 1,
            SnapshotDataKind::StableMemory { .. } => 2,
            SnapshotDataKind::WasmChunk { hash } => {
                let name = format!("chunk-{}.bin", hex_bytes(hash));
                create_file(&self.directory, &name)?.write_all(reply.chunk())?;
                self.checksums
                    .push((name.into(), reply.chunk_checksum().clone()));
                self.check_custody()?;
                return Ok(self);
            }
        };
        self.regions[index].write_all(reply.chunk())?;
        self.hashes[index].update(reply.chunk());
        self.check_custody()?;
        Ok(self)
    }

    /// Verify complete local bytes, retain their exact checksum and durably publish them.
    ///
    /// Complete declared coverage is required. The fixed closed tree must match hashes
    /// computed from the actual admitted bytes, including original metadata and request.
    /// Existing model transitions derive `Downloaded` and `ChecksumVerified` together; their
    /// exact checksum is persisted atomically before the existing durable publisher runs.
    /// On interruption, reopen the journal and use its ordinary `ChecksumVerified` publication
    /// recovery without another read or transfer. Created partial trees cannot be resumed
    /// through this writer; preserve them for operator-owned disposition. Ordinary Durable
    /// replay reads only retained progress, without fresh byte verification.
    ///
    /// The caller still owns authentic backend completeness, token/raw-ID association and
    /// stable custody. A returned checksum is local evidence, not a terminal/effect permit.
    /// # Errors
    /// Rejects incomplete coverage, extra/changed/unsafe bytes, replaced custody and failed
    /// persistence/publication. Failure can leave a verified or published tree; retain it.
    /// Closing directory-custody rejection can follow a retained `Durable` transition.
    pub fn finish(self) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
        self.finish_with(DownloadJournalGuard::finalize_artifact)
    }

    fn finish_with(
        mut self,
        publish: impl FnOnce(
            &mut DownloadJournalGuard<'layout>,
            &str,
            &str,
        ) -> Result<(), DownloadJournalError>,
    ) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
        if self.coverage.complete().is_none() {
            return Err(IcSnapshotArtifactError::IncompleteCoverage);
        }
        self.check_custody()?;
        for (name, hash) in REGIONS.into_iter().zip(self.hashes.iter()) {
            self.checksums.push((
                name.into(),
                ArtifactChecksumRecord::from_digest(hash.clone().finalize().into()),
            ));
        }
        self.check_closed_tree()?;
        let expected = checksum_relative_files(std::mem::take(&mut self.checksums))
            .map_err(ArtifactError::from)?;
        checksum_directory(&self.path)?.verify(expected.hash())?;
        self.check_custody()?;
        let target = self.coverage.metadata().request().target();
        let mut next = self.journal.next(
            target,
            &self.snapshot,
            ArtifactStateRecord::Downloaded,
            None,
        )?;
        next.advance(
            target,
            &self.snapshot,
            ArtifactStateRecord::ChecksumVerified,
            Some(expected.clone()),
        )
        .map_err(DownloadJournalError::from)?;
        self.journal.store(next, write_json_durable)?;
        publish(self.journal, target, &self.snapshot)?;
        self.journal.check_usable()?;
        check_directory_identity(
            self.path
                .parent()
                .ok_or(IcSnapshotArtifactError::CustodyChanged)?,
            &self.parent,
        )?;
        let entry = self
            .journal
            .record
            .artifact(target, &self.snapshot)
            .map_err(DownloadJournalError::from)?;
        check_directory_identity(
            &self.journal.layout.root().join(entry.artifact_path()),
            &self.directory,
        )?;
        Ok(expected)
    }

    fn check_custody(&self) -> Result<(), IcSnapshotArtifactError> {
        self.journal.check_usable()?;
        for (path, held) in [
            (
                self.path
                    .parent()
                    .ok_or(IcSnapshotArtifactError::CustodyChanged)?,
                &self.parent,
            ),
            (self.path.as_path(), &self.directory),
        ] {
            check_directory_identity(path, held)?;
        }
        Ok(())
    }

    fn check_closed_tree(&self) -> Result<(), IcSnapshotArtifactError> {
        check_closed_tree(&self.directory, &self.checksums)
    }

    fn check_region_custody(&self) -> Result<(), IcSnapshotArtifactError> {
        for ((name, file), expected) in REGIONS
            .iter()
            .zip(&self.regions)
            .zip(self.coverage.covered_region_bytes())
        {
            let held = unix_fs::fstat(file).map_err(errno_to_io)?;
            let current = unix_fs::statat(&self.directory, *name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(errno_to_io)?;
            let same_file = (current.st_dev, current.st_ino) == (held.st_dev, held.st_ino);
            let exact_extent = current.st_size == held.st_size
                && u64::try_from(held.st_size).ok() == Some(expected);
            if !FileType::from_raw_mode(current.st_mode).is_file() || !same_file || !exact_extent {
                return Err(IcSnapshotArtifactError::FileShape);
            }
        }
        Ok(())
    }
}

fn check_closed_tree(
    directory: &File,
    checksums: &[(PathBuf, ArtifactChecksumRecord)],
) -> Result<(), IcSnapshotArtifactError> {
    let mut expected = checksums
        .iter()
        .map(|(path, _)| path.as_os_str().as_encoded_bytes().to_vec())
        .collect::<BTreeSet<_>>();
    let mut directory = Dir::read_from(directory).map_err(errno_to_io)?;
    while let Some(entry) = directory.read() {
        let entry = entry.map_err(errno_to_io)?;
        let name = entry.file_name().to_bytes();
        if matches!(name, b"." | b"..") {
            continue;
        }
        if !expected.remove(name) {
            return Err(IcSnapshotArtifactError::UnexpectedEntry);
        }
    }
    if !expected.is_empty() {
        return Err(IcSnapshotArtifactError::UnexpectedEntry);
    }
    Ok(())
}

impl fmt::Debug for IcSnapshotArtifactWriter<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotArtifactWriter")
            .field("coverage", &self.coverage)
            .finish_non_exhaustive()
    }
}

fn open_directory(path: &Path) -> io::Result<File> {
    unix_fs::open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(errno_to_io)
}

fn check_directory_identity(path: &Path, held: &File) -> Result<(), IcSnapshotArtifactError> {
    let current = fs::symlink_metadata(path)?;
    let original = held.metadata()?;
    if !current.is_dir() || current.dev() != original.dev() || current.ino() != original.ino() {
        return Err(IcSnapshotArtifactError::CustodyChanged);
    }
    Ok(())
}

fn create_file(directory: &File, name: &str) -> io::Result<File> {
    unix_fs::openat(
        directory,
        name,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::from_bits_truncate(0o600),
    )
    .map(File::from)
    .map_err(errno_to_io)
}

fn errno_to_io(error: rustix::io::Errno) -> io::Error {
    io::Error::from_raw_os_error(error.raw_os_error())
}

/// Typed local artifact failures; no wire payload or raw snapshot identity is retained.
#[derive(Debug, Error)]
pub enum IcSnapshotArtifactError {
    /// Original journal state, timestamp or raw metadata differs.
    #[error("IC snapshot artifact original evidence mismatch")]
    OriginalMismatch,
    /// Complete declared data coverage has not been retained.
    #[error("IC snapshot artifact coverage is incomplete")]
    IncompleteCoverage,
    /// The original artifact parent or staging directory has been replaced.
    #[error("IC snapshot artifact directory custody changed")]
    CustodyChanged,
    /// The fixed closed artifact tree contains missing or additional entries.
    #[error("IC snapshot artifact tree entries differ")]
    UnexpectedEntry,
    /// A retained direct child differs in regular-file identity or bounded length.
    #[error("IC snapshot artifact file type or length differs")]
    FileShape,
    /// Retained original plan, complete durable selection or journal admission failed.
    #[error(transparent)]
    Integrity(#[from] super::DownloadIntegrityError),
    /// Exact metadata/range/chunk coverage admission failed.
    #[error(transparent)]
    Coverage(#[from] IcSnapshotDataCoverageError),
    /// Local expected admitted bytes differ from the current tree.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// Original download journal or durable publication failed.
    #[error(transparent)]
    Journal(#[from] DownloadJournalError),
    /// Descriptor-based artifact traversal failed.
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    /// Private descriptor creation or byte IO failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests;
