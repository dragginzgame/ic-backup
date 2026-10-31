//! Keep fixture JSON bytes and independent terminal acceptance exact.

use super::*;
use serde_json::json;

#[test]
fn verification_preserves_json_bytes_and_exact_acceptance_fields() {
    let metadata = ArtifactChecksumRecord::from_bytes(b"source metadata");
    let accepted = ArtifactChecksumRecord::from_bytes(b"complete application acceptance");
    let record = PostLoadVerificationRecord::new(&metadata, &[1, 2], 17, &[3, 4], &accepted);
    let original = json!({
        "original_source_metadata": metadata, "loaded_destination": [1, 2],
        "verification_operation_sequence": 17, "verification_snapshot_id": [3, 4],
        "complete_state_evidence": accepted,
    });
    assert_eq!(
        serde_json::to_vec_pretty(&record).unwrap(),
        serde_json::to_vec_pretty(&original).unwrap()
    );
    let decode = |value: &serde_json::Value| {
        serde_json::from_value::<PostLoadAcceptanceRecord>(value.clone())
    };
    assert!(
        decode(&original)
            .unwrap()
            .matches(&metadata, &[1, 2], &accepted)
    );
    // Diagnostic fields never supplied acceptance in the original reader.
    let mut changed = original.clone();
    changed
        .as_object_mut()
        .unwrap()
        .remove("verification_operation_sequence");
    changed["verification_snapshot_id"] = json!("opaque diagnostic");
    changed["extra"] = json!(true);
    assert!(
        decode(&changed)
            .unwrap()
            .matches(&metadata, &[1, 2], &accepted)
    );
    for field in ["original_source_metadata", "complete_state_evidence"] {
        let mut changed = original.clone();
        changed[field]["hash"] = json!(changed[field]["hash"].as_str().unwrap().to_uppercase());
        assert!(
            !decode(&changed)
                .unwrap()
                .matches(&metadata, &[1, 2], &accepted)
        );
        changed[field]["extra"] = json!(true);
        assert!(decode(&changed).is_err());
        changed.as_object_mut().unwrap().remove(field);
        assert!(decode(&changed).is_err());
    }
    let mut changed = original;
    changed["loaded_destination"] = json!([2, 1]);
    assert!(
        !decode(&changed)
            .unwrap()
            .matches(&metadata, &[1, 2], &accepted)
    );
    changed
        .as_object_mut()
        .unwrap()
        .remove("loaded_destination");
    assert!(decode(&changed).is_err());
}
