//! Read-only payload checking; no operation replay or restore admission.
mod source;
pub use source::{PayloadReadError, PayloadSource};

use super::{
    BackupStore, BackupSummary, MANIFEST_LIMIT, PRIVATE, StoreError, manifest, open_optional,
};
use ic_host_artifacts::artifact::{ArtifactIdentity, Sha256Digest, hash_exact_reader};
use ic_host_fs::{
    capability::{Directory, EntryKind},
    identity::FileIdentity,
    read::read_opened_file,
};
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs::{File, Metadata},
    num::NonZeroU64,
};

impl BackupStore {
    /// Freshly check every payload member against a caller-trusted manifest commitment.
    ///
    /// Validates the bundle ID and bounded manifest, then admits the exact flat
    /// payload member set and hashes each private regular file through held
    /// directories. Rechecks file identity/length after each read and member set,
    /// child-directory identity and original manifest before success. Root rename
    /// keeps its existing held-store meaning; changed children refuse. These
    /// sequential checks cannot fence noncooperating writers.
    /// Streaming uses constant scratch space; the manifest bounds
    /// member count. The caller's byte limit applies to total declared payload,
    /// independently of the 1 MiB manifest cap. A failing file read may probe one
    /// byte past its declaration to detect growth, then stops without a result.
    ///
    /// The caller excludes namespace/content writers throughout this call and
    /// supplies a manifest hash from trusted original custody. A hash freshly
    /// obtained from inventory alone supplies no authenticated provenance.
    /// Success returns the manifest summary after checking bytes; it holds no
    /// files for future use and does not make them immutable. Any later use must
    /// admit them again under its own custody.
    ///
    /// This does not inspect plan/ledger/metadata evidence or source references,
    /// replay capture, sync files, publish completion or authorize restore. There
    /// are no writes, repairs or network calls. Full bundle verification remains
    /// a separate prerequisite for restore.
    ///
    /// ```no_run
    /// use ic_backup::store::BackupStore;
    /// use std::{num::NonZeroU64, path::Path};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = BackupStore::open(Path::new("/private/backups"))?;
    /// // Illustrative commitment: supply the hash from trusted original custody.
    /// let trusted_manifest = "11".repeat(32).parse()?;
    /// let summary = store.verify_payload(
    ///     "00000000000000000000000000000001",
    ///     trusted_manifest,
    ///     NonZeroU64::new(1024 * 1024 * 1024).unwrap(),
    /// )?;
    /// // This checks payload bytes, not the operation evidence needed for restore.
    /// # let _ = summary;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Refuses invalid/missing bundle structure, manifest commitment or schema
    /// mismatch, a declared payload above the caller limit, extra/missing payload
    /// members, unsafe files and exact-length/hash failures. No partial success.
    pub fn verify_payload(
        &self,
        bundle_id: &str,
        expected_manifest: Sha256Digest,
        maximum_payload_bytes: NonZeroU64,
    ) -> Result<BackupSummary, StoreError> {
        self.verify_payload_with(
            bundle_id,
            expected_manifest,
            maximum_payload_bytes,
            |file, length| hash_exact_reader(file, length),
        )
    }

    // Share the production read and closing admission path with deterministic
    // native fault tests. This is not a public reader or provider extension.
    pub(super) fn verify_payload_with(
        &self,
        bundle_id: &str,
        expected_manifest: Sha256Digest,
        maximum_payload_bytes: NonZeroU64,
        mut read: impl FnMut(&mut File, u64) -> Result<ArtifactIdentity, super::ArtifactError>,
    ) -> Result<BackupSummary, StoreError> {
        let original =
            PayloadDirectory::open(self, bundle_id, expected_manifest, maximum_payload_bytes)?;
        original.verify_with(|file, length| read(file, length).map(|identity| (identity, ())))?;
        Ok(original.decoded.summary)
    }
}

// Both one-shot verification and held readers use this exact admission/closing owner.
struct PayloadDirectory<'store> {
    store: &'store BackupStore,
    bundles: Directory,
    bundle: Directory,
    payload: Directory,
    manifest_file: File,
    original_manifest: Metadata,
    decoded: manifest::DecodedManifest,
}

