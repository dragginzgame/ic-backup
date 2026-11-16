//! Native held-source regressions; synthetic bytes do not qualify IC effects.
use super::*;
use ic_platform::{
    candid_codec::Reserved,
    management_canister_types::{
        CanisterTimer, ChunkHash, OnLowWasmMemoryHookStatus, ReadCanisterSnapshotDataArgs,
        ReadCanisterSnapshotMetadataResult, SnapshotDataKind, SnapshotSource,
    },
};

impl Fixture {
    fn source(&self) -> PayloadSource<'_> {
        self.store
            .open_payload(
                &self.record.operation_id,
                self.commitment,
                NonZeroU64::new(self.record.payload_bytes).unwrap(),
            )
            .unwrap()
    }
    fn request(&self, kind: SnapshotDataKind) -> ReadCanisterSnapshotDataArgs {
        ReadCanisterSnapshotDataArgs {
            canister_id: ic_platform::Principal::from_text(&self.record.canister).unwrap(),
            snapshot_id: vec![1, 2],
            kind,
        }
    }
}

#[test]
fn source_reads_follow_exact_original_schedule_without_reopening_by_absolute_path() {
    let fixture = Fixture::new("source-readback");
    let before = fs::read(fixture.folder.join("manifest.json")).unwrap();
    let mut source = fixture.source();
    assert_eq!(*source.summary(), fixture.verify().unwrap());
    let mut request =
        crate::store::CapturePolicy::decode(include_bytes!("../fixtures/capture-plan.json"))
            .unwrap()
            .request()
            .clone();
    request.target = source.summary().target;
    request.release = source.summary().release;
    request.module_sha256 = source.summary().module_sha256;
    request.limits.extent_bytes = 4096;
    request.limits.payload_bytes = 1 << 30;
    request.limits.network_requests = 8192;
    request.limits.signing_requests = 8192;
    let plan = CapturePolicy::new(request).unwrap();
    let metadata = ReadCanisterSnapshotMetadataResult {
        source: Some(SnapshotSource::TakenFromCanister(Reserved)),
        taken_at_timestamp: 100,
        canister_version: 1,
        wasm_module_size: 8,
        wasm_memory_size: 0,
        stable_memory_size: 32_769,
        globals: vec![],
        certified_data: vec![],
        global_timer: Some(CanisterTimer::Inactive),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::ConditionNotSatisfied),
        wasm_chunk_store: [b"".as_slice(), b"stored chunk"]
            .into_iter()
            .map(|bytes| ChunkHash {
                hash: Sha256Digest::compute(bytes).as_bytes().to_vec(),
            })
            .collect(),
    };
    let transfer = plan.plan_transfer(&[1, 2], &metadata).unwrap();
    let mut total = 0;
    for request in transfer.requests() {
        total += source.read_data(&request).unwrap().len() as u64;
    }
    assert_eq!(total, source.summary().payload_bytes);
    assert_eq!(
        fs::read(fixture.folder.join("manifest.json")).unwrap(),
        before
    );
    assert_eq!(fixture.folder.read_dir().unwrap().count(), 2);
}

#[test]
fn arbitrary_bounded_ranges_cross_blocks_and_preserve_exact_bytes() {
    const BLOCK: usize = 1024 * 1024;
    let bytes: Vec<_> = (0..2 * BLOCK + 17).map(|n| (n % 251) as u8).collect();
    let fixture = Fixture::with_regions(
        "source-blocks",
        vec![vec![19; BLOCK]],
        bytes.clone(),
        bytes.clone(),
        bytes.clone(),
    );
    let mut source = fixture.source();
    for region in 0..3 {
        for (offset, size) in [
            (0, 1),
            (0, BLOCK),
            (BLOCK - 3, 9),
            (BLOCK + 1, BLOCK),
            (2 * BLOCK, 17),
            (2 * BLOCK + 16, 1),
        ] {
            let offset = offset as u64;
            let size = size as u64;
            let kind = match region {
                0 => SnapshotDataKind::WasmModule { offset, size },
                1 => SnapshotDataKind::WasmMemory { offset, size },
                _ => SnapshotDataKind::StableMemory { offset, size },
            };
            assert_eq!(
                source.read_data(&fixture.request(kind)).unwrap(),
                bytes[offset as usize..(offset + size) as usize]
            );
        }
    }
    let chunk = vec![19; BLOCK];
    assert_eq!(
        source
            .read_data(&fixture.request(SnapshotDataKind::WasmChunk {
                hash: Sha256Digest::compute(&chunk).as_bytes().to_vec()
            }))
            .unwrap(),
        chunk
    );
}

