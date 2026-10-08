//! Native immutable graph publication, original digest admission and evidence retention.

use super::*;
use crate::{
    model::effect_graph::{EffectNodeRecord, EffectNodeRequest, MAX_EFFECT_DEPENDENCIES},
    test_support::temp_dir,
};
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
};
fn node(id: u64, deps: Vec<u64>) -> EffectNodeRecord {
    EffectNodeRecord::new(EffectNodeRequest {
        operation_sequence: id,
        depends_on: deps,
    })
    .expect("node")
}
fn graph() -> EffectGraphRecord {
    EffectGraphRecord::new(vec![node(10, vec![]), node(20, vec![10])]).expect("graph")
}
fn layout() -> (std::path::PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-effect-graph");
    fs::create_dir(&root).expect("layout");
    let guard = BackupLayoutGuard::acquire(&root).expect("layout exclusion");
    (root, guard)
}

#[test]
fn exact_immutable_graph_reconciles_lost_creation_and_rejects_replacement() {
    let (root, layout) = layout();
    let record = graph();
    let path = root.join("effect-graph.json");
    create_effect_graph(&layout, &record).expect("durable graph");
    let bytes = fs::read(&path).expect("evidence");
    assert_eq!(
        fs::metadata(&path).expect("metadata").permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        read_effect_graph(&layout, &record.digest()).expect("reconcile lost create response"),
        record
    );
    assert!(
        matches!(create_effect_graph(&layout,&record),Err(EffectGraphPersistenceError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        read_effect_graph(
            &layout,
            &ArtifactChecksumRecord::from_bytes(b"changed graph")
        ),
        Err(EffectGraphPersistenceError::DigestMismatch)
    ));
    assert_eq!(fs::read(&path).expect("unchanged"), bytes);
    let lock = JournalLock::acquire(&path).expect("hold other cooperating graph owner");
    assert!(matches!(
        read_effect_graph(&layout, &record.digest()),
        Err(EffectGraphPersistenceError::Lock(
            JournalLockError::Locked { .. }
        ))
    ));
    drop(lock);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn output_and_input_byte_bounds_reject_without_discarding_evidence() {
    let (root, layout) = layout();
    let dependencies: Vec<_> = (0..u64::try_from(MAX_EFFECT_DEPENDENCIES).expect("bound"))
        .map(|id| u64::MAX - id)
        .collect();
    let mut nodes: Vec<_> = dependencies.iter().map(|id| node(*id, vec![])).collect();
    for id in 0..64 {
        nodes.push(node(id, dependencies.clone()));
    }
    let excessive = EffectGraphRecord::new(nodes).expect("valid bounded dense graph");
    assert!(matches!(
        create_effect_graph(&layout, &excessive),
        Err(EffectGraphPersistenceError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_EFFECT_GRAPH_BYTES
            }
        ))
    ));
    let path = root.join("effect-graph.json");
    assert!(!path.exists());
    create_effect_graph(&layout, &graph()).expect("absence preserved");
    fs::write(
        &path,
        vec![b' '; usize::try_from(MAX_EFFECT_GRAPH_BYTES).expect("bound") + 1],
    )
    .expect("oversized fixture");
    assert!(matches!(
        read_effect_graph(&layout, &graph().digest()),
        Err(EffectGraphPersistenceError::Persistence(
            PersistenceError::RecordTooLarge {
                limit: MAX_EFFECT_GRAPH_BYTES
            }
        ))
    ));
    assert_eq!(
        fs::metadata(&path).expect("retained").len(),
        MAX_EFFECT_GRAPH_BYTES + 1
    );
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn invalid_graph_unsafe_entry_and_replaced_root_fail_without_repair() {
    let (root, layout) = layout();
    let record = graph();
    let path = root.join("effect-graph.json");
    create_effect_graph(&layout, &record).expect("graph");
    fs::write(
        &path,
        b"{\"version\":1,\"nodes\":[{\"operation_sequence\":1,\"depends_on\":[1]}]}",
    )
    .expect("cyclic evidence");
    let bytes = fs::read(&path).expect("evidence");
    assert!(
        matches!(read_effect_graph(&layout,&record.digest()),Err(EffectGraphPersistenceError::Persistence(PersistenceError::Json(ref error))) if error.is_data())
    );
    assert_eq!(fs::read(&path).expect("retained"), bytes);
    let retained = root.join("retained.json");
    fs::rename(&path, &retained).expect("retain fixture");
    symlink(&retained, &path).expect("unsafe");
    assert!(read_effect_graph(&layout, &record.digest()).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .expect("link retained")
            .file_type()
            .is_symlink()
    );
    let old = root.with_extension("retained");
    fs::rename(&root, &old).expect("retain root");
    fs::create_dir(&root).expect("replacement");
    assert!(matches!(
        read_effect_graph(&layout, &record.digest()),
        Err(EffectGraphPersistenceError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert!(matches!(
        create_effect_graph(&layout, &record),
        Err(EffectGraphPersistenceError::Persistence(
            PersistenceError::LayoutChanged { .. }
        ))
    ));
    assert!(!path.exists());
    drop(layout);
    fs::remove_dir_all(root).expect("clean replacement");
    fs::remove_dir_all(old).expect("clean successful retained fixture");
}
