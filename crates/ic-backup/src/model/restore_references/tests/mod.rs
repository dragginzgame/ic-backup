use super::*;

fn reference(name: &str, hash: &str) -> RestoreReferenceRecord {
    RestoreReferenceRecord::new(PathBuf::from(format!("/operator/{name}.json")), hash)
        .expect("valid reference")
}

#[test]
fn canonical_hashes_and_exact_locations_own_equality() {
    let hash = "ab".repeat(32);
    assert_eq!(
        reference("journal", &hash),
        reference("journal", &hash.to_uppercase())
    );
    for path in [
        "relative.json",
        "/",
        "/operator/../journal.json",
        "/operator/./journal.json",
        "/operator//journal.json",
        "/operator/journal\0.json",
    ] {
        assert!(matches!(
            RestoreReferenceRecord::new(path.into(), &hash),
            Err(RestoreReferenceError::InvalidJournal { .. })
        ));
    }
    assert!(matches!(
        RestoreReferenceRecord::new("/operator/journal".into(), "not a hash"),
        Err(RestoreReferenceError::Checksum(ChecksumError::InvalidHash(
            _
        )))
    ));
    let oversized = format!("/{}", "x".repeat(MAX_JOURNAL_PATH_BYTES));
    assert!(matches!(
        RestoreReferenceRecord::new(oversized.into(), &hash),
        Err(RestoreReferenceError::InvalidJournal { .. })
    ));
}

#[test]
fn retention_is_ordered_idempotent_and_cannot_rebind_intent() {
    let mut record = RestoreReferencesRecord::empty();
    let first = reference("a", &"aa".repeat(32));
    let last = reference("z", &"bb".repeat(32));
    assert!(record.retain(last.clone()).expect("retain last"));
    assert!(record.retain(first.clone()).expect("retain first"));
    assert!(!record.retain(first.clone()).expect("idempotent retention"));
    assert_eq!(record.entries(), &[first, last]);
    let before = record.clone();
    assert!(
        matches!(record.retain(reference("a", &"cc".repeat(32))), Err(RestoreReferenceError::AuthorityConflict { journal }) if journal == Path::new("/operator/a.json"))
    );
    assert_eq!(record, before);
}

#[test]
fn persisted_generation_fields_and_duplicate_identities_reject() {
    let hash = "ab".repeat(32);
    let entry =
        serde_json::json!({"journal":"/operator/journal.json", "authority":hash.to_uppercase()});
    let valid = serde_json::json!({"version":1,"restores":[entry.clone()]});
    let record: RestoreReferencesRecord =
        serde_json::from_value(valid).expect("decode current record");
    assert_eq!(record.entries()[0].authority(), hash);
    let serialized = serde_json::to_value(record).expect("encode record");
    assert_eq!(
        serialized,
        serde_json::json!({"version":1,"restores":[{"journal":"/operator/journal.json","authority":hash}]})
    );
    for invalid in [
        serde_json::json!({"version":2,"restores":[]}),
        serde_json::json!({"version":1}),
        serde_json::json!({"restores":[]}),
        serde_json::json!({"version":1,"restores":[],"unknown":true}),
        serde_json::json!({"version":1,"restores":[entry.clone(),entry]}),
        serde_json::json!({"version":1,"restores":[{"journal":"relative","authority":hash}]}),
        serde_json::json!({"version":1,"restores":[{"journal":"/operator/journal.json","authority":"invalid"}]}),
        serde_json::json!({"version":1,"restores":[{"journal":"/operator/journal.json","authority":hash,"unknown":true}]}),
        serde_json::json!({"version":1,"restores":[{"journal":"/operator/journal.json"}]}),
    ] {
        assert!(serde_json::from_value::<RestoreReferencesRecord>(invalid).is_err());
    }
}

#[test]
fn count_bound_rejects_new_entries_and_excess_decoding() {
    let hash = "12".repeat(32);
    let mut record = RestoreReferencesRecord::empty();
    for index in 0..MAX_RESTORE_REFERENCES {
        record
            .retain(reference(&index.to_string(), &hash))
            .expect("within count bound");
    }
    assert!(
        !record
            .retain(reference("0", &hash))
            .expect("exact replay at full capacity")
    );
    let before = record.clone();
    assert!(matches!(
        record.retain(reference("extra", &hash)),
        Err(RestoreReferenceError::TooManyReferences {
            limit: MAX_RESTORE_REFERENCES
        })
    ));
    assert_eq!(record, before);
    let bytes = serde_json::to_vec(&record).expect("encode full admitted list");
    let decoded: RestoreReferencesRecord =
        serde_json::from_slice(&bytes).expect("exact bound accepts");
    assert_eq!(decoded, record);
    let mut value = serde_json::to_value(&record).expect("record as JSON");
    value["restores"]
        .as_array_mut()
        .expect("entries")
        .push(serde_json::to_value(reference("extra", &hash)).expect("new entry"));
    assert!(serde_json::from_value::<RestoreReferencesRecord>(value).is_err());
}

#[cfg(unix)]
#[test]
fn non_utf8_journal_identity_rejects() {
    use std::os::unix::ffi::OsStringExt;
    let path = PathBuf::from(std::ffi::OsString::from_vec(
        b"/operator/\xff.json".to_vec(),
    ));
    assert!(
        matches!(RestoreReferenceRecord::new(path.clone(), &"ab".repeat(32)), Err(RestoreReferenceError::InvalidJournal { journal }) if journal == path)
    );
}