#[test]
fn invalid_identity_range_and_chunk_requests_leave_the_source_usable() {
    let fixture = Fixture::new("source-invalid-request");
    let mut source = fixture.source();
    let good = fixture.request(SnapshotDataKind::WasmModule { offset: 0, size: 8 });
    let mut wrong_target = fixture.request(SnapshotDataKind::WasmModule { offset: 0, size: 8 });
    wrong_target.canister_id = ic_platform::Principal::anonymous();
    assert!(matches!(
        source.read_data(&wrong_target),
        Err(PayloadReadError::Request)
    ));
    for id in [vec![], vec![1], vec![2, 1], vec![1; 257]] {
        let mut wrong = fixture.request(SnapshotDataKind::WasmModule { offset: 0, size: 8 });
        wrong.snapshot_id = id;
        assert!(matches!(
            source.read_data(&wrong),
            Err(PayloadReadError::Request)
        ));
    }
    for kind in [
        SnapshotDataKind::WasmModule { offset: 0, size: 0 },
        SnapshotDataKind::WasmModule { offset: 8, size: 1 },
        SnapshotDataKind::WasmMemory { offset: 0, size: 1 },
        SnapshotDataKind::StableMemory {
            offset: 0,
            size: 1024 * 1024 + 1,
        },
        SnapshotDataKind::StableMemory {
            offset: u64::MAX,
            size: 1,
        },
        SnapshotDataKind::WasmChunk { hash: vec![0; 31] },
        SnapshotDataKind::WasmChunk { hash: vec![0; 32] },
    ] {
        assert!(matches!(
            source.read_data(&fixture.request(kind)),
            Err(PayloadReadError::Request)
        ));
    }
    assert_eq!(source.read_data(&good).unwrap(), b"\0asm\x01\0\0\0");
    assert!(
        source
            .read_data(&fixture.request(SnapshotDataKind::WasmChunk {
                hash: Sha256Digest::compute(b"").as_bytes().to_vec()
            }))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn same_inode_same_length_corruption_is_caught_and_handle_stays_invalidated() {
    let fixture = Fixture::new("source-inplace-corruption");
    let mut source = fixture.source();
    let path = fixture.folder.join("payload/module.wasm");
    let original_identity =
        ic_host_fs::identity::FileIdentity::from_metadata(&fs::metadata(&path).unwrap());
    fs::write(&path, b"bad wasm").unwrap();
    assert_eq!(
        original_identity,
        ic_host_fs::identity::FileIdentity::from_metadata(&fs::metadata(&path).unwrap())
    );
    let request = fixture.request(SnapshotDataKind::WasmModule { offset: 0, size: 8 });
    assert!(matches!(
        source.read_data(&request),
        Err(PayloadReadError::Store(StoreError::Payload {
            source: ArtifactError::DigestMismatch { .. },
            ..
        }))
    ));
    fs::write(&path, b"\0asm\x01\0\0\0").unwrap();
    assert!(matches!(
        source.read_data(&request),
        Err(PayloadReadError::Inactive)
    ));
}

#[test]
fn replacement_even_with_identical_bytes_refuses_and_preserves_both_files() {
    let fixture = Fixture::new("source-replaced-file");
    let mut source = fixture.source();
    let path = fixture.folder.join("payload/module.wasm");
    let retained = fixture.folder.join("retained-module");
    fs::rename(&path, &retained).unwrap();
    write_file(&path, b"\0asm\x01\0\0\0");
    assert!(matches!(
        source.read_data(&fixture.request(SnapshotDataKind::WasmModule { offset: 0, size: 1 })),
        Err(PayloadReadError::Store(StoreError::CustodyChanged))
    ));
    assert_eq!(fs::read(path).unwrap(), fs::read(retained).unwrap());
}

#[test]
fn changed_member_length_manifest_or_tree_cannot_return_source_bytes() {
    for change in [
        "short",
        "long",
        "extra",
        "manifest",
        "directory",
        "permissions",
        "hardlink",
        "empty-chunk",
    ] {
        let fixture = Fixture::new(change);
        let mut source = fixture.source();
        let path = fixture.folder.join("payload/module.wasm");
        match change {
            "short" => fs::write(path, b"short").unwrap(),
            "long" => fs::write(path, b"longer wasm").unwrap(),
            "extra" => write_file(&fixture.folder.join("payload/extra.bin"), b"extra"),
            "manifest" => {
                let path = fixture.folder.join("manifest.json");
                let mut bytes = fs::read(&path).unwrap();
                bytes[1] ^= 1;
                fs::write(path, bytes).unwrap();
            }
            "directory" => {
                fs::rename(
                    fixture.folder.join("payload"),
                    fixture.folder.join("retained-payload"),
                )
                .unwrap();
                DirBuilder::new()
                    .mode(0o700)
                    .create(fixture.folder.join("payload"))
                    .unwrap();
            }
            "permissions" => fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap(),
            "hardlink" => fs::hard_link(path, fixture.folder.join("alias")).unwrap(),
            "empty-chunk" => fs::write(
                fixture
                    .folder
                    .join(format!("payload/chunk-{}.bin", Sha256Digest::compute(b""))),
                b"x",
            )
            .unwrap(),
            _ => unreachable!(),
        }
        let request = fixture.request(SnapshotDataKind::WasmModule { offset: 0, size: 1 });
        assert!(
            matches!(source.read_data(&request), Err(PayloadReadError::Store(_))),
            "{change}"
        );
        assert!(
            matches!(source.read_data(&request), Err(PayloadReadError::Inactive)),
            "{change}"
        );
    }
}

#[test]
fn held_source_follows_root_rename_and_never_reads_a_replacement_store() {
    let fixture = Fixture::new("source-root-rename");
    let mut source = fixture.source();
    let original_root = fixture.folder.parent().unwrap().parent().unwrap();
    let retained_root = original_root.with_extension("retained");
    fs::rename(original_root, &retained_root).unwrap();
    DirBuilder::new().mode(0o700).create(original_root).unwrap();
    let _replacement = BackupStore::initialize(original_root).unwrap();
    assert_eq!(
        source
            .read_data(&fixture.request(SnapshotDataKind::WasmModule { offset: 0, size: 8 }))
            .unwrap(),
        b"\0asm\x01\0\0\0"
    );
    assert!(
        original_root
            .join("bundles")
            .read_dir()
            .unwrap()
            .next()
            .is_none()
    );
}

#[test]
fn opening_checks_trusted_manifest_budget_and_complete_payload_before_returning_a_source() {
    let fixture = Fixture::new("source-opening");
    assert!(matches!(
        fixture.store.open_payload(
            &fixture.record.operation_id,
            Sha256Digest::compute(b"wrong"),
            NonZeroU64::new(1).unwrap()
        ),
        Err(StoreError::Read(ArtifactError::DigestMismatch { .. }))
    ));
    assert!(matches!(
        fixture.store.open_payload(
            &fixture.record.operation_id,
            fixture.commitment,
            NonZeroU64::new(1).unwrap()
        ),
        Err(StoreError::PayloadLimit { .. })
    ));
    fs::write(
        fixture.folder.join("payload/stable.bin"),
        vec![0xac; 32_769],
    )
    .unwrap();
    assert!(matches!(
        fixture.store.open_payload(
            &fixture.record.operation_id,
            fixture.commitment,
            NonZeroU64::new(fixture.record.payload_bytes).unwrap()
        ),
        Err(StoreError::Payload {
            source: ArtifactError::DigestMismatch { .. },
            ..
        })
    ));
}
