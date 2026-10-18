use super::*;
use crate::test_support::temp_dir;
use crate::test_support::{create_json_durable, write_json_durable};
use ic_host_fs::durable::NamedWriteError;
use serde::Serializer;
use std::fs;

struct FailingSerialize;

impl Serialize for FailingSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom(
            "intentional serialization failure",
        ))
    }
}

#[test]
fn bounded_json_size_uses_exact_pretty_encoding_and_original_limits() {
    for value in [
        serde_json::Value::Null,
        serde_json::json!({"escaped": "\n\t\"\\", "utf8": "é雪", "nested": [1, 2]}),
        serde_json::json!({"state": "a".repeat(100_001)}),
    ] {
        let size = serde_json::to_vec_pretty(&value)
            .expect("original encoding")
            .len() as u64;
        for limit in [size, size + 1, u64::MAX] {
            check_json_size(&value, limit).expect("inclusive exact encoding budget");
        }
        for limit in [0, size - 1] {
            assert!(matches!(
                check_json_size(&value, limit),
                Err(PersistenceError::RecordTooLarge { limit: actual }) if actual == limit
            ));
        }
    }
}

#[test]
fn bounded_json_size_preserves_serializer_errors() {
    assert!(matches!(
        check_json_size(&FailingSerialize, u64::MAX),
        Err(PersistenceError::Json(error)) if !error.is_io()
    ));
}

