use super::*;

#[test]
fn custody_evidence_has_one_exact_generation_and_identity() {
    let record = CommandCustodyRecord::new("/operator/journal.json".into(), u64::MAX, 42, 7)
        .expect("observed identity");
    let value = serde_json::to_value(&record).expect("encode record");
    assert_eq!(
        value,
        serde_json::json!({"version":1,"journal":"/operator/journal.json","operation_sequence":u64::MAX,"device":42,"inode":7})
    );
    let decoded: CommandCustodyRecord =
        serde_json::from_value(value.clone()).expect("decode exact evidence");
    assert_eq!(decoded, record);
    assert!(decoded.matches_file(42, 7));
    assert!(!decoded.matches_file(42, 8));
    assert!(!decoded.matches_file(43, 7));
    for key in [
        "version",
        "journal",
        "operation_sequence",
        "device",
        "inode",
    ] {
        let mut invalid = value.clone();
        invalid.as_object_mut().expect("object").remove(key);
        assert!(serde_json::from_value::<CommandCustodyRecord>(invalid).is_err());
        let mut invalid = value.clone();
        invalid[key] = serde_json::Value::Null;
        assert!(serde_json::from_value::<CommandCustodyRecord>(invalid).is_err());
    }
    for (key, invalid_value) in [
        ("version", serde_json::json!(2)),
        ("inode", serde_json::json!(0)),
        ("journal", serde_json::json!("relative.json")),
        ("journal", serde_json::json!("/operator/../journal.json")),
        ("journal", serde_json::json!("/operator/./journal.json")),
        ("journal", serde_json::json!("/operator//journal.json")),
        (
            "journal",
            serde_json::json!(format!("/{}", "x".repeat(4096))),
        ),
        ("unknown", serde_json::json!(true)),
    ] {
        let mut invalid = value.clone();
        invalid[key] = invalid_value;
        assert!(serde_json::from_value::<CommandCustodyRecord>(invalid).is_err());
    }
    assert!(serde_json::from_str::<CommandCustodyRecord>(r#"{"version":1,"version":1,"journal":"/operator/journal.json","operation_sequence":0,"device":0,"inode":1}"#).is_err());
}

#[test]
fn worst_case_path_escaping_remains_inside_the_record_byte_bound() {
    let journal = PathBuf::from(format!("/{}", "\u{1}".repeat(4095)));
    let record = CommandCustodyRecord::new(journal, u64::MAX, u64::MAX, u64::MAX)
        .expect("bounded exact path");
    let bytes = serde_json::to_vec_pretty(&record).expect("encode fully escaped path");
    assert!(bytes.len() as u64 <= MAX_COMMAND_CUSTODY_RECORD_BYTES);
    assert_eq!(
        serde_json::from_slice::<CommandCustodyRecord>(&bytes).expect("roundtrip"),
        record
    );
    assert!(matches!(
        CommandCustodyRecord::new("relative".into(), 0, 0, 1),
        Err(CommandCustodyRecordError::InvalidJournal { .. })
    ));
    assert!(matches!(
        CommandCustodyRecord::new("/journal".into(), 0, 0, 0),
        Err(CommandCustodyRecordError::UnknownFileIdentity)
    ));
}
