//! Held original files for the common recorder. No caller-selected evidence paths.

#[cfg(test)]
pub(in crate::store) mod tests;

use super::{Call, FactRecord, Journal, ReferenceRecord, Step};
use crate::store::{
    BackupDigest, LedgerError, PRIVATE, StoreError,
    capture::{
        acquire, child, exact, lock_target, owner_bytes, retain, same_directory, same_identity,
        target_for,
    },
    open_optional,
    payload::unchanged_file,
    read_optional,
};
use crate::store::{CapturePlan, operation::intent::Intent};
use ic_host_fs::{Directory, Kind as EntryKind, Lock};
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::File,
    io::{self, Seek, SeekFrom, Write},
};
use thiserror::Error;

/// Held capture approval and original files over the shared recorder.
pub type CaptureOperation = Recorder<CapturePlan>;
impl<P> Recorder<P> {
    /// Exact immutable original approval retained by this handle.
    #[must_use]
    pub fn plan(&self) -> &P {
        &self.intent
    }
}

#[derive(Clone, Copy)]
pub(in crate::store) enum Role {
    Request,
    Result,
    ObservationRequest,
    ObservationResult,
    ManagerLookupRequest,
    ManagerLookupResult,
}

impl Step {
    pub(in crate::store) fn name(self, role: Role) -> String {
        let step = match self {
            Self::InitialStatus => "initial-status".into(),
            Self::ListSnapshots => "list-snapshots".into(),
            Self::TakeSnapshot => "take-snapshot".into(),
            Self::Metadata => "metadata".into(),
            Self::Data(n) => format!("data-{n:08}"),
            Self::FinalStatus => "final-status".into(),
            Self::RestoreInitialStatus => "restore-initial-status".into(),
            Self::RestoreListSnapshots => "restore-list-snapshots".into(),
            Self::AllocateSnapshot => "allocate-snapshot".into(),
            Self::Upload(n) => format!("upload-{n:08}"),
            Self::ReadbackMetadata => "readback-metadata".into(),
            Self::Readback(n) => format!("readback-{n:08}"),
            Self::BeforeLoadStatus => "before-load-status".into(),
            Self::LoadSnapshot => "load-snapshot".into(),
            Self::AfterLoadStatus => "after-load-status".into(),
        };
        let role = match role {
            Role::Request => "request",
            Role::Result => "result",
            Role::ManagerLookupRequest => "manager-lookup-request",
            Role::ManagerLookupResult => "manager-lookup-result",
            Role::ObservationRequest => "observation-request",
            Role::ObservationResult => "observation-certificate",
        };
        format!("{step}-{role}.bin")
    }
}

impl ReferenceRecord {
    pub(in crate::store) fn new(bytes: &[u8]) -> Result<Self, OperationStoreError> {
        Ok(Self {
            bytes: bytes
                .len()
                .try_into()
                .map_err(|_| OperationStoreError::Changed)?,
            sha256: *BackupDigest::compute(bytes).as_bytes(),
        })
    }
}

impl Call {
    pub(in crate::store) fn reference(&self, role: Role) -> Option<&ReferenceRecord> {
        match role {
            Role::Request => self.update.request.as_ref(),
            Role::Result => self.update.result.as_ref(),
            Role::ManagerLookupRequest => self.manager_lookup.as_ref()?.request.as_ref(),
            Role::ManagerLookupResult => self.manager_lookup.as_ref()?.result.as_ref(),
            Role::ObservationRequest => self.observation.as_ref()?.request.as_ref(),
            Role::ObservationResult => self.observation.as_ref()?.result.as_ref(),
        }
    }
}

impl Journal {
    pub(in crate::store) fn evidence(&self) -> impl Iterator<Item = (String, &ReferenceRecord)> {
        self.calls.iter().flat_map(|call| {
            [
                Role::Request,
                Role::Result,
                Role::ObservationRequest,
                Role::ObservationResult,
                Role::ManagerLookupRequest,
                Role::ManagerLookupResult,
            ]
            .into_iter()
            .filter_map(move |role| call.reference(role).map(|r| (call.step.name(role), r)))
        })
    }
}

