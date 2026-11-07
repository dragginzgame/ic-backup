//! Adapted Canic state/validation cases and exact local identity regressions.

use super::*;
use crate::tests::support::schema;

const HASH: &str = "abababababababababababababababababababababababababababababababab";
const CANISTER: &str = "aaaaa-aa";

fn request(canister: &str, snapshot: &str) -> DownloadArtifactRequest {
    DownloadArtifactRequest {
        canister_id: canister.to_owned(),
        snapshot_id: snapshot.to_owned(),
        snapshot_taken_at_timestamp: u64::MAX,
        snapshot_total_size_bytes: u64::MAX,
    }
}

fn journal() -> DownloadJournalRecord {
    DownloadJournalRecord::new(HASH, request(CANISTER, "snap-1")).expect("exact selection")
}

#[test]
fn canonical_identity_and_exact_metadata_survive_roundtrip() {
    let record =
        DownloadJournalRecord::new(&HASH.to_uppercase(), request("AAAAA-AA", "Snapshot-X"))
            .expect("normalize principal and digest");
    let entry = &record.artifacts()[0];
    assert_eq!(record.intent(), HASH);
    assert_eq!(entry.canister_id(), CANISTER);
    assert_eq!(entry.snapshot_id(), "Snapshot-X");
    assert_eq!(entry.snapshot_taken_at_timestamp(), u64::MAX);
    assert_eq!(entry.snapshot_total_size_bytes(), u64::MAX);
    assert_eq!(entry.staging_path(), "artifacts/aaaaa-aa.tmp");
    assert_eq!(entry.artifact_path(), "artifacts/aaaaa-aa");
    let decoded: DownloadJournalRecord =
        serde_json::from_slice(&serde_json::to_vec(&record).expect("serialize")).expect("decode");
    assert_eq!(decoded, record);
    assert!(matches!(
        record.artifact(CANISTER, "snapshot-x"),
        Err(DownloadJournalRecordError::SnapshotMismatch)
    ));
    assert!(matches!(
        record.artifact("2vxsx-fae", "Snapshot-X"),
        Err(DownloadJournalRecordError::UnknownArtifact)
    ));
}

#[test]
fn exact_transitions_preserve_identity_and_cannot_reset_spending_evidence() {
    let mut record = journal();
    let checksum = ArtifactChecksumRecord::from_bytes(b"actual staged bytes");
    let original = record.clone();
    assert!(
        record
            .advance(CANISTER, "snap-1", ArtifactTransition::Durable)
            .is_err()
    );
    assert_eq!(record, original);
    record
        .advance(
            CANISTER,
            "snap-1",
            ArtifactTransition::ChecksumVerified(checksum.clone()),
        )
        .unwrap();
    let verified = record.clone();
    assert!(
        record
            .advance(CANISTER, "wrong", ArtifactTransition::Durable)
            .is_err()
    );
    assert_eq!(record, verified);
    record
        .advance(CANISTER, "snap-1", ArtifactTransition::Durable)
        .unwrap();
    assert_eq!(record.artifacts()[0].checksum(), Some(&checksum));
    let durable = record.clone();
    for transition in [
        ArtifactTransition::Durable,
        ArtifactTransition::ChecksumVerified(checksum),
    ] {
        assert!(record.advance(CANISTER, "snap-1", transition).is_err());
        assert_eq!(record, durable);
    }
    let roundtrip: DownloadJournalRecord =
        serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
    assert_eq!(record, roundtrip);
}

