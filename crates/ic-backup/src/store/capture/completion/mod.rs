//! Exact create-only completion under the original closed journal prefix.
use super::{
    CaptureDataAdvanceError, CaptureError, CaptureFinalStatusError, CaptureOperation,
    CaptureProvider, PreparedStatus, StatusEvidence, StatusHttpTransport, StatusProgress,
    assembly::tree_members,
    child,
    ledger::{EventRecord, State},
    open_optional, same_directory,
};
use crate::store::{
    BackupSummary, MANIFEST_LIMIT, StoreError,
    manifest::{self, CommitmentRecord, ManifestRecord},
    payload::unchanged_file,
};
use ic_host_artifacts::artifact::{Sha256Digest, hash_exact_reader};
use ic_host_fs::{
    capability::Directory,
    durable::{create_retained_at, publish_retained_at},
};
use ic_platform::ic_protocol::{EnvelopeContent, decode_call_content_strict};
use std::{
    convert::Infallible,
    ffi::OsStr,
    io::Write,
    os::fd::AsFd,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

/// Completion refuses uncertainty without recapture, replacement or claim release.
#[derive(Debug, Error)]
pub enum CaptureCompletionError<E> {
    /// Original journal, bounds or custody refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Payload authentication or durable assembly refused.
    #[error("capture assembly refused completion")]
    Payload(Box<CaptureDataAdvanceError<E, Infallible>>),
    /// Historical final status authentication/policy refused.
    #[error("final capture status refused completion")]
    Status(Box<CaptureFinalStatusError<E>>),
    /// No settled successful final status or an unused unresolved lookup remains.
    #[error("capture completion requires settled final status and original lookup settlement")]
    FinalStatusRequired,
    /// Manifest construction, read or create-only publication refused.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// A first completion timestamp cannot be represented as positive Unix nanoseconds.
    #[error("capture completion clock refused")]
    Clock,
    /// Published manifest differs from the caller-selected original receipt.
    #[error("capture completion manifest mismatch")]
    ManifestMismatch,
}
impl CaptureOperation {
    /// Verify and durably publish the original local backup completion manifest.
    ///
    /// Requires already-settled final status; never signs, submits or reconciles a
    /// remote call. Reauthenticates source/final evidence and synchronizes exact
    /// assembled files. `PublishReady` freezes the original completion timestamp
    /// and ledger. Complete retained staging or a visible manifest can reconcile
    /// locally; partial, changed, aliased or duplicate files refuse without cleanup.
    /// Returns the original receipt on replay, without a new timestamp or ledger
    /// append. Explicit target succession uses this reconciliation; read-only completion
    /// verification and restore authority remain distinct.
    /// # Errors
    /// Refuses unfinished calls, invalid original evidence, changed files/custody,
    /// exhausted evidence budget, partial publication or local IO. Failed writes
    /// invalidate the live handle; reopen retains all files and authority.
    pub async fn complete_capture<P: CaptureProvider>(
        &mut self,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<BackupSummary, CaptureCompletionError<P::Error>> {
        self.authenticate_completion(transport, provider).await?;
        self.assemble_data(transport, provider)
            .await
            .map_err(|e| CaptureCompletionError::Payload(Box::new(e)))?;
        if self.check()?.completed_at_ns.is_none() {
            let completed_at_ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|d| u64::try_from(d.as_nanos()).ok())
                .filter(|n| *n > 0)
                .ok_or(CaptureCompletionError::Clock)?;
            self.commit(&EventRecord::PublishReady { completed_at_ns }, None)?;
        }
        let state = self.check()?;
        let bundles = child(&self.store, "bundles", false)?;
        let bundle = child(&bundles, &self.plan.request().operation_id, false)?;
        let bytes = self.manifest_bytes(&state, &bundle)?;
        let summary = self.manifest_summary(&bytes)?;
        // All fallible persistence retains an unusable live handle until closing checks.
        self.usable = false;
        self.publish_manifest(&bundle, &bytes)?;
        same_directory(&bundles, &self.plan.request().operation_id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        self.usable = true;
        let checked = self.check();
        self.usable = checked.is_ok();
        if checked? != state {
            self.usable = false;
            return Err(CaptureError::Changed.into());
        }
        Ok(summary)
    }

    /// Authenticate an already-published completion against the selected manifest hash.
    ///
    /// Rechecks original plan, closed journal, certified capture/data/final status and
    /// exact payload/manifest bytes under existing operation custody. No signing,
    /// HTTP, live admission, synchronization, publication or journal append occurs.
    /// Readiness or staging alone refuses; this never finishes an interrupted publish.
    /// The summary is passive evidence, not a restore approval or held payload source.
    /// Future restore must bind its own same-release/target intent and source custody.
    /// # Errors
    /// Refuses an absent or mismatched receipt, damaged original evidence or custody,
    /// failed authentication/policy, and IO. No refused file is repaired or removed.
    pub async fn verify_completion<P: CaptureProvider>(
        &mut self,
        expected_manifest: Sha256Digest,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<BackupSummary, CaptureCompletionError<P::Error>> {
        let state = self.check()?;
        if !self.check_assembly(&state)? {
            return Err(CaptureError::Incomplete.into());
        }
        let bundles = child(&self.store, "bundles", false)?;
        let bundle = child(&bundles, &self.plan.request().operation_id, false)?;
        let bytes = self.manifest_bytes(&state, &bundle)?;
        let summary = self.manifest_summary(&bytes)?;
        if summary.manifest_sha256 != expected_manifest {
            return Err(CaptureCompletionError::ManifestMismatch);
        }
        self.authenticate_completion(transport, provider).await?;
        self.verify_assembled_data(transport, provider)
            .await
            .map_err(|e| CaptureCompletionError::Payload(Box::new(e)))?;
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        Ok(summary)
    }

    async fn authenticate_completion<P: CaptureProvider>(
        &mut self,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<(), CaptureCompletionError<P::Error>> {
        let state = self.check()?;
        let final_status = state
            .final_status
            .as_ref()
            .filter(|s| s.phase == super::CaptureStatusPhase::Settled)
            .ok_or(CaptureCompletionError::FinalStatusRequired)?;
        if state
            .lookup
            .as_ref()
            .is_some_and(|l| l.settlement_root.is_none())
        {
            return Err(CaptureCompletionError::FinalStatusRequired);
        }
        let admission = final_status
            .admission
            .clone()
            .ok_or(CaptureError::Incomplete)?;
        if self
            .advance_final_status(&admission, transport, provider)
            .await
            .map_err(|e| CaptureCompletionError::Status(Box::new(e)))?
            != StatusProgress::Verified
        {
            return Err(CaptureCompletionError::FinalStatusRequired);
        }
        Ok(())
    }

    pub(super) fn check_manifest(
        &self,
        state: &State,
        bundle: &Directory,
    ) -> Result<bool, CaptureError> {
        let expected = self.manifest_bytes(state, bundle)?;
        let pending = open_optional(bundle, "manifest.pending")?;
        let published = open_optional(bundle, "manifest.json")?;
        if pending.is_some() && published.is_some() {
            return Err(CaptureError::Changed);
        }
        let completed = published.is_some();
        for (name, file) in [("manifest.pending", pending), ("manifest.json", published)] {
            if let Some(file) = file {
                verify_manifest_file(bundle, name, &file, &expected)?;
            }
        }
        Ok(completed)
    }

    fn manifest_summary(&self, bytes: &[u8]) -> Result<BackupSummary, StoreError> {
        manifest::decode(&self.plan.request().operation_id, bytes)
            .map(|manifest| manifest.summary)
            .map_err(|source| StoreError::Manifest {
                bundle: self.plan.request().operation_id.clone(),
                source,
            })
    }

    fn manifest_bytes(&self, state: &State, bundle: &Directory) -> Result<Vec<u8>, CaptureError> {
        let payload = child(bundle, "payload", false)?;
        let (digest, members) = tree_members(&payload, self.plan.request().limits.payload_bytes)?;
        if Some(digest) != state.assembly_sha256 {
            return Err(CaptureError::Changed);
        }
        same_directory(bundle, "payload", &payload)?;
        let m = state.metadata.as_ref().ok_or(CaptureError::Incomplete)?;
        let request = PreparedStatus::decode(&self.evidence(
            "metadata-request.bin",
            m.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        let evidence = StatusEvidence::decode(&self.evidence(
            "metadata-result.bin",
            m.result.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        let EnvelopeContent::Call { arg, .. } =
            decode_call_content_strict(&request.envelope, m.admission.call.request_bytes as usize)
                .map_err(|_| CaptureError::Changed)?
        else {
            return Err(CaptureError::Changed);
        };
        let mut ledger =
            open_optional(&self.directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
        let before = ledger.metadata()?;
        let identity = hash_exact_reader(&mut ledger, state.bytes).map_err(StoreError::from)?;
        unchanged_file(&self.directory, "ledger.bin", &ledger, &before)?;
        let p = self.plan.request();
        let record = ManifestRecord {
            format: manifest::FORMAT.into(),
            operation_id: p.operation_id.clone(),
            network: p.target.network.to_string(),
            canister: p.target.canister.to_text(),
            snapshot_id_hex: m.snapshot_id.iter().map(|b| format!("{b:02x}")).collect(),
            captured_at_ns: m.timestamp,
            completed_at_ns: state.completed_at_ns.ok_or(CaptureError::Incomplete)?,
            release: p.release.to_string(),
            module_sha256: p.module_sha256.to_string(),
            payload_bytes: members.iter().map(|m| m.bytes).sum(),
            plan: commitment(self.plan.encoded()),
            ledger: CommitmentRecord {
                bytes: identity.bytes,
                sha256: identity.sha256.to_string(),
            },
            metadata_arguments: commitment(&arg),
            metadata_reply: commitment(&evidence.response),
            members,
        };
        let bytes = serde_json::to_vec(&record)?;
        if bytes.len() > MANIFEST_LIMIT {
            return Err(CaptureError::Changed);
        }
        manifest::decode(&p.operation_id, &bytes).map_err(|source| StoreError::Manifest {
            bundle: p.operation_id.clone(),
            source,
        })?;
        Ok(bytes)
    }

    fn publish_manifest(&self, bundle: &Directory, bytes: &[u8]) -> Result<(), CaptureError> {
        self.ledger_file.sync_all()?;
        self.directory.sync()?;
        if let Some(file) = open_optional(bundle, "manifest.json")? {
            verify_manifest_file(bundle, "manifest.json", &file, bytes)?;
            file.sync_all()?;
            bundle.sync()?;
            return Ok(());
        }
        let parent = bundle.open_syncable()?;
        let file = if let Some(file) = open_optional(bundle, "manifest.pending")? {
            file
        } else {
            let mut file =
                create_retained_at(parent.as_fd(), OsStr::new("manifest.pending"), 0o600)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            file
        };
        verify_manifest_file(bundle, "manifest.pending", &file, bytes)?;
        publish_retained_at(
            parent.as_fd(),
            OsStr::new("manifest.pending"),
            OsStr::new("manifest.json"),
            file,
        )
        .map_err(StoreError::from)?;
        Ok(())
    }
}
fn commitment(bytes: &[u8]) -> CommitmentRecord {
    CommitmentRecord {
        bytes: bytes.len() as u64,
        sha256: Sha256Digest::compute(bytes).to_string(),
    }
}

fn verify_manifest_file(
    bundle: &Directory,
    name: &str,
    file: &std::fs::File,
    expected: &[u8],
) -> Result<(), CaptureError> {
    let before = file.metadata()?;
    let reader = unchanged_file(bundle, name, file, &before)?;
    let bytes =
        ic_host_fs::read::read_opened_file(reader, MANIFEST_LIMIT).map_err(StoreError::from)?;
    if bytes != expected {
        return Err(CaptureError::Changed);
    }
    unchanged_file(bundle, name, file, &before)?;
    Ok(())
}