#[derive(Debug)]
/// Held original capture intent and files. Dropping releases locks, never target ownership.
pub struct Recorder<P> {
    pub(in crate::store) store: Directory,
    parent: Directory,
    pub(in crate::store) directory: Directory,
    target: Directory,
    target_lock_file: File,
    pub(in crate::store) intent: P,
    intent_file: File,
    pub(in crate::store) journal_file: File,
    lock: Lock,
    evidence: BTreeMap<String, File>,
    pub(in crate::store) usable: bool,
}

#[expect(
    private_bounds,
    reason = "the recorder is sealed to the two concrete original intent types"
)]
impl<P: Intent> Recorder<P> {
    pub(in crate::store) fn sync_evidence(&self) -> Result<(), OperationStoreError> {
        self.intent_file.sync_all()?;
        self.journal_file.sync_all()?;
        for file in self.evidence.values() {
            file.sync_all()?;
        }
        self.directory.sync()?;
        Ok(())
    }

    pub(in crate::store) fn create(
        store: &Directory,
        intent: P,
    ) -> Result<Self, OperationStoreError> {
        exact(store, "format", b"ic-backup/host-store/1\n", 128)?;
        let parent = child(store, "operations", false)?;
        let id = &intent.view().operation_id();
        if parent.open_directory(OsStr::new(id), PRIVATE)?.is_some() {
            return Err(OperationStoreError::Occupied);
        }
        let initial = Journal::create(&intent)?;
        let target = target_for(store, intent.view().target(), true)?;
        let (target_lock, created) = lock_target(&target)?;
        if !created || read_optional(&target, "owner.json", 1024)?.is_some() {
            return Err(OperationStoreError::Occupied);
        }
        retain(
            &target,
            "owner.json",
            &owner_bytes(id, intent.view().digest())?,
        )?;
        target.sync()?;
        Self::create_claimed(store, intent, target_lock, &initial)
    }

    pub(in crate::store) fn create_claimed(
        store: &Directory,
        intent: P,
        target_lock: Lock,
        initial: &[u8],
    ) -> Result<Self, OperationStoreError> {
        let parent = child(store, "operations", false)?;
        let id = &intent.view().operation_id();
        let directory = child(&parent, id, true)?;
        let file = directory.create_file(OsStr::new("lock"), 0o600)?;
        file.sync_all()?;
        let lock = acquire(file)?;
        retain(&directory, "intent.json", intent.view().encoded())?;
        retain(&directory, "journal.bin", initial)?;
        directory.sync()?;
        parent.sync()?;
        let recorder = Self::held(store, directory, intent, lock)?;
        recorder.check_target_lock(&target_lock)?;
        drop(target_lock);
        Ok(recorder)
    }

    pub(in crate::store) fn check_target_lock(
        &self,
        held: &Lock,
    ) -> Result<(), OperationStoreError> {
        if ic_host_fs::FileId::from_file(&self.target_lock_file)?
            != ic_host_fs::FileId::from_file(held.file())?
        {
            return Err(OperationStoreError::Changed);
        }
        Ok(())
    }

    // Selection is bounded and grants no custody or authority. The caller must
    // reopen under its established lock order and compare these exact bytes.
    pub(in crate::store) fn select_intent(
        store: &Directory,
        id: &str,
    ) -> Result<P, OperationStoreError> {
        if !crate::store::manifest::lower_hex(id, 32) {
            return Err(OperationStoreError::InvalidPlan);
        }
        exact(store, "format", b"ic-backup/host-store/1\n", 128)?;
        let parent = child(store, "operations", false)?;
        Self::read_intent(&child(&parent, id, false)?, id)
    }

    #[cfg(test)]
    pub(in crate::store) fn open(store: &Directory, id: &str) -> Result<Self, OperationStoreError> {
        Self::open_with_codec(store, id, None)
    }

