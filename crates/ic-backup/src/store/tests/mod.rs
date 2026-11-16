mod payload;

use super::manifest::{ManifestRecord, MemberRecord};
use super::*;
use std::{
    fs::{self, DirBuilder, OpenOptions},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt, symlink},
    path::PathBuf,
    sync::{
        Arc, Barrier,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

const GOLDEN: &[u8] = include_bytes!("fixtures/completion.json");
static NEXT: AtomicU64 = AtomicU64::new(0);

pub(super) fn root(label: &str) -> PathBuf {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/local-store-tests");
    fs::create_dir_all(&parent).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = parent.join(format!(
        "{label}-{}-{now}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    DirBuilder::new().mode(0o700).create(&path).unwrap();
    path
}

fn limits() -> ScanLimits {
    ScanLimits {
        entries: NonZeroUsize::new(100).unwrap(),
        bytes: NonZeroUsize::new(4 * MANIFEST_LIMIT).unwrap(),
        results: NonZeroUsize::new(100).unwrap(),
    }
}

fn record(id: u64, capture: u64, complete: u64) -> ManifestRecord {
    let mut record: ManifestRecord = serde_json::from_slice(GOLDEN).unwrap();
    record.operation_id = format!("{id:032x}");
    record.captured_at_ns = capture;
    record.completed_at_ns = complete;
    record
}

pub(super) fn write_file(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn bundle(root: &Path, record: &ManifestRecord) -> PathBuf {
    let path = root.join("bundles").join(&record.operation_id);
    DirBuilder::new().mode(0o700).create(&path).unwrap();
    write_file(
        &path.join("manifest.json"),
        &serde_json::to_vec(record).unwrap(),
    );
    path
}

fn target() -> BackupTarget {
    BackupTarget {
        network: "11".repeat(32).parse().unwrap(),
        canister: Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
    }
}

#[test]
fn initialization_is_create_once_and_open_never_repairs() {
    let path = root("initialize");
    assert!(matches!(
        BackupStore::open(&path),
        Err(StoreError::IncompleteStore)
    ));
    assert_eq!(fs::read_dir(&path).unwrap().count(), 0);
    let store = BackupStore::initialize(&path).unwrap();
    assert_eq!(fs::read(path.join("format")).unwrap(), FORMAT);
    assert_eq!(
        fs::metadata(path.join("format"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(store.list_backups(None, limits()).unwrap().is_empty());
    assert!(matches!(
        BackupStore::initialize(&path),
        Err(StoreError::Occupied)
    ));
    BackupStore::open(&path).unwrap();
    assert!(matches!(
        BackupStore::open(&path.join("absent")),
        Err(StoreError::MissingStore)
    ));
    assert!(!path.join("absent").exists());
}

#[test]
fn interrupted_initialization_remains_occupied() {
    let path = root("partial-initialize");
    write_file(&path.join("format.pending"), FORMAT);
    assert!(matches!(
        BackupStore::initialize(&path),
        Err(StoreError::Occupied)
    ));
    assert!(matches!(
        BackupStore::open(&path),
        Err(StoreError::IncompleteStore)
    ));
    assert_eq!(fs::read(path.join("format.pending")).unwrap(), FORMAT);
    assert!(!path.join("format").exists());
}

#[test]
fn competing_initializers_cannot_overwrite_each_other() {
    let path = root("contended-initialize");
    let barrier = Arc::new(Barrier::new(2));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let path = path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                BackupStore::initialize(&path).is_ok()
            })
        })
        .collect();
    assert_eq!(
        workers
            .into_iter()
            .map(|w| usize::from(w.join().unwrap()))
            .sum::<usize>(),
        1
    );
    BackupStore::open(&path).unwrap();
}

#[test]
fn list_and_latest_use_capture_time_with_deterministic_ties() {
    let path = root("recency");
    let store = BackupStore::initialize(&path).unwrap();
    for (id, capture, complete) in [(1, 10, 999), (2, 20, 30), (3, 20, 30)] {
        bundle(&path, &record(id, capture, complete));
    }
    let mut other = record(4, 100, 100);
    other.network = "22".repeat(32);
    bundle(&path, &other);
    let mut other_canister = record(5, 100, 100);
    other_canister.canister = "ryjl3-tyaaa-aaaaa-aaaba-cai".into();
    bundle(&path, &other_canister);
    let selected = store.list_backups(Some(&target()), limits()).unwrap();
    assert_eq!(
        selected
            .iter()
            .map(|r| r.bundle_id.as_str())
            .collect::<Vec<_>>(),
        [
            "00000000000000000000000000000003",
            "00000000000000000000000000000002",
            "00000000000000000000000000000001"
        ]
    );
    let mut one = limits();
    one.results = NonZeroUsize::new(1).unwrap();
    assert_eq!(
        store.latest_completed_backup(&target(), one).unwrap(),
        selected.first().cloned()
    );
    assert!(matches!(
        store.list_backups(None, one),
        Err(StoreError::ResultLimit)
    ));
    assert_eq!(store.list_backups(None, limits()).unwrap().len(), 5);
}

#[test]
fn unfinished_bundles_are_omitted_and_new_completions_appear_on_refresh() {
    let path = root("unfinished");
    let store = BackupStore::initialize(&path).unwrap();
    let original = record(1, 100, 200);
    let pending = path.join("bundles").join(&original.operation_id);
    DirBuilder::new().mode(0o700).create(&pending).unwrap();
    write_file(&pending.join("manifest.pending"), b"incomplete");
    assert!(
        store
            .latest_completed_backup(&target(), limits())
            .unwrap()
            .is_none()
    );
    write_file(&pending.join("manifest.json"), GOLDEN);
    assert_eq!(
        store
            .latest_completed_backup(&target(), limits())
            .unwrap()
            .unwrap()
            .captured_at_ns,
        100
    );
    assert_eq!(
        fs::read(pending.join("manifest.pending")).unwrap(),
        b"incomplete"
    );
}

#[test]
fn malformed_nonmatching_manifest_never_becomes_partial_success() {
    let path = root("malformed");
    let store = BackupStore::initialize(&path).unwrap();
    bundle(&path, &record(1, 100, 200));
    let mut other = record(2, 300, 400);
    other.network = "22".repeat(32);
    let bad = bundle(&path, &other).join("manifest.json");
    fs::write(&bad, b"{\"format\":").unwrap();
    assert!(matches!(
        store.latest_completed_backup(&target(), limits()),
        Err(StoreError::Manifest {
            source: ManifestError::Encoding(_),
            ..
        })
    ));
    assert_eq!(fs::read(&bad).unwrap(), b"{\"format\":");
}

#[test]
fn entry_and_byte_budgets_include_nonmatching_and_unfinished_entries() {
    let path = root("limits");
    let store = BackupStore::initialize(&path).unwrap();
    let mut other = record(1, 10, 20);
    other.network = "22".repeat(32);
    bundle(&path, &other);
    let unfinished = path.join("bundles/00000000000000000000000000000002");
    DirBuilder::new().mode(0o700).create(unfinished).unwrap();
    let mut tiny = limits();
    tiny.entries = NonZeroUsize::new(1).unwrap();
    assert!(matches!(
        store.latest_completed_backup(&target(), tiny),
        Err(StoreError::Filesystem(CapabilityError::EntryLimit { .. }))
    ));
    tiny = limits();
    tiny.bytes = NonZeroUsize::new(1).unwrap();
    assert!(matches!(
        store.latest_completed_backup(&target(), tiny),
        Err(StoreError::Read(ArtifactError::LimitExceeded { .. }))
    ));
    tiny.bytes = NonZeroUsize::new(serde_json::to_vec(&other).unwrap().len()).unwrap();
    assert!(
        store
            .latest_completed_backup(&target(), tiny)
            .unwrap()
            .is_none()
    );
}

#[test]
fn unsafe_root_bundle_and_manifest_links_are_refused() {
    let path = root("links");
    let store = BackupStore::initialize(&path).unwrap();
    let alias_parent = root("root-alias");
    let alias = alias_parent.join("alias");
    symlink(&path, &alias).unwrap();
    assert!(matches!(
        BackupStore::open(&alias),
        Err(StoreError::Filesystem(CapabilityError::Refused { .. }))
    ));
    let outside = root("outside");
    symlink(
        &outside,
        path.join("bundles/00000000000000000000000000000001"),
    )
    .unwrap();
    assert!(matches!(
        store.list_backups(None, limits()),
        Err(StoreError::InvalidLayout)
    ));

    let path = root("manifest-link");
    let store = BackupStore::initialize(&path).unwrap();
    let folder = path.join("bundles/00000000000000000000000000000001");
    DirBuilder::new().mode(0o700).create(&folder).unwrap();
    let source = outside.join("record");
    write_file(&source, GOLDEN);
    symlink(&source, folder.join("manifest.json")).unwrap();
    assert!(matches!(
        store.list_backups(None, limits()),
        Err(StoreError::Filesystem(CapabilityError::Refused { .. }))
    ));
    assert_eq!(fs::read(source).unwrap(), GOLDEN);
}

#[test]
fn nonprivate_and_unexpected_store_structure_are_refused() {
    let path = root("permissions");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(
        BackupStore::initialize(&path),
        Err(StoreError::Filesystem(CapabilityError::Permissions { .. }))
    ));
    let path = root("layout");
    BackupStore::initialize(&path).unwrap();
    write_file(&path.join("unexpected"), b"keep");
    assert!(BackupStore::open(&path).is_err());
    assert_eq!(fs::read(path.join("unexpected")).unwrap(), b"keep");
}

#[test]
fn held_store_does_not_follow_a_replacement_root() {
    let path = root("held-root");
    let store = BackupStore::initialize(&path).unwrap();
    bundle(&path, &record(1, 10, 20));
    let moved = path.with_extension("retained");
    fs::rename(&path, &moved).unwrap();
    DirBuilder::new().mode(0o700).create(&path).unwrap();
    BackupStore::initialize(&path).unwrap();
    assert_eq!(store.list_backups(None, limits()).unwrap().len(), 1);
    assert!(
        BackupStore::open(&path)
            .unwrap()
            .list_backups(None, limits())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn golden_manifest_is_passive_even_without_payload_or_operation_files() {
    let summary = manifest::decode("00000000000000000000000000000001", GOLDEN)
        .unwrap()
        .summary;
    assert_eq!(summary.target, target());
    assert_eq!(summary.payload_bytes, 8);
    assert_eq!(
        summary.manifest_sha256.to_string(),
        "d6ae354787c3955deffe807213f2ac6b953d96456abd8eb8413ba5944804d64e"
    );
    assert_eq!(summary.captured_at_ns, 100);
    assert_eq!(summary.completed_at_ns, 200);
    let path = root("passive");
    let store = BackupStore::initialize(&path).unwrap();
    let folder = path.join("bundles").join(&summary.bundle_id);
    DirBuilder::new().mode(0o700).create(&folder).unwrap();
    write_file(&folder.join("manifest.json"), GOLDEN);
    assert_eq!(store.list_backups(None, limits()).unwrap(), vec![summary]);
    assert!(!folder.join("payload").exists());
}

#[test]
fn manifest_rejects_ambiguous_unsupported_and_inconsistent_records() {
    let id = "00000000000000000000000000000001";
    let raw = std::str::from_utf8(GOLDEN).unwrap();
    for bad in [
        raw.replacen('{', "{\"unknown\":1,", 1),
        raw.replacen('{', "{\"captured_at_ns\":1,", 1),
        raw.replace("\"captured_at_ns\":100,", ""),
    ] {
        assert!(matches!(
            manifest::decode(id, bad.as_bytes()),
            Err(ManifestError::Encoding(_))
        ));
    }
    let mut r = record(1, 1, 2);
    r.format = "future".into();
    assert!(matches!(
        manifest::decode(id, &serde_json::to_vec(&r).unwrap()),
        Err(ManifestError::UnsupportedFormat)
    ));
    for mutate in [
        |r: &mut ManifestRecord| r.operation_id = "../escape".into(),
        |r: &mut ManifestRecord| r.network = "INVALID".into(),
        |r: &mut ManifestRecord| r.payload_bytes += 1,
        |r: &mut ManifestRecord| r.members[0].name = "../escape".into(),
        |r: &mut ManifestRecord| r.members[1].name = "module.wasm".into(),
        |r: &mut ManifestRecord| r.members[0].bytes = u64::MAX,
        |r: &mut ManifestRecord| r.metadata_reply.bytes = 0,
        |r: &mut ManifestRecord| r.snapshot_id_hex = "1".into(),
    ] {
        let mut r = record(1, 1, 2);
        mutate(&mut r);
        assert!(matches!(
            manifest::decode(id, &serde_json::to_vec(&r).unwrap()),
            Err(ManifestError::Invalid(_))
        ));
    }
}

#[test]
fn manifest_member_limit_is_enforced_during_decode() {
    let mut r = record(1, 1, 2);
    for _ in 0..1025 {
        r.members.push(MemberRecord {
            name: "stable.bin".into(),
            bytes: 0,
            sha256: "00".repeat(32),
        });
    }
    assert!(matches!(
        manifest::decode(&r.operation_id, &serde_json::to_vec(&r).unwrap()),
        Err(ManifestError::Encoding(_))
    ));
}

#[test]
fn cumulative_and_individual_manifest_byte_limits_refuse_without_partial_rows() {
    let path = root("cumulative-bytes");
    let store = BackupStore::initialize(&path).unwrap();
    let a = record(1, 10, 20);
    let b = record(2, 30, 40);
    bundle(&path, &a);
    bundle(&path, &b);
    let total = serde_json::to_vec(&a).unwrap().len() + serde_json::to_vec(&b).unwrap().len();
    let mut budget = limits();
    budget.bytes = NonZeroUsize::new(total).unwrap();
    assert_eq!(store.list_backups(None, budget).unwrap().len(), 2);
    budget.bytes = NonZeroUsize::new(total - 1).unwrap();
    assert!(matches!(
        store.list_backups(None, budget),
        Err(StoreError::Read(ArtifactError::LimitExceeded { .. }))
    ));

    let path = root("oversized-manifest");
    let store = BackupStore::initialize(&path).unwrap();
    let folder = path.join("bundles/00000000000000000000000000000001");
    DirBuilder::new().mode(0o700).create(&folder).unwrap();
    let mut bytes = GOLDEN.to_vec();
    bytes.resize(MANIFEST_LIMIT + 1, b' ');
    write_file(&folder.join("manifest.json"), &bytes);
    assert!(matches!(
        store.list_backups(None, limits()),
        Err(StoreError::Read(ArtifactError::LimitExceeded { .. }))
    ));
}

#[test]
fn aliased_manifest_is_refused_and_retained() {
    let path = root("hardlink");
    let store = BackupStore::initialize(&path).unwrap();
    let folder = bundle(&path, &record(1, 1, 2));
    let original = folder.join("manifest.json");
    fs::hard_link(&original, folder.join("alias")).unwrap();
    assert!(matches!(
        store.list_backups(None, limits()),
        Err(StoreError::InvalidLayout)
    ));
    assert_eq!(fs::metadata(original).unwrap().nlink(), 2);
}
