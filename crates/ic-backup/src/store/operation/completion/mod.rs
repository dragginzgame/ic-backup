//! Local materialization and exact publication under the same fixed procedure.

use super::{
    FactRecord, Journal, Step,
    custody::{CaptureOperation, Role},
    replay::Error,
};
use crate::store::{
    BackupDigest, BackupSummary, MANIFEST_LIMIT, PRIVATE, StoreError,
    capture::{
        CaptureTransfer, OperationStoreError,
        assembly::tree_members,
        child,
        completion::{publish_manifest_file, verify_manifest_file},
        intent::CapturePlan,
        procedure::{Next, Procedure},
        same_directory,
    },
    manifest::{self, CommitmentRecord, ManifestRecord, MemberRecord},
    open_optional,
    operation::{effect::Effect, intent::Intent},
    payload::unchanged_file,
};
use ic_host_artifacts::Length;
use ic_host_fs::Kind as EntryKind;
use ic_platform_protocol::management_snapshots::{Extent, ReadKind, SnapshotArgs, SnapshotChunk};
use std::{
    ffi::OsStr,
    io::{Read, Write},
};

struct Member {
    name: String,
    ordinals: Vec<u32>,
}

impl CaptureOperation {
    // Structural custody may follow a completed receipt after ownership changes.
    // Public reopen/source/advance admission still authenticates the full procedure.
    // A held restore additionally checks its original manifest hash and payload files.
    pub(in crate::store) fn published_binding(
        &self,
        journal: &Journal,
    ) -> Result<bool, OperationStoreError> {
        let Some(completed_at_ns) = journal.completed_at_ns else {
            return Ok(false);
        };
        let bundles = child(&self.store, "bundles", false)?;
        let bundle = child(&bundles, &self.intent.request().operation_id, false)?;
        let Some(bytes) = crate::store::read_optional(&bundle, "manifest.json", MANIFEST_LIMIT)?
        else {
            return Ok(false);
        };
        if open_optional(&bundle, "manifest.pending")?.is_some() {
            return Err(OperationStoreError::Changed);
        }
        let manifest =
            manifest::decode(&self.intent.request().operation_id, &bytes).map_err(|source| {
                StoreError::Manifest {
                    bundle: self.intent.request().operation_id.clone(),
                    source,
                }
            })?;
        let before = self.journal_file.metadata()?;
        let mut reader =
            unchanged_file(&self.directory, "journal.bin", &self.journal_file, &before)?;
        let identity =
            ic_host_artifacts::copy(&mut reader, std::io::sink(), Length::Exactly(journal.bytes))
                .map_err(StoreError::from)?;
        unchanged_file(&self.directory, "journal.bin", &self.journal_file, &before)?;
        let payload = child(&bundle, "payload", false)?;
        let members = tree_members(&payload, self.intent.request().limits.payload_bytes)?;
        let p = self.intent.request();
        if manifest.plan != commitment(self.intent.encoded())
            || manifest.ledger
                != (CommitmentRecord {
                    bytes: identity.bytes,
                    sha256: BackupDigest::from_bytes(identity.sha256).to_string(),
                })
            || manifest.summary.target != p.target
            || manifest.summary.release != p.release
            || manifest.summary.module_sha256 != p.module_sha256
            || manifest.summary.completed_at_ns != completed_at_ns
            || manifest.members != members
        {
            return Err(OperationStoreError::Changed);
        }
        same_directory(&bundle, "payload", &payload)?;
        same_directory(&bundles, &p.operation_id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        Ok(true)
    }

    pub(in crate::store) fn published(
        &self,
        journal: &Journal,
        flow: &Procedure<'_>,
    ) -> Result<Option<BackupSummary>, Error> {
        if journal.completed_at_ns.is_none() {
            return Ok(None);
        }
        let bundles = child(&self.store, "bundles", false)?;
        let bundle = child(&bundles, &self.intent.request().operation_id, false)?;
        let bytes = self.manifest_bytes(journal, flow)?;
        let pending =
            open_optional(&bundle, "manifest.pending").map_err(OperationStoreError::from)?;
        let published =
            open_optional(&bundle, "manifest.json").map_err(OperationStoreError::from)?;
        if pending.is_some() && published.is_some() {
            return Err(OperationStoreError::Changed.into());
        }
        for (name, file) in [
            ("manifest.pending", &pending),
            ("manifest.json", &published),
        ] {
            if let Some(file) = file {
                verify_manifest_file(&bundle, name, file, &bytes)?;
            }
        }
        if published.is_none() {
            return Ok(None);
        }
        let summary = manifest::decode(&self.intent.request().operation_id, &bytes)
            .map_err(|source| {
                OperationStoreError::Store(StoreError::Manifest {
                    bundle: self.intent.request().operation_id.clone(),
                    source,
                })
            })?
            .summary;
        same_directory(&bundles, &summary.bundle_id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        Ok(Some(summary))
    }

    pub(in crate::store) fn sync_published(&mut self) -> Result<BackupSummary, Error> {
        let journal = self.check()?;
        let mut intent = CapturePlan::decode(self.intent.encoded())?;
        intent.attach_codec(self.intent.profile().codec);
        let flow = self.project(&journal, &intent)?;
        let summary = self
            .published(&journal, &flow)?
            .ok_or(OperationStoreError::Incomplete)?;
        let bundles = child(&self.store, "bundles", false)?;
        let bundle = child(&bundles, &summary.bundle_id, false)?;
        let payload = child(&bundle, "payload", false)?;
        let members = self.materialize(&journal, &flow, false)?;
        self.usable = false;
        for member in members {
            open_optional(&payload, &member.name)
                .map_err(OperationStoreError::from)?
                .ok_or(OperationStoreError::Incomplete)?
                .sync_all()
                .map_err(OperationStoreError::from)?;
        }
        open_optional(&bundle, "manifest.json")
            .map_err(OperationStoreError::from)?
            .ok_or(OperationStoreError::Incomplete)?
            .sync_all()
            .map_err(OperationStoreError::from)?;
        self.sync_evidence()?;
        payload.sync().map_err(OperationStoreError::from)?;
        bundle.sync().map_err(OperationStoreError::from)?;
        bundles.sync().map_err(OperationStoreError::from)?;
        self.finish_write(&journal)?;
        if self.verify_completion(summary.manifest_sha256)? != summary {
            return Err(OperationStoreError::Changed.into());
        }
        Ok(summary)
    }

    pub(in crate::store) fn check_bundle_layout(
        &self,
        journal: &Journal,
    ) -> Result<(), OperationStoreError> {
        let bundles = child(&self.store, "bundles", false)?;
        let id = &self.intent.plan().request().operation_id;
        let Some(bundle) = bundles.open_directory(OsStr::new(id), PRIVATE)? else {
            return if journal.completed_at_ns.is_some() {
                Err(OperationStoreError::Incomplete)
            } else {
                Ok(())
            };
        };
        if !journal.materialization {
            return Err(OperationStoreError::Changed);
        }
        for entry in bundle.entries(3)? {
            if !(entry.kind == EntryKind::Directory && entry.name == OsStr::new("payload"))
                && !(journal.completed_at_ns.is_some()
                    && entry.kind == EntryKind::File
                    && matches!(
                        entry.name.to_str(),
                        Some("manifest.pending" | "manifest.json")
                    ))
            {
                return Err(OperationStoreError::Changed);
            }
        }
        same_directory(&self.store, "bundles", &bundles)?;
        Ok(())
    }

    fn response(
        &self,
        journal: &Journal,
        step: Step,
        effect: impl FnOnce() -> Result<(Effect, usize), Error>,
    ) -> Result<Vec<u8>, Error> {
        let call = journal
            .calls
            .iter()
            .find(|c| c.step == step)
            .ok_or(OperationStoreError::Incomplete)?;
        if self.intent.profile().manager.is_some() {
            let result = super::replay::resolve_call(
                self.intent.view(),
                call,
                &effect()?,
                &mut |call, role| self.read_role(call, role),
            )?;
            return result
                .filter(|r| !r.rejected)
                .map(|r| r.response)
                .ok_or(Error::History);
        }
        if call.update.result.is_some() {
            return Ok(crate::store::capture::advance::StatusEvidence::decode(
                &self.read_role(call, Role::Result)?,
            )?
            .response);
        }
        let observation = call
            .observation
            .as_ref()
            .ok_or(OperationStoreError::Incomplete)?;
        let prepared = self.intent.view().decode_request(
            call.update.reference_ns,
            self.read_role(call, Role::Request)?,
        )?;
        match self.intent.view().observe(
            observation.reference_ns,
            &prepared,
            &self.read_role(call, Role::ObservationResult)?,
        )? {
            crate::store::capture::CaptureObservation::Replied(response) => Ok(response),
            _ => Err(Error::History),
        }
    }

    fn payload(
        &self,
        journal: &Journal,
        transfer: &CaptureTransfer<'_>,
        n: u32,
    ) -> Result<Vec<u8>, Error> {
        transfer
            .decode_data(
                n,
                &self.response(journal, Step::Data(n), || {
                    Ok((
                        Effect::Read(
                            transfer
                                .data_arguments(n)
                                .map_err(crate::store::capture::procedure::Error::from)?,
                        ),
                        1024,
                    ))
                })?,
            )
            .map_err(|error| crate::store::capture::procedure::Error::Data(error).into())
    }

    pub(in crate::store) fn materialize(
        &self,
        journal: &Journal,
        flow: &Procedure<'_>,
        create: bool,
    ) -> Result<Vec<MemberRecord>, Error> {
        if !journal.materialization {
            return Err(OperationStoreError::Incomplete.into());
        }
        let transfer = flow.transfer()?;
        let mut members: Vec<_> = ["module.wasm", "heap.bin", "stable.bin"]
            .into_iter()
            .map(|name| Member {
                name: name.into(),
                ordinals: Vec::new(),
            })
            .collect();
        for (n, request) in transfer.requests().enumerate() {
            let index = match request.kind {
                ReadKind::WasmModule(Extent { .. }) => 0,
                ReadKind::WasmMemory(Extent { .. }) => 1,
                ReadKind::StableMemory(Extent { .. }) => 2,
                ReadKind::WasmChunk(SnapshotChunk { hash }) => {
                    let hash = BackupDigest::from_bytes(
                        hash.as_slice()
                            .try_into()
                            .map_err(|_| OperationStoreError::Changed)?,
                    );
                    members.push(Member {
                        name: format!("chunk-{hash}.bin"),
                        ordinals: Vec::new(),
                    });
                    members.len() - 1
                }
            };
            members[index].ordinals.push(n as u32);
        }
        let bundles = child(&self.store, "bundles", false)?;
        let id = &self.intent.plan().request().operation_id;
        let bundle = child(&bundles, id, create)?;
        let payload = child(&bundle, "payload", create)?;
        for entry in payload
            .entries(members.len())
            .map_err(OperationStoreError::from)?
        {
            if entry.kind != EntryKind::File
                || !members.iter().any(|m| OsStr::new(&m.name) == entry.name)
            {
                return Err(OperationStoreError::Changed.into());
            }
        }
        // Admit all existing files before creating any missing member. Exact
        // completed files may be reused; partial or different files stay retained.
        let mut expected = Vec::with_capacity(members.len());
        let mut held = Vec::with_capacity(members.len());
        for member in &members {
            let mut file =
                open_optional(&payload, &member.name).map_err(OperationStoreError::from)?;
            if !create && file.is_none() {
                return Err(OperationStoreError::Incomplete.into());
            }
            let before = file
                .as_ref()
                .map(std::fs::File::metadata)
                .transpose()
                .map_err(OperationStoreError::from)?;
            let identity = ic_host_artifacts::hash_with(
                Length::AtMost(self.intent.plan().request().limits.payload_bytes),
                |hash| -> Result<(), Error> {
                    for &n in &member.ordinals {
                        let bytes = self.payload(journal, &transfer, n)?;
                        hash.write_all(&bytes).map_err(OperationStoreError::from)?;
                        // Before creating anything, admit every existing member.
                        // Read-only verification uses the complete closing tree hash below.
                        if create && let Some(file) = &mut file {
                            let mut actual = vec![0; bytes.len()];
                            file.read_exact(&mut actual)
                                .map_err(OperationStoreError::from)?;
                            if actual != bytes {
                                return Err(OperationStoreError::Changed.into());
                            }
                        }
                    }
                    Ok(())
                },
            )
            .map_err(|error| match error {
                ic_host_artifacts::ProduceError::Producer(error) => error,
                ic_host_artifacts::ProduceError::Admission(error) => {
                    OperationStoreError::from(StoreError::from(error)).into()
                }
            })?;
            if let (Some(file), Some(before)) = (&file, &before) {
                if before.len() != identity.bytes {
                    return Err(OperationStoreError::Changed.into());
                }
                unchanged_file(&payload, &member.name, file, before)
                    .map_err(OperationStoreError::from)?;
            }
            expected.push(MemberRecord {
                name: member.name.clone(),
                bytes: identity.bytes,
                sha256: BackupDigest::from_bytes(identity.sha256).to_string(),
            });
            held.push(file);
        }
        if create {
            for (member, file) in members.iter().zip(&mut held) {
                if file.is_none() {
                    let mut created = payload
                        .create_file(OsStr::new(&member.name), 0o600)
                        .map_err(OperationStoreError::from)?;
                    for &n in &member.ordinals {
                        created
                            .write_all(&self.payload(journal, &transfer, n)?)
                            .map_err(OperationStoreError::from)?;
                    }
                    *file = Some(created);
                }
                file.as_ref()
                    .ok_or(OperationStoreError::Incomplete)?
                    .sync_all()
                    .map_err(OperationStoreError::from)?;
            }
            payload.sync().map_err(OperationStoreError::from)?;
            bundle.sync().map_err(OperationStoreError::from)?;
            bundles.sync().map_err(OperationStoreError::from)?;
        }
        expected.sort_by(|a, b| a.name.cmp(&b.name));
        let actual = tree_members(&payload, self.intent.plan().request().limits.payload_bytes)?;
        if actual.len() != expected.len()
            || !actual
                .iter()
                .zip(&expected)
                .all(|(a, b)| a.name == b.name && a.bytes == b.bytes && a.sha256 == b.sha256)
        {
            return Err(OperationStoreError::Changed.into());
        }
        for (member, file) in members.iter().zip(&held) {
            let file = file.as_ref().ok_or(OperationStoreError::Incomplete)?;
            unchanged_file(
                &payload,
                &member.name,
                file,
                &file.metadata().map_err(OperationStoreError::from)?,
            )
            .map_err(OperationStoreError::from)?;
        }
        same_directory(&bundle, "payload", &payload)?;
        same_directory(&bundles, id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        Ok(actual)
    }

    pub(in crate::store) fn publish_closed(
        &mut self,
        mut journal: Journal,
        flow: &Procedure<'_>,
    ) -> Result<BackupSummary, Error> {
        if flow.next()? != Next::Publish {
            return Err(Error::History);
        }
        if journal.completed_at_ns.is_none() {
            self.usable = false;
            self.materialize(&journal, flow, true)?;
            self.finish_write(&journal)?;
            let completed_at_ns = super::runtime::now().ok_or(OperationStoreError::InvalidPlan)?;
            journal = self.append(&journal, FactRecord::PublishReady { completed_at_ns }, None)?;
        }
        let bytes = self.manifest_bytes(&journal, flow)?;
        let summary = manifest::decode(&self.intent.request().operation_id, &bytes)
            .map_err(|source| {
                OperationStoreError::Store(StoreError::Manifest {
                    bundle: self.intent.request().operation_id.clone(),
                    source,
                })
            })?
            .summary;
        let bundles = child(&self.store, "bundles", false)?;
        let bundle = child(&bundles, &summary.bundle_id, false)?;
        let pending =
            open_optional(&bundle, "manifest.pending").map_err(OperationStoreError::from)?;
        let published =
            open_optional(&bundle, "manifest.json").map_err(OperationStoreError::from)?;
        if pending.is_some() && published.is_some() {
            return Err(OperationStoreError::Changed.into());
        }
        for (name, file) in [
            ("manifest.pending", &pending),
            ("manifest.json", &published),
        ] {
            if let Some(file) = file {
                verify_manifest_file(&bundle, name, file, &bytes)?;
            }
        }
        // Completed replay is read-only, including no synchronization or live admission.
        if published.is_some() {
            return Ok(summary);
        }
        self.usable = false;
        self.journal_file
            .sync_all()
            .map_err(OperationStoreError::from)?;
        self.directory.sync().map_err(OperationStoreError::from)?;
        publish_manifest_file(&bundle, &bytes)?;
        same_directory(&bundles, &summary.bundle_id, &bundle)?;
        same_directory(&self.store, "bundles", &bundles)?;
        self.finish_write(&journal)?;
        Ok(summary)
    }

    pub(in crate::store) fn manifest_bytes(
        &self,
        journal: &Journal,
        flow: &Procedure<'_>,
    ) -> Result<Vec<u8>, Error> {
        let members = self.materialize(journal, flow, false)?;
        let transfer = flow.transfer()?;
        let snapshot_id = transfer.snapshot_id();
        let metadata_effect = Effect::Metadata(SnapshotArgs {
            canister_id: self.intent.request().target.canister,
            snapshot_id: snapshot_id.to_vec(),
        });
        let (_, arg) = metadata_effect.encode(512).map_err(|_| Error::History)?;
        let file = open_optional(&self.directory, "journal.bin")
            .map_err(OperationStoreError::from)?
            .ok_or(OperationStoreError::Incomplete)?;
        let before = file.metadata().map_err(OperationStoreError::from)?;
        let mut reader =
            unchanged_file(&self.directory, "journal.bin", &self.journal_file, &before)
                .map_err(OperationStoreError::from)?;
        let identity =
            ic_host_artifacts::copy(&mut reader, std::io::sink(), Length::Exactly(journal.bytes))
                .map_err(StoreError::from)
                .map_err(OperationStoreError::from)?;
        unchanged_file(&self.directory, "journal.bin", &self.journal_file, &before)
            .map_err(OperationStoreError::from)?;
        let p = self.intent.plan().request();
        let record = ManifestRecord {
            format: manifest::FORMAT.into(),
            operation_id: p.operation_id.clone(),
            network: p.target.network.to_string(),
            canister: p.target.canister.to_text(),
            snapshot_id_hex: snapshot_id.iter().map(|b| format!("{b:02x}")).collect(),
            captured_at_ns: transfer.metadata().taken_at_timestamp,
            completed_at_ns: journal
                .completed_at_ns
                .ok_or(OperationStoreError::Incomplete)?,
            release: p.release.to_string(),
            module_sha256: p.module_sha256.to_string(),
            payload_bytes: members.iter().map(|m| m.bytes).sum(),
            plan: commitment(self.intent.encoded()),
            ledger: CommitmentRecord {
                bytes: identity.bytes,
                sha256: BackupDigest::from_bytes(identity.sha256).to_string(),
            },
            metadata_arguments: commitment(&arg),
            metadata_reply: commitment(
                &self.response(journal, Step::Metadata, || Ok((metadata_effect, 512)))?,
            ),
            members,
        };
        let bytes = serde_json::to_vec(&record).map_err(OperationStoreError::from)?;
        if bytes.len() > MANIFEST_LIMIT {
            return Err(OperationStoreError::Changed.into());
        }
        Ok(bytes)
    }
}

fn commitment(bytes: &[u8]) -> CommitmentRecord {
    CommitmentRecord {
        bytes: bytes.len() as u64,
        sha256: BackupDigest::compute(bytes).to_string(),
    }
}
