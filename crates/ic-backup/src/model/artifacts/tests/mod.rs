use super::*;

#[test]
fn checksum_record_normalizes_equivalent_hashes_at_decode() {
    let record = ArtifactChecksumRecord::from_bytes(b"snapshot");
    let encoded =
        serde_json::json!({"algorithm": "sha256", "hash": record.hash().to_ascii_uppercase()});
    let decoded: ArtifactChecksumRecord =
        serde_json::from_value(encoded).expect("decode uppercase digest");
    assert_eq!(decoded, record);
    assert_eq!(
        serde_json::to_value(decoded).expect("encode record"),
        serde_json::json!({"algorithm": "sha256", "hash": record.hash()})
    );
}

#[test]
fn checksum_record_requires_exact_valid_fields() {
    let digest = ArtifactChecksumRecord::from_bytes(b"snapshot");
    for value in [
        serde_json::json!({"hash": digest.hash()}),
        serde_json::json!({"algorithm": "sha256"}),
        serde_json::json!({"algorithm": "sha512", "hash": digest.hash()}),
        serde_json::json!({"algorithm": "sha256", "hash": "invalid"}),
        serde_json::json!({"algorithm": "sha256", "hash": digest.hash(), "extra": true}),
    ] {
        assert!(serde_json::from_value::<ArtifactChecksumRecord>(value).is_err());
    }
    assert!(matches!(
        ArtifactChecksumRecord::from_hash("invalid"),
        Err(ChecksumError::InvalidHash(_))
    ));
}
