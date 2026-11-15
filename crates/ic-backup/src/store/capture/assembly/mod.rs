//! Local create-only assembly from original authenticated evidence; no publication.
use super::{
    CaptureDataAdvanceError, CaptureError, CaptureOperation, CaptureProvider, CaptureTransfer,
    StatusHttpTransport, child,
    ledger::{EventRecord, State},
    open_optional, same_directory,
};
use crate::store::{PRIVATE, StoreError, payload::unchanged_file};
use ic_host_artifacts::artifact::{HashingWriter, Sha256Digest, hash_exact_reader};
use ic_host_fs::{
    capability::{Directory, EntryKind},
    durable::create_retained_at,
};
use ic_platform::management_canister_types::SnapshotDataKind;
use std::{
    convert::Infallible,
    ffi::OsStr,
    fs::File,
    io::{Read, Write, sink},
    os::fd::AsFd,
};

struct Member {
    name: String,
    bytes: u64,
    ordinals: Vec<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AssemblyMode {
    Verify,
    Reconcile,
}

impl CaptureOperation {
    /// Assemble authenticated payloads into the original bundle's private `payload/` tree.
    ///
    /// Reserves local assembly in the original journal before creating files.
    /// Reuses only complete exact files after original-evidence comparison; partial,
    /// changed or unsafe files refuse without repair. Missing files may be created
    /// until assembly is verified. Verified replay requires all original files.
    /// Retains all files on error; uncertain writes require reopening the operation.
    /// No signer, HTTP, final status check, completion manifest or claim release.
    /// Returns payload bytes, not a completed backup or restore receipt.
    /// # Errors
    /// Refuses incomplete/invalid original evidence, insufficient journal budget,
    /// unsafe or changed custody, incomplete existing files and local IO failures.
    pub async fn assemble_data<P: CaptureProvider>(
        &mut self,
        transport: &StatusHttpTransport,
        original: &mut P,
    ) -> Result<u64, CaptureDataAdvanceError<P::Error, Infallible>> {
        self.assemble_data_with(transport, original, AssemblyMode::Reconcile)
            .await
    }

    pub(super) async fn verify_assembled_data<P: CaptureProvider>(
        &mut self,
        transport: &StatusHttpTransport,
        original: &mut P,
    ) -> Result<u64, CaptureDataAdvanceError<P::Error, Infallible>> {
        self.assemble_data_with(transport, original, AssemblyMode::Verify)
            .await
    }

    async fn assemble_data_with<P: CaptureProvider>(
        &mut self,
        transport: &StatusHttpTransport,
        original: &mut P,
        mode: AssemblyMode,
    ) -> Result<u64, CaptureDataAdvanceError<P::Error, Infallible>> {
        if mode == AssemblyMode::Verify && self.check()?.assembly_sha256.is_none() {
            return Err(CaptureError::Incomplete.into());
        }
        let total = self.verify_data(transport, original).await?;
        let mut state = self.check()?;
        let m = state.metadata.as_ref().ok_or(CaptureError::Incomplete)?;
        let snapshot = self
            .metadata_snapshot(&m.admission, transport, original, &state)
            .await
            .map_err(|e| CaptureDataAdvanceError::Metadata(Box::new(e)))?;
        let metadata = self
            .retained_metadata(&state, &snapshot, transport)
            .map_err(|e| CaptureDataAdvanceError::Metadata(Box::new(e)))?;
        // Local immutable plan lets the journal retain intent without borrowing conflicts.
        let plan = super::CapturePlan::decode(self.plan.encoded().to_vec())?;
        let transfer = plan.plan_transfer(&snapshot.id, &metadata)?;
        let members = members(&transfer, &state)?;
        if !state.assembly_started {
            self.commit(&EventRecord::AssemblyStarted, None)?;
            state = self.check()?;
        }
        // Reconciliation invalidates the handle before fallible persistence.
        // Verification only compares existing files and never synchronizes them.
        if mode == AssemblyMode::Reconcile {
            self.usable = false;
        }
        let digest = self.assemble_files(&members, &state, &transfer, transport, mode)?;
        self.usable = true;
        let checked = self.check();
        self.usable = checked.is_ok();
        if checked? != state {
            self.usable = false;
            return Err(CaptureError::Changed.into());
        }
        if let Some(expected) = state.assembly_sha256 {
            if digest != expected {
                return Err(CaptureError::Changed.into());
            }
        } else {
            self.commit(&EventRecord::AssemblyVerified { sha256: digest }, None)?;
        }
        Ok(total)
    }