    pub(in crate::store) fn open_with_codec(
        store: &Directory,
        id: &str,
        codec: Option<&'static dyn crate::store::RootCodec>,
    ) -> Result<Self, OperationStoreError> {
        if !crate::store::manifest::lower_hex(id, 32) {
            return Err(OperationStoreError::InvalidPlan);
        }
        exact(store, "format", b"ic-backup/host-store/1\n", 128)?;
        let parent = child(store, "operations", false)?;
        let directory = child(&parent, id, false)?;
        let lock =
            acquire(open_optional(&directory, "lock")?.ok_or(OperationStoreError::Incomplete)?)?;
        let mut intent = Self::read_intent(&directory, id)?;
        intent.attach_codec(codec);
        Self::held(store, directory, intent, lock)
    }

    fn read_intent(directory: &Directory, id: &str) -> Result<P, OperationStoreError> {
        let bytes = read_optional(
            directory,
            "intent.json",
            crate::store::capture::plan::PLAN_LIMIT,
        )?
        .ok_or(OperationStoreError::Incomplete)?;
        let intent = P::decode(&bytes)?;
        if intent.view().operation_id() != id {
            return Err(OperationStoreError::Changed);
        }
        Ok(intent)
    }

    fn held(
        store: &Directory,
        directory: Directory,
        intent: P,
        lock: Lock,
    ) -> Result<Self, OperationStoreError> {
        let parent = child(store, "operations", false)?;
        let target = target_for(store, intent.view().target(), false)?;
        let target_lock_file =
            open_optional(&target, "lock")?.ok_or(OperationStoreError::Incomplete)?;
        let intent_file =
            open_optional(&directory, "intent.json")?.ok_or(OperationStoreError::Incomplete)?;
        let journal_file =
            open_optional(&directory, "journal.bin")?.ok_or(OperationStoreError::Incomplete)?;
        let journal = Journal::open(&journal_file, &intent)?;
        let evidence = journal
            .evidence()
            .map(|(name, _)| {
                let file =
                    open_optional(&directory, &name)?.ok_or(OperationStoreError::Incomplete)?;
                Ok((name, file))
            })
            .collect::<Result<_, OperationStoreError>>()?;
        let recorder = Self {
            store: store.clone(),
            parent,
            directory,
            target,
            target_lock_file,
            intent,
            intent_file,
            journal_file,
            lock,
            evidence,
            usable: true,
        };
        recorder.check()?;
        Ok(recorder)
    }

    pub(in crate::store) fn check(&self) -> Result<Journal, OperationStoreError> {
        if !self.usable {
            return Err(OperationStoreError::Unusable);
        }
        exact(&self.store, "format", b"ic-backup/host-store/1\n", 128)?;
        same_directory(&self.store, "operations", &self.parent)?;
        same_identity(
            &target_for(&self.store, self.intent.view().target(), false)?,
            &self.target,
        )?;
        unchanged_file(
            &self.target,
            "lock",
            &self.target_lock_file,
            &self.target_lock_file.metadata()?,
        )?;
        exact(&self.target, "lock", b"", 0)?;
        for entry in self.target.entries(2)? {
            if !matches!(entry.name.to_str(), Some("lock" | "owner.json")) {
                return Err(OperationStoreError::Changed);
            }
        }
        same_directory(
            &self.parent,
            self.intent.view().operation_id(),
            &self.directory,
        )?;
        let intent = unchanged_file(
            &self.directory,
            "intent.json",
            &self.intent_file,
            &self.intent_file.metadata()?,
        )?;
        let bytes = ic_host_artifacts::read(
            intent,
            ic_host_artifacts::Length::AtMost(crate::store::capture::plan::PLAN_LIMIT as u64),
        )
        .map(|(bytes, _)| bytes)
        .map_err(crate::store::StoreError::from)?;
        if bytes != self.intent.view().encoded() {
            return Err(OperationStoreError::Changed);
        }
        unchanged_file(
            &self.directory,
            "lock",
            self.lock.file(),
            &self.lock.file().metadata()?,
        )?;
        exact(&self.directory, "lock", b"", 0)?;
        let file = unchanged_file(
            &self.directory,
            "journal.bin",
            &self.journal_file,
            &self.journal_file.metadata()?,
        )?;
        let journal = Journal::open(file, &self.intent)?;
        P::check_layout(self, &journal)?;
        let mut evidence_count = 0;
        for (name, reference) in journal.evidence() {
            self.read(&name, reference)?;
            evidence_count += 1;
        }
        if self.evidence.len() != evidence_count {
            return Err(OperationStoreError::Changed);
        }
        for entry in self.directory.entries(evidence_count + 3)? {
            if entry.kind != EntryKind::File
                || !entry.name.to_str().is_some_and(|name| {
                    matches!(name, "lock" | "intent.json" | "journal.bin")
                        || self.evidence.contains_key(name)
                })
            {
                return Err(OperationStoreError::Changed);
            }
        }
        let expected = owner_bytes(
            self.intent.view().operation_id(),
            self.intent.view().digest(),
        )?;
        if read_optional(&self.target, "owner.json", 1024)?.as_deref() != Some(expected.as_slice())
            && !P::published_binding(self, &journal)?
        {
            return Err(OperationStoreError::Changed);
        }
        Ok(journal)
    }

