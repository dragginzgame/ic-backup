//! Native immutable inventory admission, retained evidence and byte/ownership bounds.

use super::*;
use crate::{
    model::inventory::{InventoryTargetRecord, InventoryTargetRequest, MAX_INVENTORY_TARGETS},
    test_support::temp_dir,
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
};

fn record() -> InventoryRecord {
    InventoryRecord::new(vec![
        InventoryTargetRecord::new(&InventoryTargetRequest {
            canister_id: "aaaaa-aa".into(),
            parent_canister_id: None,
            role: None,
            module_hash: None,
        })
        .expect("row"),
    ])
    .expect("inventory")
}
fn layout() -> (std::path::PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-inventory");
    fs::create_dir(&root).expect("layout");
    let guard = BackupLayoutGuard::acquire(&root).expect("exclude other owners");
    (root, guard)
}

#[test]
fn immutable_publication_reconciles_exact_digest_and_preserves_prior_evidence() {
    let (root, layout) = layout();
    let record = record();
    let path = root.join("inventory.json");
    create_inventory(&layout, &record).expect("durable declaration");
    let original = fs::read(&path).expect("bytes");
    assert_eq!(
        fs::metadata(&path).expect("metadata").permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        read_inventory(&layout, &record.digest()).expect("reconcile lost create response"),
        record
    );
    assert!(
        matches!(create_inventory(&layout,&record),Err(InventoryError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        read_inventory(
            &layout,
            &ArtifactChecksumRecord::from_bytes(b"different declaration")
        ),
        Err(InventoryError::DigestMismatch)
    ));
    assert_eq!(fs::read(&path).expect("unchanged"), original);
    let lock = JournalLock::acquire(&path).expect("hold cooperating journal owner");
    assert!(matches!(
        read_inventory(&layout, &record.digest()),
        Err(InventoryError::Lock(JournalLockError::Locked { .. }))
    ));
    drop(lock);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn canonical_output_bound_rejects_escaped_roles_before_creating_a_record() {
    let (root, layout) = layout();
    let rows = (0..MAX_INVENTORY_TARGETS)
        .map(|index| {
            InventoryTargetRecord::new(&InventoryTargetRequest {
                canister_id: ic_principal::Principal::from_slice(&index.to_be_bytes()).to_text(),
                parent_canister_id: None,
                role: Some("\0".repeat(256)),
                module_hash: None,
            })
            .expect("bounded role bytes")
        })
        .collect();
    let excessive = InventoryRecord::new(rows).expect("bounded physical set");
    assert!(matches!(
        create_inventory(&layout, &excessive),
        Err(InventoryError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_INVENTORY_BYTES
            }
        ))
    ));
    assert!(!root.join("inventory.json").exists());
    create_inventory(&layout, &record()).expect("original absence preserved");
    let path = root.join("inventory.json");
    fs::write(
        &path,
        vec![b' '; usize::try_from(MAX_INVENTORY_BYTES).expect("bound") + 1],
    )
    .expect("oversized input");
    assert!(matches!(
        read_inventory(&layout, &record().digest()),
        Err(InventoryError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_INVENTORY_BYTES
            }
        ))
    ));
    assert_eq!(
        fs::metadata(&path).expect("retained").len(),
        MAX_INVENTORY_BYTES + 1
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn malformed_unsafe_and_replaced_layout_records_remain_retained() {
    let (root, layout) = layout();
    let original = record();
    let path = root.join("inventory.json");
    create_inventory(&layout, &original).expect("record");
    fs::write(&path, b"{\"version\":1,\"targets\":[]}").expect("malformed exact set");
    let evidence = fs::read(&path).expect("retain bytes");
    assert!(
        matches!(read_inventory(&layout,&original.digest()),Err(InventoryError::Persistence(PersistenceError::Json(ref error))) if error.is_data())
    );
    assert_eq!(fs::read(&path).expect("unchanged"), evidence);
    let retained = root.join("retained.json");
    fs::rename(&path, &retained).expect("retain file");
    symlink(&retained, &path).expect("unsafe record");
    assert!(read_inventory(&layout, &original.digest()).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .expect("retained link")
            .file_type()
            .is_symlink()
    );
    let retained_root = root.with_extension("retained");
    fs::rename(&root, &retained_root).expect("retain root");
    fs::create_dir(&root).expect("replacement");
    assert!(matches!(
        read_inventory(&layout, &original.digest()),
        Err(InventoryError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert!(matches!(
        create_inventory(&layout, &original),
        Err(InventoryError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert!(!root.join("inventory.json").exists());
    drop(layout);
    fs::remove_dir_all(root).expect("clean replacement");
    fs::remove_dir_all(retained_root).expect("clean successful retained fixture");
}
