//! Fixture-owned terminal admission over the actual connected `PocketIC` journey.

use super::{
    ArtifactChecksumRecord, BackupLayoutGuard, CompleteDriver, DownloadJournalGuard,
    ExecutionWorkflowRecord, IcSnapshotMetadataReply, IcSnapshotMetadataRequest,
    IcSnapshotUploadReply, IcSnapshotUploadReplyKind, Method, Path, TOKEN, files, fs, io,
    management, read_complete_execution_workflow,
};
use ic_backup::{
    ops::persistence::{
        JournalLock, RestoreReferenceReleaseError, RestoreReferenceReleaseOutcome,
        RestoreTerminalEvidenceProvider, read_complete_execution_workflow_locked,
        read_execution_progress,
    },
    view::restore_reference_release::TerminalRestoreEvidence,
};

struct Terminal<'a> {
    driver: &'a CompleteDriver,
    restore: &'a BackupLayoutGuard,
    source: &'a BackupLayoutGuard,
    journal: &'a JournalLock,
    accepted: Option<&'a ArtifactChecksumRecord>,
}

impl RestoreTerminalEvidenceProvider for Terminal<'_> {
    type Error = io::Error;

    fn qualify_terminal(
        &self,
        journal: &Path,
        bytes: &[u8],
    ) -> Result<TerminalRestoreEvidence, Self::Error> {
        let original: ExecutionWorkflowRecord = serde_json::from_slice(bytes)?;
        require(
            journal == self.restore.root().join("execution-workflow.json")
                && original == self.driver.workflow,
        )?;
        read_complete_execution_workflow_locked(self.restore, self.journal, &original.digest())
            .map_err(io::Error::other)?;
        // Local Applied checkpoints are necessary, but cannot supply this independent
        // fixture-owned acceptance of the loaded program and its application state.
        let accepted = self.accepted.ok_or(io::ErrorKind::PermissionDenied)?;
        let source = self
            .driver
            .restore
            .as_ref()
            .ok_or(io::ErrorKind::InvalidData)?;
        let request = IcSnapshotMetadataRequest::new(
            &self.driver.backend.target.to_text(),
            self.driver
                .snapshot
                .as_ref()
                .ok_or(io::ErrorKind::InvalidData)?
                .id(),
        )
        .map_err(io::Error::other)?;
        let metadata = IcSnapshotMetadataReply::decode(
            &request,
            self.driver
                .raw_metadata
                .as_ref()
                .ok_or(io::ErrorKind::InvalidData)?,
        )
        .map_err(io::Error::other)?;
        let download = DownloadJournalGuard::open(self.source, source.source_plan.digest().hash())
            .map_err(io::Error::other)?;
        require(
            download
                .verify_ic_snapshot_artifact(&source.source_plan, TOKEN, &metadata)
                .map_err(io::Error::other)?
                == source.source.checksum
                && download.record().map_err(io::Error::other)?.digest() == source.source.manifest,
        )?;
        let progress = read_execution_progress(self.source, &source.source_plan.digest())
            .map_err(io::Error::other)?;
        require(progress.applied_operations == source.source_plan.operations().len())?;
        let upload = download
            .prepare_ic_snapshot_upload_metadata(&source.source_plan, TOKEN, &metadata)
            .map_err(io::Error::other)?;
        let allocation = &source
            .allocation
            .as_ref()
            .ok_or(io::ErrorKind::InvalidData)?
            .2;
        let reply = IcSnapshotUploadReply::decode(&upload, allocation).map_err(io::Error::other)?;
        let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = reply.kind() else {
            return Err(io::ErrorKind::InvalidData.into());
        };
        let load = self.restore.root().join("execution-stage-17");
        require(fs::read(load.join("fixture-accepted.sha256"))? == accepted.hash().as_bytes())?;
        let verification: serde_json::Value = serde_json::from_slice(&fs::read(
            self.driver.backend.root.join("post-load-verification.json"),
        )?)?;
        require(
            verification["original_source_metadata"] == serde_json::to_value(metadata.digest())?
                && verification["loaded_destination"] == serde_json::to_value(snapshot_id)?
                && verification["complete_state_evidence"] == serde_json::to_value(accepted)?,
        )?;
        for sequence in [17, 3] {
            let stage = self
                .restore
                .root()
                .join(format!("execution-stage-{sequence}"));
            let request = management(
                &self.driver.backend,
                if sequence == 17 {
                    Method::LoadCanisterSnapshot
                } else {
                    Method::StartCanister
                },
                (sequence == 17).then(|| snapshot_id.clone()),
            );
            require(
                fs::read(stage.join("restore-arguments.candid"))? == request.arguments()
                    && fs::read(stage.join("restore-reply.candid"))? == b"DIDL\0\0",
            )?;
        }
        // This inspected, exclusively owned program has no timers, outbound calls,
        // payments or application fence. All management calls are in-process; no
        // external backup command was created, so its complete command set is empty.
        // This assertion is specific to state.wat, never a production safety default.
        Ok(TerminalRestoreEvidence::new(original.digest(), vec![]))
    }
}