    pub(in crate::store) fn read(
        &self,
        name: &str,
        reference: &ReferenceRecord,
    ) -> Result<Vec<u8>, OperationStoreError> {
        let held = self
            .evidence
            .get(name)
            .ok_or(OperationStoreError::Incomplete)?;
        let before = held.metadata()?;
        let file = unchanged_file(&self.directory, name, held, &before)?;
        let (bytes, identity) = ic_host_artifacts::read(
            file,
            ic_host_artifacts::Length::AtMost(u64::from(reference.bytes)),
        )
        .map_err(crate::store::StoreError::from)?;
        if identity.bytes != u64::from(reference.bytes) || identity.sha256 != reference.sha256 {
            return Err(OperationStoreError::Changed);
        }
        unchanged_file(&self.directory, name, held, &before)?;
        Ok(bytes)
    }

    pub(in crate::store) fn append(
        &mut self,
        original: &Journal,
        fact: FactRecord,
        evidence: Option<&[u8]>,
    ) -> Result<Journal, OperationStoreError> {
        self.append_with(original, fact, evidence, |file, frame| {
            file.write_all(frame)?;
            file.sync_all()
        })
    }

    pub(in crate::store) fn finish_write(
        &mut self,
        expected: &Journal,
    ) -> Result<(), OperationStoreError> {
        self.usable = true;
        let result = self.check().and_then(|actual| {
            if actual == *expected {
                Ok(())
            } else {
                Err(OperationStoreError::Changed)
            }
        });
        self.usable = result.is_ok();
        result
    }

