use super::*;
use std::num::NonZeroU64;
mod source_tests;

struct Fixture {
    store: BackupStore,
    folder: PathBuf,
    record: ManifestRecord,
    commitment: Sha256Digest,
}

impl Fixture {
    fn new(label: &str) -> Self {
        Self::with_chunks(label, vec![Vec::new(), b"stored chunk".to_vec()])
    }

    fn with_chunks(label: &str, chunks: Vec<Vec<u8>>) -> Self {
        Self::with_regions(
            label,
            chunks,
            b"\0asm\x01\0\0\0".to_vec(),
            Vec::new(),
            vec![0xab; 32_769],
        )
    }

    fn with_regions(
        label: &str,
        chunks: Vec<Vec<u8>>,
        module: Vec<u8>,
        heap: Vec<u8>,
        stable: Vec<u8>,
    ) -> Self {
        let path = root(label);
        let store = BackupStore::initialize(&path).unwrap();
        let mut record = record(1, 100, 200);
        let mut contents = vec![
            ("module.wasm".to_owned(), module),
            ("heap.bin".to_owned(), heap),
            ("stable.bin".to_owned(), stable),
        ];
        // An empty chunk is a real named file, distinct from an empty region.
        for bytes in chunks {
            let name = format!("chunk-{}.bin", Sha256Digest::compute(&bytes));
            contents.push((name, bytes));
        }
        record.members = contents
            .iter()
            .map(|(name, bytes)| MemberRecord {
                name: name.clone(),
                bytes: bytes.len() as u64,
                sha256: Sha256Digest::compute(bytes).to_string(),
            })
            .collect();
        record.module_sha256 = record.members[0].sha256.clone();
        record.payload_bytes = record.members.iter().map(|m| m.bytes).sum();
        let commitment = Sha256Digest::compute(&serde_json::to_vec(&record).unwrap());
        let folder = bundle(&path, &record);
        let payload = folder.join("payload");
        DirBuilder::new().mode(0o700).create(&payload).unwrap();
        for (name, bytes) in contents {
            write_file(&payload.join(name), &bytes);
        }
        Self {
            store,
            folder,
            record,
            commitment,
        }
    }

    fn verify(&self) -> Result<BackupSummary, StoreError> {
        self.store.verify_payload(
            &self.record.operation_id,
            self.commitment,
            NonZeroU64::new(self.record.payload_bytes).unwrap(),
        )
    }
}

#[test]
fn checks_all_regions_and_chunks_at_exact_budget_without_operation_evidence() {
    let fixture = Fixture::new("verified-payload");
    let summary = fixture.verify().unwrap();
    assert_eq!(summary.manifest_sha256, fixture.commitment);
    assert_eq!(summary.payload_bytes, 32_789);
    assert_eq!(
        summary,
        fixture.store.list_backups(None, limits()).unwrap()[0]
    );
    assert!(
        fixture
            .folder
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("operations")
            .read_dir()
            .unwrap()
            .next()
            .is_none()
    );
    assert_eq!(fixture.folder.read_dir().unwrap().count(), 2);
    assert_eq!(
        fs::read(fixture.folder.join("payload/heap.bin")).unwrap(),
        b""
    );
}

#[test]
fn corruption_is_distinct_from_listing_and_never_repaired() {
    let fixture = Fixture::new("corrupt-payload");
    let file = fixture.folder.join("payload/module.wasm");
    fs::write(&file, b"bad wasm").unwrap();
    let err = fixture.verify().unwrap_err();
    assert!(matches!(err,
        StoreError::Payload { member, source: ArtifactError::DigestMismatch { .. } }
        if member == "module.wasm"));
    assert_eq!(fs::read(file).unwrap(), b"bad wasm");
    assert_eq!(fixture.store.list_backups(None, limits()).unwrap().len(), 1);
}

#[test]
fn short_and_grown_files_return_exact_length_failures() {
    for (label, bytes) in [
        ("short-payload", b"short".as_slice()),
        ("long-payload", b"too long wasm".as_slice()),
    ] {
        let fixture = Fixture::new(label);
        let file = fixture.folder.join("payload/module.wasm");
        fs::write(&file, bytes).unwrap();
        match fixture.verify().unwrap_err() {
            StoreError::Payload { member, source } => {
                assert_eq!(member, "module.wasm");
                if bytes.len() < 8 {
                    assert!(matches!(
                        source,
                        ArtifactError::LengthMismatch {
                            expected: 8,
                            actual: 5
                        }
                    ));
                } else {
                    assert!(matches!(source, ArtifactError::LimitExceeded { limit: 8 }));
                }
            }
            other => panic!("unexpected refusal: {other:?}"),
        }
        assert_eq!(fs::read(file).unwrap(), bytes);
    }
}

