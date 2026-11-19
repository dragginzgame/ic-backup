use super::*;

#[test]
fn commitments_preserve_sha256_and_canonical_text() {
    let text = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let digest = BackupDigest::compute(b"abc");
    assert_eq!(digest.to_string(), text);
    assert_eq!(text.parse::<BackupDigest>().unwrap(), digest);
    assert_eq!(BackupDigest::from_bytes(*digest.as_bytes()), digest);

    for invalid in [
        text.to_uppercase(),
        text[..62].into(),
        format!("{text}00"),
        "é".repeat(32),
        "g".repeat(64),
    ] {
        assert!(invalid.parse::<BackupDigest>().is_err());
    }
}