#[test]
fn current_schema_requires_every_field_and_closed_state_evidence() {
    let value = serde_json::to_value(journal()).expect("serialize");
    for (state, checksum) in [
        (
            ArtifactStateRecord::Created,
            Some(ArtifactChecksumRecord::from_bytes(b"bytes")),
        ),
        (ArtifactStateRecord::ChecksumVerified, None),
        (ArtifactStateRecord::Durable, None),
    ] {
        let mut invalid = value.clone();
        invalid["artifacts"][0]["state"] = serde_json::to_value(state).unwrap();
        invalid["artifacts"][0]["checksum"] = serde_json::to_value(checksum).unwrap();
        assert!(
            serde_json::from_value::<DownloadJournalRecord>(invalid).is_err(),
            "invalid checksum state {state:?}"
        );
    }
    schema::required_fields::<DownloadJournalRecord>(
        &value,
        "",
        &["version", "intent", "artifacts"],
    );
    schema::required_fields::<DownloadJournalRecord>(
        &value,
        "/artifacts/0",
        &[
            "canister_id",
            "snapshot_id",
            "snapshot_taken_at_timestamp",
            "snapshot_total_size_bytes",
            "staging_path",
            "artifact_path",
            "state",
            "checksum",
        ],
    );
    for (field, invalid_value) in [
        ("state", serde_json::json!("ChecksumVerified")),
        ("state", serde_json::json!("Durable")),
        ("state", serde_json::json!("Unknown")),
        ("state", serde_json::json!("Downloaded")),
        (
            "checksum",
            serde_json::json!({"algorithm":"sha256","hash":HASH}),
        ),
        ("artifact_path", serde_json::json!("../outside")),
        ("artifact_path", serde_json::json!("artifacts/AAAAA-AA")),
        ("staging_path", serde_json::json!("/tmp/arbitrary")),
        ("unexpected", serde_json::json!(true)),
    ] {
        let mut invalid = value.clone();
        invalid["artifacts"][0][field] = invalid_value;
        assert!(
            serde_json::from_value::<DownloadJournalRecord>(invalid).is_err(),
            "rejected {field}"
        );
    }
    for field in ["version", "intent", "unexpected"] {
        let mut invalid = value.clone();
        invalid[field] = serde_json::json!(2);
        assert!(serde_json::from_value::<DownloadJournalRecord>(invalid).is_err());
    }
    schema::duplicate_versions::<DownloadJournalRecord>(&value);
}

#[test]
fn only_one_artifact_is_admitted_without_reinterpreting_old_sets() {
    let original = serde_json::to_value(journal()).unwrap();
    let entry = original["artifacts"][0].clone();
    for entries in [serde_json::json!([]), serde_json::json!([entry, entry])] {
        let mut invalid = original.clone();
        invalid["artifacts"] = entries;
        assert!(serde_json::from_value::<DownloadJournalRecord>(invalid).is_err());
    }
}

#[test]
fn snapshot_tokens_are_exact_bounded_ascii_and_principals_are_validated() {
    for value in [
        "",
        "white space",
        "line\nfeed",
        "🦀",
        "\0",
        &"x".repeat(MAX_SNAPSHOT_ID_BYTES + 1),
    ] {
        assert!(matches!(
            DownloadJournalRecord::new(HASH, request(CANISTER, value)),
            Err(DownloadJournalRecordError::InvalidSnapshotId)
        ));
    }
    assert!(
        DownloadJournalRecord::new(HASH, request(CANISTER, &"x".repeat(MAX_SNAPSHOT_ID_BYTES)))
            .is_ok()
    );
    for value in ["bad", "aaaaaaa", &"a".repeat(64)] {
        assert!(matches!(
            DownloadJournalRecord::new(HASH, request(value, "s")),
            Err(DownloadJournalRecordError::InvalidPrincipal)
        ));
    }
}

#[test]
fn download_manifest_independent_goldens_cover_states_metadata_and_exact_tokens() {
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/download-manifest.json"
    ))
    .unwrap();
    for golden in contract["goldens"].as_array().unwrap() {
        let record: DownloadJournalRecord =
            serde_json::from_value(golden["record"].clone()).unwrap();
        assert_eq!(record.digest().hash(), golden["sha256"].as_str().unwrap());
        let mut changed = golden["record"].clone();
        changed["intent"] = serde_json::json!(HASH.to_uppercase());
        changed["artifacts"][0]["canister_id"] = serde_json::json!("AAAAA-AA");
        let normalized: DownloadJournalRecord = serde_json::from_value(changed).unwrap();
        assert_eq!(record.digest(), normalized.digest());
        for (field, value) in [
            ("snapshot_id", serde_json::json!("s".repeat(256))),
            ("snapshot_taken_at_timestamp", serde_json::json!(0)),
            ("snapshot_total_size_bytes", serde_json::json!(0)),
        ] {
            let mut changed = golden["record"].clone();
            changed["artifacts"][0][field] = value;
            let changed: DownloadJournalRecord = serde_json::from_value(changed).unwrap();
            assert_ne!(record.digest(), changed.digest());
        }
    }
    for rejected in contract["rejected_records"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<DownloadJournalRecord>(rejected["record"].clone()).is_err()
        );
    }
}