#[test]
fn durable_json_replaces_the_complete_document() {
    let root = temp_dir("ic-backup-durable-json-replace");
    let path = root.join("journal.json");
    fs::create_dir_all(&root).expect("create temp root");
    fs::write(&path, b"previous-document-with-more-bytes").expect("write previous document");

    write_json_durable(&path, &serde_json::json!({"state": "ready"})).expect("replace document");

    let written = fs::read_to_string(&path).expect("read replaced document");
    let decoded: serde_json::Value = serde_json::from_str(&written).expect("decode document");
    assert_eq!(decoded, serde_json::json!({"state": "ready"}));
    assert_no_staging_file(&root, "journal.json");
    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn serialization_failure_preserves_the_previous_document() {
    let root = temp_dir("ic-backup-durable-json-serialize");
    let path = root.join("journal.json");
    fs::create_dir_all(&root).expect("create temp root");
    fs::write(&path, b"previous-document").expect("write previous document");

    let error = write_json_durable(&path, &FailingSerialize)
        .expect_err("serialization failure should reject");

    assert!(matches!(error, PersistenceError::Json(_)));
    assert_eq!(
        fs::read(&path).expect("read previous document"),
        b"previous-document"
    );
    assert_no_staging_file(&root, "journal.json");
    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn bounded_encoding_publishes_its_exact_single_pass_and_refuses_before_effects() {
    use std::cell::Cell;

    struct Probe<'a>(&'a Cell<u8>);
    impl Serialize for Probe<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.0.set(self.0.get() + 1);
            serializer.serialize_str(if self.0.get() == 1 {
                "first"
            } else {
                "changed"
            })
        }
    }

    for create in [false, true] {
        for limit in [6, 7] {
            let root = temp_dir("ic-backup-bounded-encoding");
            let parent = root.join("private/nested");
            let path = parent.join("record.json");
            let calls = Cell::new(0);
            let encoded = encode_json(&Probe(&calls), limit);
            assert_eq!(calls.get(), 1);
            assert!(!parent.exists());
            if limit == 6 {
                assert!(matches!(
                    encoded,
                    Err(PersistenceError::RecordTooLarge { limit: 6 })
                ));
            } else {
                let bytes = encoded.unwrap();
                assert_eq!(bytes, b"\"first\"");
                if create {
                    create_encoded(&path, &bytes)
                } else {
                    write_encoded(&path, &bytes)
                }
                .unwrap();
                assert_eq!(fs::read(&path).unwrap(), bytes);
                assert_eq!(calls.get(), 1);
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[test]
fn rename_failure_removes_the_staging_file() {
    let root = temp_dir("ic-backup-durable-json-rename");
    let path = root.join("journal.json");
    fs::create_dir_all(&path).expect("create conflicting target directory");

    let error = write_json_durable(&path, &serde_json::json!({"state": "ready"}))
        .expect_err("rename over directory should reject");

    assert!(matches!(
        error,
        PersistenceError::Publication(
            ic_host_fs::durable::NamedWriteError::BeforePublication { .. }
        )
    ));
    assert!(path.is_dir());
    assert_no_staging_file(&root, "journal.json");
    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn serialization_runs_once_before_parent_creation_even_when_it_fails() {
    use std::cell::Cell;

    struct Probe<'a> {
        parent: &'a Path,
        calls: &'a Cell<u8>,
        reject: bool,
    }
    impl Serialize for Probe<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            assert!(!self.parent.exists());
            self.calls.set(self.calls.get() + 1);
            if self.reject {
                Err(serde::ser::Error::custom("failed preflight"))
            } else {
                serializer.serialize_u8(7)
            }
        }
    }

    for create in [false, true] {
        for reject in [false, true] {
            let root = temp_dir("ic-backup-json-preflight");
            let parent = root.join("private/nested");
            let path = parent.join("record.json");
            let calls = Cell::new(0);
            let value = Probe {
                parent: &parent,
                calls: &calls,
                reject,
            };
            let result = if create {
                create_json_durable(&path, &value)
            } else {
                write_json_durable(&path, &value)
            };
            assert_eq!(calls.get(), 1);
            if reject {
                assert!(matches!(result, Err(PersistenceError::Json(_))));
                assert!(!root.join("private").exists());
            } else {
                result.unwrap();
                assert_eq!(fs::read(&path).unwrap(), b"7");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    for directory in [root.join("private"), parent] {
                        assert_eq!(
                            fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                            0o700
                        );
                    }
                }
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn private_json_parents_refuse_aliases_and_traversal_before_effects() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let root = temp_dir("ic-backup-json-private-parents");
    let physical = root.join("physical");
    fs::create_dir(&physical).unwrap();
    fs::create_dir(physical.join("existing")).unwrap();
    let original = physical.join("record.json");
    fs::write(&original, b"original bytes").unwrap();
    fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();

    let alias = root.join("alias");
    symlink(&physical, &alias).unwrap();
    let value = serde_json::json!({"state": "new"});
    for path in [
        alias.join("record.json"),
        alias.join("existing/record.json"),
        alias.join("missing/record.json"),
    ] {
        for create in [false, true] {
            let result = if create {
                create_json_durable(&path, &value)
            } else {
                write_json_durable(&path, &value)
            };
            assert!(matches!(
                result,
                Err(PersistenceError::Io(error)) if error.kind() == io::ErrorKind::NotADirectory
            ));
        }
    }
    assert!(!physical.join("missing").exists());
    assert!(!physical.join("existing/record.json").exists());
    assert_eq!(fs::read(&original).unwrap(), b"original bytes");
    assert_eq!(
        fs::metadata(&original).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let missing = root.join("missing");
    assert!(matches!(
        create_json_durable(&missing.join("../physical/new.json"), &value),
        Err(PersistenceError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput
    ));
    assert!(!missing.exists());
    assert!(!physical.join("new.json").exists());

    // An explicitly selected root is resolved by its owning boundary once.
    let selected = alias.canonicalize().unwrap();
    create_json_durable(&selected.join("existing/new.json"), &value).unwrap();
    assert_eq!(
        fs::read(selected.join("existing/new.json")).unwrap(),
        serde_json::to_vec_pretty(&value).unwrap()
    );
    assert_eq!(
        fs::metadata(selected.join("existing/new.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn foreign_staging_replacement_retains_typed_cleanup_evidence() {
    for mode in [PublicationMode::Replace, PublicationMode::CreateNew] {
        let root = temp_dir("ic-backup-json-stage-custody");
        let path = root.join("record.json");
        let displaced = root.join("displaced");
        if mode == PublicationMode::Replace {
            fs::write(&path, b"previous").unwrap();
        }
        let mut foreign = None;
        let error = publish_bytes_at_barriers(&path, b"new", mode, |barrier| {
            if barrier == DurableWriteBarrier::BeforeRename {
                let stage = fs::read_dir(&root)
                    .unwrap()
                    .map(Result::unwrap)
                    .find(|entry| entry.path() != path)
                    .unwrap()
                    .path();
                fs::rename(&stage, &displaced).unwrap();
                fs::write(&stage, b"foreign").unwrap();
                foreign = Some(stage);
            }
        })
        .unwrap_err();
        assert!(matches!(error, PersistenceError::Publication(
            NamedWriteError::BeforePublication { source, cleanup_error: Some(cleanup) }
        ) if source.kind() == io::ErrorKind::InvalidData && cleanup.kind() == io::ErrorKind::InvalidData));
        assert_eq!(fs::read(displaced).unwrap(), b"new");
        assert_eq!(fs::read(foreign.unwrap()).unwrap(), b"foreign");
        if mode == PublicationMode::Replace {
            assert_eq!(fs::read(path).unwrap(), b"previous");
        } else {
            assert!(!path.exists());
        }
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn held_parent_publication_does_not_switch_to_a_replaced_path() {
    let root = temp_dir("ic-backup-json-parent-custody");
    let parent = root.join("parent");
    let moved = root.join("moved");
    let path = parent.join("record.json");
    let mut durable = false;
    let bytes = encode_json(&serde_json::json!({"new": true}), 16 * 1024 * 1024).unwrap();
    publish_bytes_at_barriers(&path, &bytes, PublicationMode::Replace, |barrier| {
        if barrier == DurableWriteBarrier::BeforeRename {
            fs::rename(&parent, &moved).unwrap();
            fs::create_dir(&parent).unwrap();
            fs::write(&path, b"foreign").unwrap();
        } else {
            durable = true;
        }
    })
    .unwrap();
    assert!(durable);
    assert_eq!(fs::read(path).unwrap(), b"foreign");
    assert_eq!(
        read_json::<serde_json::Value>(&moved.join("record.json"), 1024).unwrap(),
        serde_json::json!({"new": true})
    );
    fs::remove_dir_all(root).unwrap();
}

fn assert_no_staging_file(root: &Path, target_name: &str) {
    let staging_files = fs::read_dir(root)
        .expect("read temp root")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() != target_name)
        .collect::<Vec<_>>();
    assert!(staging_files.is_empty(), "staging files remain");
}

#[test]
fn host_publication_failures_keep_exact_phase_source_and_cleanup_evidence() {
    use std::error::Error;
    let producer = PersistenceError::from(NamedWriteError::Producer {
        source: io::Error::new(io::ErrorKind::BrokenPipe, "producer failed"),
        cleanup_error: Some(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "stage held",
        )),
    });
    assert!(matches!(
        &producer,
        PersistenceError::Publication(NamedWriteError::Producer { source, cleanup_error: Some(cleanup) })
            if source.kind() == io::ErrorKind::BrokenPipe
                && cleanup.kind() == io::ErrorKind::PermissionDenied
    ));
    assert!(
        producer
            .to_string()
            .contains("staging cleanup failed: stage held")
    );
    assert_eq!(producer.source().unwrap().to_string(), "producer failed");

    let before = PersistenceError::from(NamedWriteError::<io::Error>::BeforePublication {
        source: io::Error::new(io::ErrorKind::AlreadyExists, "original remains"),
        cleanup_error: Some(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "stage held",
        )),
    });
    assert!(matches!(
        &before,
        PersistenceError::Publication(NamedWriteError::BeforePublication { source, cleanup_error: Some(cleanup) })
            if source.kind() == io::ErrorKind::AlreadyExists
                && cleanup.kind() == io::ErrorKind::PermissionDenied
    ));
    assert!(
        before
            .to_string()
            .contains("staging cleanup failed: stage held")
    );
    assert_eq!(before.source().unwrap().to_string(), "original remains");

    let after = PersistenceError::from(NamedWriteError::<io::Error>::AfterPublication {
        source: io::Error::other("sync uncertain"),
    });
    assert!(matches!(
        &after,
        PersistenceError::Publication(NamedWriteError::AfterPublication { source }) if source.to_string() == "sync uncertain"
    ));
    assert!(
        after
            .to_string()
            .contains("output published but completion failed")
    );
    assert_eq!(after.source().unwrap().to_string(), "sync uncertain");
}
