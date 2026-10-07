use super::*;

#[test]
fn digest_ingress_preserves_equivalent_case_exact_records_and_original_errors() {
    let hash = "0123456789abcdef".repeat(4);
    let record = ArtifactChecksumRecord::from_hash(&hash).unwrap();
    for input in [hash.clone(), hash.to_ascii_uppercase(), "aA0F".repeat(16)] {
        let admitted = ArtifactChecksumRecord::from_hash(&input).unwrap();
        assert_eq!(admitted.hash(), input.to_ascii_lowercase());
        admitted.verify(&input).unwrap();
        let decoded: ArtifactChecksumRecord =
            serde_json::from_value(serde_json::json!({"algorithm":"sha256", "hash":input}))
                .unwrap();
        assert_eq!(decoded, admitted);
        assert_eq!(
            serde_json::to_value(admitted).unwrap(),
            serde_json::json!({"algorithm":"sha256", "hash":input.to_ascii_lowercase()})
        );
    }
    for invalid in [
        String::new(),
        "A".repeat(63),
        "A".repeat(65),
        "G".repeat(64),
        "\0".repeat(64),
        "é".repeat(32),
        format!("{hash} "),
    ] {
        assert!(
            matches!(ArtifactChecksumRecord::from_hash(&invalid), Err(ChecksumError::InvalidHash(original)) if original == invalid)
        );
        assert!(
            matches!(record.verify(&invalid), Err(ChecksumError::InvalidHash(original)) if original == invalid)
        );
    }
    assert!(
        matches!(record.verify(&"AB".repeat(32)), Err(ChecksumError::ChecksumMismatch { expected, actual }) if expected == "ab".repeat(32) && actual == hash)
    );
    assert_eq!(record.hash(), hash);
}

#[test]
fn raw_digest_projection_preserves_leading_zero_bytes_and_exact_record_fields() {
    let bytes: [u8; 32] = std::array::from_fn(|index| u8::try_from(index).unwrap());
    let record = ArtifactChecksumRecord::from_digest(bytes);
    let hash = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    assert_eq!(record.hash(), hash);
    assert_eq!(
        serde_json::to_value(&record).unwrap(),
        serde_json::json!({"algorithm":"sha256", "hash":hash})
    );
    assert_eq!(record, ArtifactChecksumRecord::from_hash(hash).unwrap());
}

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