    fn append_with(
        &mut self,
        original: &Journal,
        fact: FactRecord,
        evidence: Option<&[u8]>,
        write: impl FnOnce(&mut File, &[u8]) -> io::Result<()>,
    ) -> Result<Journal, OperationStoreError> {
        let reference = match &fact {
            FactRecord::CallPrepared { step, request } => Some((step.name(Role::Request), request)),
            FactRecord::CallResult { step, result } => Some((step.name(Role::Result), result)),
            FactRecord::ManagerLookupPrepared { step, request } => {
                Some((step.name(Role::ManagerLookupRequest), request))
            }
            FactRecord::ManagerLookupResult { step, result } => {
                Some((step.name(Role::ManagerLookupResult), result))
            }
            FactRecord::ObservationPrepared { step, request } => {
                Some((step.name(Role::ObservationRequest), request))
            }
            FactRecord::ObservationResult { step, certificate } => {
                Some((step.name(Role::ObservationResult), certificate))
            }
            _ => None,
        };
        let retained = match (reference, evidence) {
            (Some((name, reference)), Some(bytes))
                if *reference == ReferenceRecord::new(bytes)? =>
            {
                Some((name, bytes))
            }
            (None, None) => None,
            _ => return Err(OperationStoreError::Changed),
        };
        let frame = original.candidate(fact)?;
        let mut file = self
            .directory
            .open_file(
                OsStr::new("journal.bin"),
                ic_host_fs::Access::Update,
                crate::store::FILE_POLICY,
            )?
            .ok_or(OperationStoreError::Incomplete)?;
        if ic_host_fs::FileId::from_file(&file)?
            != ic_host_fs::FileId::from_file(&self.journal_file)?
        {
            return Err(OperationStoreError::Changed);
        }
        if self.check()? != *original {
            return Err(OperationStoreError::Changed);
        }
        self.usable = false;
        file.sync_all()?;
        for file in self.evidence.values() {
            file.sync_all()?;
        }
        self.directory.sync()?;
        if let Some((name, bytes)) = retained {
            retain(&self.directory, &name, bytes)?;
            self.directory.sync()?;
            let file =
                open_optional(&self.directory, &name)?.ok_or(OperationStoreError::Incomplete)?;
            self.evidence.insert(name, file);
        }
        file.seek(SeekFrom::Start(original.bytes))?;
        write(&mut file, &frame)?;
        self.usable = true;
        let checked = self.check().and_then(|actual| {
            if actual.bytes == original.bytes + frame.len() as u64
                && actual.head.as_bytes().as_slice() == &frame[frame.len() - 32..]
            {
                Ok(actual)
            } else {
                Err(OperationStoreError::Changed)
            }
        });
        self.usable = checked.is_ok();
        checked
    }

    // Synthetic histories and fault writers use the same closing admission as
    // production; only fixtures request arbitrary facts without a procedure.
    #[cfg(test)]
    pub(in crate::store) fn append_fixture(
        &mut self,
        fact: FactRecord,
        evidence: Option<&[u8]>,
    ) -> Result<Journal, OperationStoreError> {
        let original = self.check()?;
        self.append(&original, fact, evidence)
    }

    #[cfg(test)]
    fn append_fixture_with(
        &mut self,
        fact: FactRecord,
        evidence: Option<&[u8]>,
        write: impl FnOnce(&mut File, &[u8]) -> io::Result<()>,
    ) -> Result<Journal, OperationStoreError> {
        let original = self.check()?;
        self.append_with(&original, fact, evidence, write)
    }
}

/// Original operation admission or local persistence refusal. No error authorizes retrying effects.
#[derive(Debug, Error)]
pub enum OperationStoreError {
    /// Invalid identity, encoding generation or finite limits.
    #[error("invalid operation plan")]
    InvalidPlan,
    /// Caller-selected approval differs from retained exact intent.
    #[error("operation plan approval mismatch")]
    ApprovalMismatch,
    /// Target or operation already has retained evidence.
    #[error("operation target or identity is occupied")]
    Occupied,
    /// Another process/handle currently holds exclusion.
    #[error("operation is locked")]
    Locked,
    /// Missing original evidence; opening never repairs it.
    #[error("operation intent is incomplete")]
    Incomplete,
    /// Original record bytes, ledger or namespace no longer agree.
    #[error("operation intent or custody changed")]
    Changed,
    /// Original live handle encountered uncertain persistence; reopen for diagnosis.
    #[error("operation session is unusable after uncertain persistence")]
    Unusable,
    /// Invalid derived admission or exhausted original call allowance.
    #[error("call admission is invalid or exceeds original authority")]
    StatusAdmission,
    /// Bounded framing, exact chain or supported-transition admission failed.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Strict JSON admission/encoding failure.
    #[error("invalid operation record encoding")]
    Encoding(#[from] serde_json::Error),
    /// Private store and bounded file admission.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Held-directory admission failure.
    #[error(transparent)]
    Filesystem(#[from] ic_host_fs::Error),
    /// Original native IO failure; all partial files remain retained.
    #[error(transparent)]
    Io(#[from] io::Error),
}
