//! Adapted Canic state/validation cases and exact local identity regressions.

use super::*;
use ic_principal::Principal;

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
    DownloadJournalRecord::new(HASH, vec![request(CANISTER, "snap-1")]).expect("exact selection")
}

#[test]
fn canonical_identity_and_exact_metadata_survive_roundtrip() {
    let record = DownloadJournalRecord::new(
        &HASH.to_uppercase(),
        vec![request("AAAAA-AA", "Snapshot-X")],
    )
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
fn canonical_transitions_update_evidence_together_and_project_resume_actions() {
    let mut record = journal();
    let checksum = ArtifactChecksumRecord::from_bytes(b"actual staged bytes");
    for (state, action, evidence) in [
        (ArtifactStateRecord::Created, ResumeAction::Download, None),
        (
            ArtifactStateRecord::Downloaded,
            ResumeAction::VerifyChecksum,
            None,
        ),
        (
            ArtifactStateRecord::ChecksumVerified,
            ResumeAction::Finalize,
            Some(checksum.clone()),
        ),
        (ArtifactStateRecord::Durable, ResumeAction::Skip, None),
    ] {
        if state != ArtifactStateRecord::Created {
            record
                .advance(CANISTER, "snap-1", state, evidence)
                .expect("canonical transition");
        }
        let view = record.resume_view();
        assert_eq!(view.artifacts[0].state, state);
        assert_eq!(view.artifacts[0].resume_action, action);
        assert_eq!(view.is_complete, state == ArtifactStateRecord::Durable);
        assert_eq!(
            view.pending_artifacts,
            usize::from(state != ArtifactStateRecord::Durable)
        );
        let bytes = serde_json::to_vec(&record).expect("serialize exact state");
        assert_eq!(
            serde_json::from_slice::<DownloadJournalRecord>(&bytes).expect("decode state"),
            record
        );
    }
    assert_eq!(record.artifacts()[0].checksum(), Some(&checksum));
}

#[test]
fn rejected_transitions_preserve_all_fields_and_never_rebind_identity() {
    let mut record = journal();
    let original = record.clone();
    for state in [
        ArtifactStateRecord::Created,
        ArtifactStateRecord::ChecksumVerified,
        ArtifactStateRecord::Durable,
    ] {
        assert!(matches!(
            record.advance(CANISTER, "snap-1", state, None),
            Err(DownloadJournalRecordError::InvalidStateTransition { .. })
        ));
        assert_eq!(record, original);
    }
    assert!(matches!(
        record.advance(
            CANISTER,
            "different-snapshot",
            ArtifactStateRecord::Downloaded,
            None
        ),
        Err(DownloadJournalRecordError::SnapshotMismatch)
    ));
    assert!(matches!(
        record.advance(
            CANISTER,
            "snap-1",
            ArtifactStateRecord::Downloaded,
            Some(ArtifactChecksumRecord::from_bytes(b"bytes"))
        ),
        Err(DownloadJournalRecordError::InvalidChecksumState(
            ArtifactStateRecord::Downloaded
        ))
    ));
    assert_eq!(record, original);
    record
        .advance(CANISTER, "snap-1", ArtifactStateRecord::Downloaded, None)
        .expect("download");
    let downloaded = record.clone();
    assert!(matches!(
        record.advance(
            CANISTER,
            "snap-1",
            ArtifactStateRecord::ChecksumVerified,
            None
        ),
        Err(DownloadJournalRecordError::InvalidChecksumState(
            ArtifactStateRecord::ChecksumVerified
        ))
    ));
    assert_eq!(record, downloaded);
    record
        .advance(
            CANISTER,
            "snap-1",
            ArtifactStateRecord::ChecksumVerified,
            Some(ArtifactChecksumRecord::from_bytes(b"bytes")),
        )
        .expect("verify");
    record
        .advance(CANISTER, "snap-1", ArtifactStateRecord::Durable, None)
        .expect("publish");
    let durable = record.clone();
    assert!(matches!(
        record.advance(CANISTER, "snap-1", ArtifactStateRecord::Downloaded, None),
        Err(DownloadJournalRecordError::InvalidStateTransition { .. })
    ));
    assert_eq!(record, durable);
}

#[test]
fn current_schema_requires_every_field_and_closed_state_evidence() {
    let value = serde_json::to_value(journal()).expect("serialize");
    for field in ["version", "intent", "artifacts"] {
        let mut invalid = value.clone();
        invalid.as_object_mut().expect("object").remove(field);
        assert!(
            serde_json::from_value::<DownloadJournalRecord>(invalid).is_err(),
            "missing {field}"
        );
    }
    for field in [
        "canister_id",
        "snapshot_id",
        "snapshot_taken_at_timestamp",
        "snapshot_total_size_bytes",
        "staging_path",
        "artifact_path",
        "state",
        "checksum",
    ] {
        let mut invalid = value.clone();
        invalid["artifacts"][0]
            .as_object_mut()
            .expect("entry")
            .remove(field);
        assert!(
            serde_json::from_value::<DownloadJournalRecord>(invalid).is_err(),
            "missing {field}"
        );
    }
    for (field, invalid_value) in [
        ("state", serde_json::json!("ChecksumVerified")),
        ("state", serde_json::json!("Durable")),
        ("state", serde_json::json!("Unknown")),
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
    let duplicated = serde_json::to_string(&value).expect("json").replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(serde_json::from_str::<DownloadJournalRecord>(&duplicated).is_err());
}

#[test]
fn empty_duplicate_and_excessive_selections_reject_before_resume() {
    assert!(matches!(
        DownloadJournalRecord::new(HASH, vec![]),
        Err(DownloadJournalRecordError::EmptyArtifacts)
    ));
    assert!(matches!(
        DownloadJournalRecord::new(
            HASH,
            vec![request(CANISTER, "snap-1"), request("AAAAA-AA", "snap-2")]
        ),
        Err(DownloadJournalRecordError::DuplicateCanister)
    ));
    assert!(matches!(
        DownloadJournalRecord::new(
            HASH,
            vec![request(CANISTER, "s"); MAX_DOWNLOAD_ARTIFACTS + 1]
        ),
        Err(DownloadJournalRecordError::TooManyArtifacts)
    ));
    let mut value = serde_json::to_value(journal()).expect("json");
    let entry = value["artifacts"][0].clone();
    value["artifacts"] = serde_json::json!(vec![entry; MAX_DOWNLOAD_ARTIFACTS + 1]);
    assert!(
        serde_json::from_value::<DownloadJournalRecord>(value)
            .expect_err("bounded collection")
            .to_string()
            .contains("artifact count exceeds")
    );
    let requests = (0..MAX_DOWNLOAD_ARTIFACTS)
        .map(|index| request(&Principal::from_slice(&index.to_be_bytes()).to_text(), "s"))
        .collect();
    let record = DownloadJournalRecord::new(HASH, requests).expect("exact count limit");
    assert_eq!(record.artifacts().len(), MAX_DOWNLOAD_ARTIFACTS);
    assert_eq!(
        record.resume_view().pending_artifacts,
        MAX_DOWNLOAD_ARTIFACTS
    );
    assert_eq!(
        serde_json::from_slice::<DownloadJournalRecord>(
            &serde_json::to_vec(&record).expect("json")
        )
        .expect("decode limit"),
        record
    );
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
            DownloadJournalRecord::new(HASH, vec![request(CANISTER, value)]),
            Err(DownloadJournalRecordError::InvalidSnapshotId)
        ));
    }
    assert!(
        DownloadJournalRecord::new(
            HASH,
            vec![request(CANISTER, &"x".repeat(MAX_SNAPSHOT_ID_BYTES))]
        )
        .is_ok()
    );
    for value in ["bad", "aaaaaaa", &"a".repeat(64)] {
        assert!(matches!(
            DownloadJournalRecord::new(HASH, vec![request(value, "s")]),
            Err(DownloadJournalRecordError::InvalidPrincipal)
        ));
    }
}