#[test]
fn manifest_commitment_and_budget_are_admitted_before_payload_access() {
    let fixture = Fixture::new("payload-admission");
    fs::rename(
        fixture.folder.join("payload"),
        fixture.folder.join("retained-payload"),
    )
    .unwrap();
    let wrong = Sha256Digest::compute(b"untrusted replacement");
    assert!(matches!(fixture.store.verify_payload(
        &fixture.record.operation_id, wrong, NonZeroU64::new(1).unwrap()
    ), Err(StoreError::Read(ArtifactError::DigestMismatch { expected, actual }))
        if expected == wrong && actual.sha256 == fixture.commitment));
    assert!(matches!(
        fixture.store.verify_payload(
            &fixture.record.operation_id,
            fixture.commitment,
            NonZeroU64::new(1).unwrap()
        ),
        Err(StoreError::PayloadLimit {
            declared: 32_789,
            limit: 1
        })
    ));
    assert!(matches!(fixture.verify(), Err(StoreError::InvalidLayout)));
    assert!(!fixture.folder.join("payload").exists());
}

#[test]
fn missing_extra_nested_linked_and_nonprivate_payload_members_refuse() {
    for case in [
        "missing",
        "extra",
        "nested",
        "symlink",
        "hardlink",
        "permissions",
    ] {
        let fixture = Fixture::new(case);
        let payload = fixture.folder.join("payload");
        let module = payload.join("module.wasm");
        let retained = fixture.folder.join("retained-module");
        match case {
            "extra" => write_file(&payload.join("unknown.bin"), b"unexpected"),
            "hardlink" => fs::hard_link(&module, &retained).unwrap(),
            "permissions" => {
                fs::set_permissions(&module, fs::Permissions::from_mode(0o644)).unwrap()
            }
            _ => {
                fs::rename(&module, &retained).unwrap();
                match case {
                    "nested" => DirBuilder::new().mode(0o700).create(&module).unwrap(),
                    "symlink" => symlink(&retained, &module).unwrap(),
                    "missing" => {}
                    _ => unreachable!(),
                }
            }
        }
        let error = fixture.verify().unwrap_err();
        match case {
            "permissions" => assert!(matches!(
                error,
                StoreError::Filesystem(CapabilityError::Permissions { .. })
            )),
            "extra" => assert!(matches!(
                error,
                StoreError::InvalidLayout
                    | StoreError::Filesystem(CapabilityError::EntryLimit { .. })
            )),
            _ => assert!(matches!(error, StoreError::InvalidLayout)),
        }
        if retained.exists() {
            assert_eq!(fs::read(retained).unwrap(), b"\0asm\x01\0\0\0");
        }
    }
}

#[test]
fn payload_directory_substitution_and_unsafe_ids_refuse() {
    let fixture = Fixture::new("payload-directory-link");
    let payload = fixture.folder.join("payload");
    let original = fixture.folder.join("retained-payload");
    fs::rename(&payload, &original).unwrap();
    symlink(&original, &payload).unwrap();
    assert!(matches!(
        fixture.verify(),
        Err(StoreError::Filesystem(CapabilityError::Refused { .. }))
    ));
    for id in [
        "../escape",
        "",
        "/outside",
        "0000000000000000000000000000000A",
    ] {
        assert!(matches!(
            fixture
                .store
                .verify_payload(id, fixture.commitment, NonZeroU64::new(1).unwrap()),
            Err(StoreError::InvalidLayout)
        ));
    }
    assert_eq!(
        fs::read(original.join("module.wasm")).unwrap(),
        b"\0asm\x01\0\0\0"
    );
}

#[test]
fn a_self_consistent_replacement_cannot_change_the_trusted_manifest() {
    let mut fixture = Fixture::new("replaced-manifest");
    let module = fixture.folder.join("payload/module.wasm");
    fs::write(module, b"modified").unwrap();
    let replacement = Sha256Digest::compute(b"modified").to_string();
    fixture.record.module_sha256 = replacement.clone();
    fixture.record.members[0].sha256 = replacement;
    fs::write(
        fixture.folder.join("manifest.json"),
        serde_json::to_vec(&fixture.record).unwrap(),
    )
    .unwrap();
    assert!(
        matches!(fixture.verify(), Err(StoreError::Read(ArtifactError::DigestMismatch { expected, .. }))
        if expected == fixture.commitment)
    );
    let listed = fixture.store.list_backups(None, limits()).unwrap();
    assert_ne!(listed[0].manifest_sha256, fixture.commitment);
}

