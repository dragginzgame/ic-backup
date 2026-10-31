//! One bounded upload session: full admission, exact payload commitments and closing verification.

use super::super::{check_closed_tree, check_directory_identity, member};
use super::{
    DownloadJournalGuard, File, IcSnapshotArtifactError, IcSnapshotDataUploadPlan,
    IcSnapshotUploadArtifactError, IcSnapshotUploadRequest, read_chunk,
};
use ic_host_fs::identity::FileIdentity;
use ic_platform::management_canister_types::SnapshotDataKind;
use std::{fs::Metadata, os::unix::fs::MetadataExt};

// Metadata observations detect drift; they are never trusted as content digests.
// Every transmitted payload must independently match its immutable original hash.
#[derive(Eq, PartialEq)]
struct Stamp {
    identity: FileIdentity,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}
impl From<Metadata> for Stamp {
    fn from(value: Metadata) -> Self {
        Self {
            identity: FileIdentity::from_metadata(&value),
            length: value.len(),
            modified: (value.mtime(), value.mtime_nsec()),
            changed: (value.ctime(), value.ctime_nsec()),
        }
    }
}

struct RetainedMember {
    member: member::ExpectedMember,
    stamp: Stamp,
}

pub(crate) struct UploadSession<'a, 'workflow, 'request, 'source> {
    source: &'a DownloadJournalGuard<'a>,
    snapshot: &'a str,
    upload: &'a IcSnapshotDataUploadPlan<'workflow, 'request, 'source>,
    directory: File,
    path: std::path::PathBuf,
    members: Vec<RetainedMember>,
}

impl<'a, 'workflow, 'request, 'source> UploadSession<'a, 'workflow, 'request, 'source> {
    pub(super) fn new(
        source: &'a DownloadJournalGuard<'a>,
        snapshot: &'a str,
        upload: &'a IcSnapshotDataUploadPlan<'workflow, 'request, 'source>,
    ) -> Result<Self, IcSnapshotUploadArtifactError> {
        let metadata = upload.metadata();
        let (directory, members) =
            source.with_upload_source(metadata.source_plan(), snapshot, metadata, |directory| {
                let held = directory
                    .try_clone()
                    .map_err(IcSnapshotArtifactError::from)?;
                let members = member::expected(metadata.source())?;
                let original = stamps(directory, members.iter())?;
                let members = members
                    .into_iter()
                    .zip(original)
                    .map(|(member, stamp)| RetainedMember { member, stamp })
                    .collect();
                Ok((held, members))
            })?;
        let entry = source
            .record
            .artifact(metadata.target(), snapshot)
            .map_err(super::super::DownloadJournalError::from)
            .map_err(IcSnapshotArtifactError::from)?;
        let session = Self {
            source,
            snapshot,
            upload,
            directory,
            path: source.layout.root().join(entry.artifact_path()),
            members,
        };
        session.check()?;
        Ok(session)
    }

    pub(crate) fn prepare(
        &self,
        sequence: u64,
        kind: SnapshotDataKind,
    ) -> Result<IcSnapshotUploadRequest<'source>, IcSnapshotUploadArtifactError> {
        self.check()?;
        let metadata = self.upload.metadata();
        let bytes = read_chunk(&self.directory, metadata.source(), &kind)?;
        let payload =
            IcSnapshotUploadRequest::data(metadata, self.upload.destination(), kind, &bytes)?;
        let plan = self
            .upload
            .plan()
            .ok_or(IcSnapshotUploadArtifactError::SourceMismatch)?;
        if plan
            .operation(sequence)
            .map_err(|_| IcSnapshotUploadArtifactError::SourceMismatch)?
            .request()
            != payload.binding_digest().hash()
        {
            return Err(IcSnapshotUploadArtifactError::SourceMismatch);
        }
        self.check()?;
        Ok(payload)
    }

    pub(crate) fn finish(self) -> Result<(), IcSnapshotUploadArtifactError> {
        self.check()?;
        self.source.require_upload_source(
            self.upload.metadata().source_plan(),
            self.snapshot,
            self.upload.metadata(),
        )?;
        self.check()
    }

    fn check(&self) -> Result<(), IcSnapshotUploadArtifactError> {
        self.source
            .admit_ic_artifact_original(self.upload.metadata().source_plan())
            .map_err(IcSnapshotArtifactError::from)?;
        self.source
            .check_artifact_parent()
            .map_err(IcSnapshotArtifactError::from)?;
        check_directory_identity(&self.path, &self.directory)?;
        // Observe every member before comparing, retaining original IO-error precedence.
        let current = stamps(
            &self.directory,
            self.members.iter().map(|original| &original.member),
        )?;
        if current
            .iter()
            .zip(&self.members)
            .any(|(current, original)| current != &original.stamp)
        {
            return Err(IcSnapshotArtifactError::CustodyChanged.into());
        }
        Ok(())
    }
}

fn stamps<'a>(
    directory: &File,
    members: impl Iterator<Item = &'a member::ExpectedMember> + Clone,
) -> Result<Vec<Stamp>, IcSnapshotArtifactError> {
    check_closed_tree(
        directory,
        members
            .clone()
            .map(|member| std::ffi::OsStr::new(&member.name)),
    )?;
    members
        .map(|member| {
            let (_, metadata) = member.open(directory)?;
            Ok(metadata.into())
        })
        .collect()
}