    fn assemble_files(
        &self,
        members: &[Member],
        state: &State,
        transfer: &CaptureTransfer<'_>,
        transport: &StatusHttpTransport,
        mode: AssemblyMode,
    ) -> Result<[u8; 32], super::DataReadError> {
        let bundles = child(&self.store, "bundles", false)?;
        let bundle = child(
            &bundles,
            &self.plan.request().operation_id,
            state.assembly_sha256.is_none(),
        )?;
        let directory = child(&bundle, "payload", state.assembly_sha256.is_none())?;
        let present = names(&directory, self.plan.request().limits.payload_bytes)?;
        if present
            .iter()
            .any(|name| !members.iter().any(|m| m.name == *name))
        {
            return Err(CaptureError::Changed.into());
        }
        let mut files = Vec::with_capacity(members.len());
        let mut expected = Vec::with_capacity(members.len());
        // Admit every existing member before creating any missing one.
        for member in members {
            let mut file = open_optional(&directory, &member.name).map_err(CaptureError::from)?;
            let before = file
                .as_ref()
                .map(File::metadata)
                .transpose()
                .map_err(CaptureError::from)?;
            if before.as_ref().is_some_and(|m| m.len() != member.bytes) {
                return Err(CaptureError::Changed.into());
            }
            if file.is_none() && state.assembly_sha256.is_some() {
                return Err(CaptureError::Incomplete.into());
            }
            let mut hash = HashingWriter::new(sink(), member.bytes);
            for &n in &member.ordinals {
                let bytes = self.data_payload(n, &state.data[n as usize], transfer, transport)?;
                hash.write_all(&bytes).map_err(CaptureError::from)?;
                if let Some(file) = &mut file {
                    let mut actual = vec![0; bytes.len()];
                    file.read_exact(&mut actual).map_err(CaptureError::from)?;
                    if actual != bytes {
                        return Err(CaptureError::Changed.into());
                    }
                }
            }
            if let (Some(file), Some(before)) = (&file, &before) {
                unchanged_file(&directory, &member.name, file, before)
                    .map_err(CaptureError::from)?;
            }
            expected.push(hash.identity());
            files.push(file);
        }
        same_directory(&bundle, "payload", &directory)?;
        same_directory(&bundles, &self.plan.request().operation_id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        if mode == AssemblyMode::Verify {
            return Ok(tree_digest(
                &directory,
                self.plan.request().limits.payload_bytes,
            )?);
        }
        let parent = directory.open_syncable().map_err(CaptureError::from)?;
        for ((member, file), expected) in members.iter().zip(&mut files).zip(&expected) {
            if file.is_none() {
                let mut created =
                    create_retained_at(parent.as_fd(), OsStr::new(&member.name), 0o600)
                        .map_err(CaptureError::from)?;
                // A failed write leaves the created file in place.
                for &n in &member.ordinals {
                    let bytes =
                        self.data_payload(n, &state.data[n as usize], transfer, transport)?;
                    created.write_all(&bytes).map_err(CaptureError::from)?;
                }
                *file = Some(created);
            }
            let file = file.as_ref().ok_or(CaptureError::Incomplete)?;
            file.sync_all().map_err(CaptureError::from)?;
            if file.metadata().map_err(CaptureError::from)?.len() != member.bytes {
                return Err(CaptureError::Changed.into());
            }
            let before = file.metadata().map_err(CaptureError::from)?;
            let mut reader = unchanged_file(&directory, &member.name, file, &before)
                .map_err(CaptureError::from)?;
            let actual = hash_exact_reader(&mut reader, member.bytes)
                .map_err(StoreError::from)
                .map_err(CaptureError::from)?;
            if actual != *expected {
                return Err(CaptureError::Changed.into());
            }
            unchanged_file(&directory, &member.name, file, &before).map_err(CaptureError::from)?;
        }
        directory.sync().map_err(CaptureError::from)?;
        bundle.sync().map_err(CaptureError::from)?;
        bundles.sync().map_err(CaptureError::from)?;
        self.ledger_file.sync_all().map_err(CaptureError::from)?;
        self.directory.sync().map_err(CaptureError::from)?;
        let digest = tree_digest(&directory, self.plan.request().limits.payload_bytes)?;
        same_directory(&bundle, "payload", &directory)?;
        same_directory(&bundles, &self.plan.request().operation_id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        Ok(digest)
    }

    pub(super) fn check_assembly(&self, state: &State) -> Result<bool, CaptureError> {
        let bundles = child(&self.store, "bundles", false)?;
        let bundle =
            bundles.open_directory(OsStr::new(&self.plan.request().operation_id), PRIVATE)?;
        let Some(bundle) = bundle else {
            return if state.assembly_sha256.is_some() {
                Err(CaptureError::Incomplete)
            } else {
                Ok(false)
            };
        };
        if !state.assembly_started {
            return Err(CaptureError::Changed);
        }
        for entry in bundle.entries(if state.completed_at_ns.is_some() {
            3
        } else {
            1
        })? {
            let entry = entry?;
            let payload = entry.name == OsStr::new("payload") && entry.kind == EntryKind::Directory;
            let manifest = state.completed_at_ns.is_some()
                && entry.kind == EntryKind::File
                && matches!(
                    entry.name.to_str(),
                    Some("manifest.pending" | "manifest.json")
                );
            if !payload && !manifest {
                return Err(CaptureError::Changed);
            }
        }
        let directory = bundle.open_directory(OsStr::new("payload"), PRIVATE)?;
        if let Some(directory) = directory {
            names(&directory, self.plan.request().limits.payload_bytes)?;
            if let Some(expected) = state.assembly_sha256
                && tree_digest(&directory, self.plan.request().limits.payload_bytes)? != expected
            {
                return Err(CaptureError::Changed);
            }
            same_directory(&bundle, "payload", &directory)?;
        } else if state.assembly_sha256.is_some() {
            return Err(CaptureError::Incomplete);
        }
        let published = state.completed_at_ns.is_some() && self.check_manifest(state, &bundle)?;
        same_directory(&bundles, &self.plan.request().operation_id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        Ok(published)
    }
}

fn members(transfer: &CaptureTransfer<'_>, state: &State) -> Result<Vec<Member>, CaptureError> {
    let mut members: Vec<_> = ["module.wasm", "heap.bin", "stable.bin"]
        .into_iter()
        .map(|name| Member {
            name: name.into(),
            bytes: 0,
            ordinals: Vec::new(),
        })
        .collect();
    for (n, request) in transfer.requests().enumerate() {
        let index = match request.kind {
            SnapshotDataKind::WasmModule { .. } => 0,
            SnapshotDataKind::WasmMemory { .. } => 1,
            SnapshotDataKind::StableMemory { .. } => 2,
            SnapshotDataKind::WasmChunk { hash } => {
                let hash = Sha256Digest::from_bytes(
                    hash.as_slice()
                        .try_into()
                        .map_err(|_| CaptureError::Changed)?,
                );
                members.push(Member {
                    name: format!("chunk-{hash}.bin"),
                    bytes: 0,
                    ordinals: Vec::new(),
                });
                members.len() - 1
            }
        };
        members[index].bytes += u64::from(
            state.data[n]
                .payload
                .as_ref()
                .ok_or(CaptureError::Incomplete)?
                .bytes,
        );
        members[index].ordinals.push(n as u32);
    }
    Ok(members)
}

fn names(directory: &Directory, limit: u64) -> Result<Vec<String>, CaptureError> {
    let mut names = Vec::new();
    let mut total = 0_u64;
    for entry in directory.entries(1027)? {
        let entry = entry?;
        let name = entry.name.to_str().ok_or(CaptureError::Changed)?;
        let valid = matches!(name, "module.wasm" | "heap.bin" | "stable.bin")
            || name
                .strip_prefix("chunk-")
                .and_then(|s| s.strip_suffix(".bin"))
                .is_some_and(|s| super::lower_hex(s, 64));
        if !valid || entry.kind != EntryKind::File {
            return Err(CaptureError::Changed);
        }
        let file = open_optional(directory, name)?.ok_or(CaptureError::Incomplete)?;
        total = total
            .checked_add(file.metadata()?.len())
            .filter(|n| *n <= limit)
            .ok_or(CaptureError::Changed)?;
        names.push(name.to_owned());
    }
    names.sort();
    Ok(names)
}

// Compact local commitment, not a manifest or an authentication substitute.
fn tree_digest(directory: &Directory, limit: u64) -> Result<[u8; 32], CaptureError> {
    Ok(tree_members(directory, limit)?.0)
}
pub(super) fn tree_members(
    directory: &Directory,
    limit: u64,
) -> Result<([u8; 32], Vec<crate::store::manifest::MemberRecord>), CaptureError> {
    let original = names(directory, limit)?;
    let mut digest = HashingWriter::new(sink(), 128 * 1024);
    digest.write_all(b"ic-backup/assembled-payload/1\0")?;
    let mut members = Vec::with_capacity(original.len());
    let mut held: Vec<(String, File, std::fs::Metadata)> = Vec::with_capacity(original.len());
    let mut total = 0_u64;
    for name in &original {
        let mut file = open_optional(directory, name)?.ok_or(CaptureError::Incomplete)?;
        let before = file.metadata()?;
        total = total
            .checked_add(before.len())
            .filter(|n| *n <= limit)
            .ok_or(CaptureError::Changed)?;
        let identity = hash_exact_reader(&mut file, before.len()).map_err(StoreError::from)?;
        digest.write_all(&(name.len() as u32).to_le_bytes())?;
        digest.write_all(name.as_bytes())?;
        digest.write_all(&identity.bytes.to_le_bytes())?;
        digest.write_all(identity.sha256.as_bytes())?;
        members.push(crate::store::manifest::MemberRecord {
            name: name.clone(),
            bytes: identity.bytes,
            sha256: identity.sha256.to_string(),
        });
        held.push((name.clone(), file, before));
    }
    for (name, file, before) in &held {
        unchanged_file(directory, name, file, before)?;
    }
    if names(directory, limit)? != original {
        return Err(CaptureError::Changed);
    }
    Ok((*digest.identity().sha256.as_bytes(), members))
}