#[test]
fn verification_stays_with_the_held_store_after_root_replacement() {
    let fixture = Fixture::new("verify-held-root");
    let path = fixture.folder.parent().unwrap().parent().unwrap();
    let moved = path.with_extension("retained");
    fs::rename(path, &moved).unwrap();
    DirBuilder::new().mode(0o700).create(path).unwrap();
    BackupStore::initialize(path).unwrap();
    assert_eq!(
        fixture.verify().unwrap().manifest_sha256,
        fixture.commitment
    );
    assert!(
        BackupStore::open(path)
            .unwrap()
            .list_backups(None, limits())
            .unwrap()
            .is_empty()
    );
}

// Re-authored from legacy verification/tests/mod.rs::
// callback_reads_reject_same_length_replacement_and_retain_both_files,
// and upload/plan closing-scan cases at 9f7b886a824eec67dda8f81792a560436419bb2a.
// See docs/architecture/local-inventory.md for source/history and changed assumptions.
#[test]
fn legacy_same_bytes_file_replacement_during_read_is_refused() {
    let fixture = Fixture::new("legacy-file-replacement");
    let path = fixture.folder.join("payload/module.wasm");
    let retained = fixture.folder.join("retained-module");
    let mut changed = false;
    let result = fixture.store.verify_payload_with(
        &fixture.record.operation_id,
        fixture.commitment,
        NonZeroU64::new(fixture.record.payload_bytes).unwrap(),
        |file, length| {
            let identity = ic_host_artifacts::artifact::hash_exact_reader(file, length)?;
            if !changed {
                fs::rename(&path, &retained)?;
                write_file(&path, b"\0asm\x01\0\0\0");
                changed = true;
            }
            Ok(identity)
        },
    );
    assert!(changed);
    assert!(
        matches!(result, Err(StoreError::CustodyChanged)),
        "{result:?}"
    );
    assert_eq!(fs::read(path).unwrap(), b"\0asm\x01\0\0\0");
    assert_eq!(fs::read(retained).unwrap(), b"\0asm\x01\0\0\0");
}

#[test]
fn legacy_extra_entry_after_opening_scan_is_refused_without_cleanup() {
    let fixture = Fixture::new("legacy-extra-after-scan");
    let extra = fixture.folder.join("payload/extra.bin");
    let mut changed = false;
    let result = fixture.store.verify_payload_with(
        &fixture.record.operation_id,
        fixture.commitment,
        NonZeroU64::new(fixture.record.payload_bytes).unwrap(),
        |file, length| {
            let identity = ic_host_artifacts::artifact::hash_exact_reader(file, length)?;
            if !changed {
                write_file(&extra, b"retained unexpected bytes");
                changed = true;
            }
            Ok(identity)
        },
    );
    assert!(
        matches!(
            result,
            Err(StoreError::InvalidLayout
                | StoreError::Filesystem(CapabilityError::EntryLimit { .. }))
        ),
        "{result:?}"
    );
    assert_eq!(fs::read(extra).unwrap(), b"retained unexpected bytes");
}

#[test]
fn changed_manifest_during_payload_read_cannot_return_original_success() {
    for replace in [false, true] {
        let fixture = Fixture::new("legacy-manifest-drift");
        let path = fixture.folder.join("manifest.json");
        let original = fs::read(&path).unwrap();
        let mut changed = false;
        let result = fixture.store.verify_payload_with(
            &fixture.record.operation_id,
            fixture.commitment,
            NonZeroU64::new(fixture.record.payload_bytes).unwrap(),
            |file, length| {
                let identity = ic_host_artifacts::artifact::hash_exact_reader(file, length)?;
                if !changed {
                    if replace {
                        fs::rename(&path, fixture.folder.join("retained-manifest"))?;
                        write_file(&path, &original);
                    } else {
                        let mut bytes = original.clone();
                        // Same inode and length, different capture timestamp.
                        let text = std::str::from_utf8(&bytes).unwrap();
                        let start = text.find("\"captured_at_ns\":100").unwrap();
                        bytes[start + "\"captured_at_ns\":".len()] = b'2';
                        fs::write(&path, bytes)?;
                    }
                    changed = true;
                }
                Ok(identity)
            },
        );
        if replace {
            assert!(
                matches!(result, Err(StoreError::CustodyChanged)),
                "{result:?}"
            );
            assert_eq!(
                fs::read(fixture.folder.join("retained-manifest")).unwrap(),
                original
            );
        } else {
            assert!(
                matches!(
                    result,
                    Err(StoreError::Read(ArtifactError::DigestMismatch { .. }))
                ),
                "{result:?}"
            );
            assert_ne!(fs::read(&path).unwrap(), original);
        }
    }
}

