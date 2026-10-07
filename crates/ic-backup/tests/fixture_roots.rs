//! Real filesystem allocation regressions, independent of IC behavior.

mod support;

use std::{collections::BTreeSet, fs, io, sync::atomic::AtomicU64, thread};

#[test]
fn occupied_names_preserve_original_files_directories_and_links() {
    let parent = support::temp_root("ic-backup-fixture-occupied");
    fs::create_dir(parent.join("case-0")).unwrap();
    fs::write(
        parent.join("case-0/evidence"),
        b"original directory evidence",
    )
    .unwrap();
    fs::write(parent.join("case-1"), b"original file evidence").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("missing-original", parent.join("case-2")).unwrap();
    #[cfg(not(unix))]
    fs::create_dir(parent.join("case-2")).unwrap();

    let root = support::reserve_root(&parent, "case", &AtomicU64::new(0)).unwrap();
    assert_eq!(root, parent.join("case-3"));
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    assert_eq!(
        fs::read(parent.join("case-0/evidence")).unwrap(),
        b"original directory evidence"
    );
    assert_eq!(
        fs::read(parent.join("case-1")).unwrap(),
        b"original file evidence"
    );
    #[cfg(unix)]
    assert_eq!(
        fs::read_link(parent.join("case-2")).unwrap(),
        std::path::Path::new("missing-original")
    );
    fs::remove_dir_all(parent).unwrap(); // Only this successful, owned fixture.
}

#[test]
fn bounded_exhaustion_and_parent_errors_preserve_existing_evidence() {
    let parent = support::temp_root("ic-backup-fixture-exhaustion");
    for index in 0..128 {
        fs::write(parent.join(format!("case-{index}")), b"retained").unwrap();
    }
    let error = support::reserve_root(&parent, "case", &AtomicU64::new(0)).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    for index in 0..128 {
        assert_eq!(
            fs::read(parent.join(format!("case-{index}"))).unwrap(),
            b"retained"
        );
    }
    assert!(!parent.join("case-128").exists());
    let error =
        support::reserve_root(&parent.join("missing"), "case", &AtomicU64::new(0)).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(!parent.join("missing").exists());
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn independent_concurrent_allocators_reserve_distinct_roots_without_a_clock() {
    let parent = support::temp_root("ic-backup-fixture-concurrent");
    let roots = thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    support::reserve_root(&parent, "same-prefix", &AtomicU64::new(0)).unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<BTreeSet<_>>()
    });
    assert_eq!(roots.len(), 16);
    for root in &roots {
        assert!(root.is_dir());
        assert!(fs::read_dir(root).unwrap().next().is_none());
    }
    fs::remove_dir_all(parent).unwrap();
}