fn require(condition: bool) -> io::Result<()> {
    condition
        .then_some(())
        .ok_or(io::ErrorKind::InvalidData.into())
}

pub(super) fn check(
    driver: &CompleteDriver,
    restore: &BackupLayoutGuard,
    capture: &BackupLayoutGuard,
    complete: bool,
) {
    let original_capture = super::super::allocated_workflow(&driver.backend);
    read_complete_execution_workflow(capture, &original_capture.digest()).unwrap();
    let original = driver.restore.as_ref().unwrap();
    let source = BackupLayoutGuard::acquire(&original.source.root).unwrap();
    let lock = JournalLock::acquire(&restore.root().join("execution-workflow.json")).unwrap();
    let mut provider = Terminal {
        driver,
        restore,
        source: &source,
        journal: &lock,
        accepted: None,
    };
    let before = files(capture.root().parent().unwrap());
    let calls = driver.backend.calls;
    let references = source.restore_references().unwrap();
    assert!(
        references
            .entries()
            .iter()
            .any(|entry| entry.journal() == lock.journal())
    );
    let unrelated: Vec<_> = references
        .entries()
        .iter()
        .filter(|entry| entry.journal() != lock.journal())
        .cloned()
        .collect();
    assert!(
        !unrelated.is_empty(),
        "fixture retains another unfinished restore"
    );
    assert!(matches!(
        source.release_restore(&lock, &provider),
        Err(RestoreReferenceReleaseError::Terminal(_))
    ));
    assert_eq!(files(capture.root().parent().unwrap()), before);
    if complete {
        let wrong_acceptance = ArtifactChecksumRecord::from_bytes(b"another post-load acceptance");
        provider.accepted = Some(&wrong_acceptance);
        assert!(matches!(
            source.release_restore(&lock, &provider),
            Err(RestoreReferenceReleaseError::Terminal(_))
        ));
        assert_eq!(files(capture.root().parent().unwrap()), before);
        provider.accepted = original.acceptance.as_ref();
        assert_eq!(
            source.release_restore(&lock, &provider).unwrap(),
            RestoreReferenceReleaseOutcome::Released
        );
        assert_eq!(source.restore_references().unwrap().entries(), unrelated);
        assert!(source.has_restore_references().unwrap());
        let mut after = files(capture.root().parent().unwrap());
        let references = source.root().join("restore-references.json");
        let released = after.insert(references.clone(), before[&references].clone());
        assert_ne!(released.as_ref(), Some(&before[&references]));
        assert_eq!(after, before);
        let replay = files(capture.root().parent().unwrap());
        assert_eq!(
            source.release_restore(&lock, &provider).unwrap(),
            RestoreReferenceReleaseOutcome::AlreadyReleased
        );
        assert_eq!(files(capture.root().parent().unwrap()), replay);
        // Even absent-reference replay must freshly qualify application acceptance.
        provider.accepted = None;
        assert!(matches!(
            source.release_restore(&lock, &provider),
            Err(RestoreReferenceReleaseError::Terminal(_))
        ));
        assert_eq!(files(capture.root().parent().unwrap()), replay);
    } else {
        provider.accepted = original.acceptance.as_ref();
        assert!(matches!(
            source.release_restore(&lock, &provider),
            Err(RestoreReferenceReleaseError::Terminal(_))
        ));
        assert!(source.has_restore_references().unwrap());
        assert_eq!(files(capture.root().parent().unwrap()), before);
    }
    assert_eq!(driver.backend.calls, calls);
}
