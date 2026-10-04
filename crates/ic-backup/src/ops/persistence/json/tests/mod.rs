use super::*;
use crate::test_support::temp_dir;
use serde::Serializer;

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
fn durable_json_replaces_the_complete_document() {
    let root = temp_dir("canic-backup-durable-json-replace");
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
    let root = temp_dir("canic-backup-durable-json-serialize");
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
fn rename_failure_removes_the_staging_file() {
    let root = temp_dir("canic-backup-durable-json-rename");
    let path = root.join("journal.json");
    fs::create_dir_all(&path).expect("create conflicting target directory");

    let error = write_json_durable(&path, &serde_json::json!({"state": "ready"}))
        .expect_err("rename over directory should reject");

    assert!(matches!(error, PersistenceError::Io(_)));
    assert!(path.is_dir());
    assert_no_staging_file(&root, "journal.json");
    fs::remove_dir_all(root).expect("remove temp root");
}

fn assert_no_staging_file(root: &Path, target_name: &str) {
    let prefix = format!(".{target_name}.ic-backup-tmp-");
    let staging_files = fs::read_dir(root)
        .expect("read temp root")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
        .collect::<Vec<_>>();
    assert!(staging_files.is_empty(), "staging files remain");
}