impl<'store> PayloadDirectory<'store> {
    fn open(
        store: &'store BackupStore,
        bundle_id: &str,
        expected_manifest: Sha256Digest,
        maximum_payload_bytes: NonZeroU64,
    ) -> Result<Self, StoreError> {
        if !manifest::lower_hex(bundle_id, 32) {
            return Err(StoreError::InvalidLayout);
        }
        let bundles = store
            .root
            .open_directory(OsStr::new("bundles"), PRIVATE)?
            .ok_or(StoreError::IncompleteStore)?;
        let bundle = bundles
            .open_directory(OsStr::new(bundle_id), PRIVATE)?
            .ok_or(StoreError::InvalidLayout)?;
        let manifest_file =
            open_optional(&bundle, "manifest.json")?.ok_or(StoreError::InvalidLayout)?;
        let original_manifest = manifest_file.metadata()?;
        let bytes = read_opened_file(manifest_file.try_clone()?, MANIFEST_LIMIT)?;
        ArtifactIdentity {
            bytes: bytes.len() as u64,
            sha256: Sha256Digest::compute(&bytes),
        }
        .verify(expected_manifest)?;
        let decoded =
            manifest::decode(bundle_id, &bytes).map_err(|source| StoreError::Manifest {
                bundle: bundle_id.into(),
                source,
            })?;
        if decoded.summary.payload_bytes > maximum_payload_bytes.get() {
            return Err(StoreError::PayloadLimit {
                declared: decoded.summary.payload_bytes,
                limit: maximum_payload_bytes.get(),
            });
        }
        let payload = bundle
            .open_directory(OsStr::new("payload"), PRIVATE)?
            .ok_or(StoreError::InvalidLayout)?;
        check_members(&payload, &decoded.members)?;
        Ok(Self {
            store,
            bundles,
            bundle,
            payload,
            manifest_file,
            original_manifest,
            decoded,
        })
    }

    fn verify_with<T>(
        &self,
        mut read: impl FnMut(&mut File, u64) -> Result<(ArtifactIdentity, T), super::ArtifactError>,
    ) -> Result<Vec<T>, StoreError> {
        let mut retained = Vec::with_capacity(self.decoded.members.len());
        for member in &self.decoded.members {
            let mut file =
                open_optional(&self.payload, &member.name)?.ok_or(StoreError::InvalidLayout)?;
            let original = file.metadata()?;
            let expected = member
                .sha256
                .parse()
                .map_err(|_| StoreError::InvalidLayout)?;
            let value = read(&mut file, member.bytes)
                .and_then(|(identity, value)| identity.verify(expected).map(|_| value))
                .map_err(|source| StoreError::Payload {
                    member: member.name.clone(),
                    source,
                })?;
            unchanged_file(&self.payload, &member.name, &file, &original)?;
            retained.push(value);
        }
        self.check()?;
        Ok(retained)
    }

    fn check(&self) -> Result<(), StoreError> {
        check_members(&self.payload, &self.decoded.members)?;
        let current_manifest = unchanged_file(
            &self.bundle,
            "manifest.json",
            &self.manifest_file,
            &self.original_manifest,
        )?;
        hash_exact_reader(current_manifest, self.original_manifest.len())?
            .verify(self.decoded.summary.manifest_sha256)?;
        unchanged_directory(&self.bundle, "payload", &self.payload)?;
        unchanged_directory(&self.bundles, &self.decoded.summary.bundle_id, &self.bundle)?;
        unchanged_directory(&self.store.root, "bundles", &self.bundles)?;
        Ok(())
    }
}

fn check_members(
    directory: &Directory,
    members: &[manifest::MemberRecord],
) -> Result<(), StoreError> {
    let mut names: BTreeSet<_> = members.iter().map(|m| m.name.as_str()).collect();
    for entry in directory.entries(names.len())? {
        let entry = entry?;
        if entry.kind != EntryKind::File
            || !entry.name.to_str().is_some_and(|name| names.remove(name))
        {
            return Err(StoreError::InvalidLayout);
        }
    }
    if !names.is_empty() {
        return Err(StoreError::InvalidLayout);
    }
    Ok(())
}

// Keep the original descriptor alive through the name comparison, preventing
// inode reuse from making a removed/recreated file look like the original.
fn unchanged_file(
    parent: &Directory,
    name: &str,
    held: &File,
    original: &Metadata,
) -> Result<File, StoreError> {
    let current = open_optional(parent, name)?.ok_or(StoreError::CustodyChanged)?;
    let observed = current.metadata()?;
    if FileIdentity::from_metadata(original) != FileIdentity::from_metadata(&observed)
        || original.len() != observed.len()
        || original.len() != held.metadata()?.len()
    {
        return Err(StoreError::CustodyChanged);
    }
    Ok(current)
}

fn unchanged_directory(parent: &Directory, name: &str, held: &Directory) -> Result<(), StoreError> {
    let current = parent
        .open_directory(OsStr::new(name), PRIVATE)?
        .ok_or(StoreError::CustodyChanged)?;
    if FileIdentity::from_metadata(&held.open_syncable()?.metadata()?)
        != FileIdentity::from_metadata(&current.open_syncable()?.metadata()?)
    {
        return Err(StoreError::CustodyChanged);
    }
    Ok(())
}