#[test]
fn legacy_replaced_child_directories_during_read_are_refused() {
    for child in ["payload", "bundle", "bundles"] {
        let fixture = Fixture::new("legacy-directory-drift");
        let path = match child {
            "payload" => fixture.folder.join("payload"),
            "bundle" => fixture.folder.clone(),
            "bundles" => fixture.folder.parent().unwrap().to_owned(),
            _ => unreachable!(),
        };
        let retained = path.with_extension("retained");
        let mut changed = false;
        let result = fixture.store.verify_payload_with(
            &fixture.record.operation_id,
            fixture.commitment,
            NonZeroU64::new(fixture.record.payload_bytes).unwrap(),
            |file, length| {
                let identity = ic_host_artifacts::artifact::hash_exact_reader(file, length)?;
                if !changed {
                    fs::rename(&path, &retained)?;
                    DirBuilder::new().mode(0o700).create(&path)?;
                    changed = true;
                }
                Ok(identity)
            },
        );
        assert!(
            matches!(result, Err(StoreError::CustodyChanged)),
            "{child}: {result:?}"
        );
        assert_eq!(fs::read_dir(path).unwrap().count(), 0);
        let payload = match child {
            "payload" => retained,
            "bundle" => retained.join("payload"),
            "bundles" => retained.join(&fixture.record.operation_id).join("payload"),
            _ => unreachable!(),
        };
        assert_eq!(
            fs::read(payload.join("module.wasm")).unwrap(),
            b"\0asm\x01\0\0\0"
        );
    }
}

#[test]
fn legacy_extent_change_after_hash_is_refused() {
    let fixture = Fixture::new("legacy-closing-extent");
    let path = fixture.folder.join("payload/module.wasm");
    let mut changed = false;
    let result = fixture.store.verify_payload_with(
        &fixture.record.operation_id,
        fixture.commitment,
        NonZeroU64::new(fixture.record.payload_bytes).unwrap(),
        |file, length| {
            let identity = ic_host_artifacts::artifact::hash_exact_reader(file, length)?;
            if !changed {
                OpenOptions::new()
                    .append(true)
                    .open(&path)?
                    .write_all(b"x")?;
                changed = true;
            }
            Ok(identity)
        },
    );
    assert!(
        matches!(result, Err(StoreError::CustodyChanged)),
        "{result:?}"
    );
    assert_eq!(fs::read(path).unwrap(), b"\0asm\x01\0\0\0x");
}

// Adapted from legacy tests/verification::
// maximum_chunks_and_region_bytes_verify_without_an_aggregate_buffer.
#[test]
fn legacy_maximum_chunk_count_and_chunk_size_are_accepted() {
    let mut chunks: Vec<_> = (0_u16..1023).map(|i| i.to_be_bytes().to_vec()).collect();
    chunks.push(vec![0x5a; 1024 * 1024]);
    let fixture = Fixture::with_chunks("legacy-max-chunks", chunks);
    let summary = fixture.verify().unwrap();
    assert_eq!(summary.payload_bytes, 1_083_399);
    assert_eq!(
        fixture.folder.join("payload").read_dir().unwrap().count(),
        1027
    );
    assert_eq!(
        fs::read(fixture.folder.join("manifest.json")).unwrap(),
        serde_json::to_vec(&fixture.record).unwrap()
    );
}

// Legacy TreeFault::Fifo, adapted to replacement at closing file admission.
#[test]
fn legacy_fifo_replacement_after_hash_refuses_and_retains_original() {
    use std::os::unix::fs::FileTypeExt as _;
    let fixture = Fixture::new("legacy-closing-fifo");
    let path = fixture.folder.join("payload/module.wasm");
    let retained = fixture.folder.join("retained-module");
    let mut changed = false;
    let result = fixture.store.verify_payload_with(
        &fixture.record.operation_id,
        fixture.commitment,
        NonZeroU64::new(fixture.record.payload_bytes).unwrap(),
        |file, length| {
            let identity = ic_host_artifacts::artifact::hash_exact_reader(file, length)?;
            if !changed {
                fs::rename(&path, &retained)?;
                let status = std::process::Command::new("mkfifo")
                    .args(["-m", "600"])
                    .arg(&path)
                    .status()?;
                assert!(status.success(), "native FIFO fixture creation failed");
                changed = true;
            }
            Ok(identity)
        },
    );
    assert!(
        matches!(
            result,
            Err(StoreError::Filesystem(CapabilityError::Refused { .. }))
        ),
        "{result:?}"
    );
    assert!(fs::symlink_metadata(path).unwrap().file_type().is_fifo());
    assert_eq!(fs::read(retained).unwrap(), b"\0asm\x01\0\0\0");
}
