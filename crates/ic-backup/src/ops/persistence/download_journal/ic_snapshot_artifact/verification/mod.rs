//! Explicit fresh local checks of retained opt-in IC trees; no upload permission.

use super::{
    ArtifactChecksumRecord, DownloadJournalGuard, File, IcSnapshotArtifactError,
    IcSnapshotMetadataReply, Mode, OFlags, check_closed_tree, check_directory_identity,
    checksum_relative_files, member, open_directory_no_follow, unix_fs,
};
use crate::{
    model::operation_plan::OperationPlanRecord,
    ops::{
        artifacts::ArtifactError,
        persistence::{DownloadIntegrityError, read_operation_plan},
    },
    policy::download_integrity::validate,
};
use ic_host_artifacts::artifact::{ArtifactError as InputError, hash_exact_reader};
use std::{
    io::{self, Read},
    os::unix::fs::MetadataExt,
};

impl DownloadJournalGuard<'_> {
    /// Explicitly verify one published IC tree against exact original metadata and intent.
    ///
    /// Requires the retained full plan, unchanged held journal and complete Durable
    /// selected set. Only this target's artifact bytes are read. The fixed format,
    /// original metadata/request hashes, region lengths, known bounded chunk hashes
    /// and closed direct-child tree must match the retained checksum. No metadata
    /// defaults, generic-token/raw-ID inference or progress reconstruction occurs.
    ///
    /// Reads use no-follow descriptors and a bounded checksum buffer; journal and
    /// directory identities are rechecked at closing. These sequential observations
    /// cannot fence noncooperating writers or hold fresh byte custody after return.
    /// The returned checksum is passive local evidence. Integrations still qualify
    /// authentic complete transfer, token association, fresh permission/accounting,
    /// stable byte/command custody and upload/load/start safety. Nothing is written,
    /// spent, settled or released; ordinary resume never invokes this check.
    ///
    /// # Errors
    /// Rejects wrong original evidence, incomplete selection, changed records/custody,
    /// missing/unsafe/extra children, wrong bounded lengths/hashes and IO failure.
    pub fn verify_ic_snapshot_artifact(
        &self,
        plan: &OperationPlanRecord,
        snapshot: &str,
        metadata: &IcSnapshotMetadataReply<'_>,
    ) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
        self.admit_ic_artifact_original(plan)?;
        let entry = self
            .record
            .artifact(metadata.request().target(), snapshot)
            .map_err(super::DownloadJournalError::from)?;
        if entry.snapshot_taken_at_timestamp() != metadata.metadata().taken_at_timestamp {
            return Err(IcSnapshotArtifactError::OriginalMismatch);
        }
        let expected = entry
            .checksum()
            .ok_or(IcSnapshotArtifactError::OriginalMismatch)?;
        self.check_artifact_parent()?;
        let parent_path = self.layout.root().join("artifacts");
        let parent = open_directory_no_follow(&parent_path)?;
        let path = self.layout.root().join(entry.artifact_path());
        let directory = File::from(
            unix_fs::openat(
                &parent,
                path.file_name()
                    .ok_or(IcSnapshotArtifactError::CustodyChanged)?,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(io::Error::from)?,
        );
        checksum_ic_tree(&directory, metadata)?.verify(expected.hash())?;
        check_directory_identity(&parent_path, &parent)?;
        check_directory_identity(&path, &directory)?;
        self.admit_ic_artifact_original(plan)?;
        Ok(expected.clone())
    }

    fn admit_ic_artifact_original(
        &self,
        plan: &OperationPlanRecord,
    ) -> Result<(), DownloadIntegrityError> {
        self.check_usable()?;
        read_operation_plan(self.layout, &plan.digest())?;
        self.require_unchanged_integrity_journal()?;
        validate(plan, &self.record)?;
        Ok(())
    }
}

fn checksum_ic_tree(
    directory: &File,
    metadata: &IcSnapshotMetadataReply<'_>,
) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
    checksum_ic_tree_with(directory, metadata, checksum_child)
}

pub(super) fn checksum_ic_tree_with<E: From<IcSnapshotArtifactError>>(
    directory: &File,
    metadata: &IcSnapshotMetadataReply<'_>,
    mut read_child: impl FnMut(&File, &member::ExpectedMember) -> Result<ArtifactChecksumRecord, E>,
) -> Result<ArtifactChecksumRecord, E> {
    let members = member::expected(metadata)?;
    check_closed_tree(
        directory,
        members
            .iter()
            .map(|member| std::ffi::OsStr::new(&member.name)),
    )?;
    let mut checksums = Vec::with_capacity(members.len());
    for member in members {
        let actual = read_child(directory, &member)?;
        if let Some(expected) = &member.checksum {
            actual
                .verify(expected.hash())
                .map_err(IcSnapshotArtifactError::from)?;
        }
        checksums.push((member.name.into(), actual));
    }
    check_closed_tree(
        directory,
        checksums
            .iter()
            .map(|(path, _): &(std::path::PathBuf, _)| path.as_os_str()),
    )?;
    checksum_relative_files(checksums)
        .map_err(ArtifactError::from)
        .map_err(IcSnapshotArtifactError::from)
        .map_err(E::from)
}

fn checksum_child(
    directory: &File,
    member: &member::ExpectedMember,
) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
    checksum_child_with(
        directory,
        &member.name,
        member.length,
        member.maximum,
        |file, length| checksum_exact_reader(file, length),
    )
}

pub(super) fn checksum_child_with<E: From<IcSnapshotArtifactError>>(
    directory: &File,
    name: &str,
    length: Option<u64>,
    maximum: u64,
    read: impl FnOnce(&mut File, u64) -> Result<ArtifactChecksumRecord, E>,
) -> Result<ArtifactChecksumRecord, E> {
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let (mut file, original) = open_regular_child(directory, name, length, maximum)?;
    let checksum = read(&mut file, original.len())?;
    let held = file.metadata().map_err(IcSnapshotArtifactError::from)?;
    let current = File::from(
        unix_fs::openat(directory, name, flags, Mode::empty())
            .map_err(io::Error::from)
            .map_err(IcSnapshotArtifactError::from)?,
    )
    .metadata()
    .map_err(IcSnapshotArtifactError::from)?;
    if !current.is_file()
        || original.dev() != current.dev()
        || original.ino() != current.ino()
        || original.len() != current.len()
        || original.len() != held.len()
    {
        return Err(IcSnapshotArtifactError::CustodyChanged.into());
    }
    Ok(checksum)
}

pub(super) fn checksum_exact_reader<R: Read>(
    reader: R,
    length: u64,
) -> Result<ArtifactChecksumRecord, IcSnapshotArtifactError> {
    let identity = hash_exact_reader(reader, length).map_err(|error| match error {
        InputError::LimitExceeded { .. } | InputError::LengthMismatch { .. } => {
            IcSnapshotArtifactError::FileShape
        }
        error => ArtifactError::Io(error.into()).into(),
    })?;
    Ok(ArtifactChecksumRecord::from_digest(
        *identity.sha256.as_bytes(),
    ))
}

pub(super) fn open_regular_child(
    directory: &File,
    name: &str,
    length: Option<u64>,
    maximum: u64,
) -> Result<(File, std::fs::Metadata), IcSnapshotArtifactError> {
    let file = File::from(
        unix_fs::openat(
            directory,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io::Error::from)?,
    );
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.len() > maximum
        || length.is_some_and(|length| length != metadata.len())
    {
        return Err(IcSnapshotArtifactError::FileShape);
    }
    Ok((file, metadata))
}

#[cfg(test)]
mod tests;
