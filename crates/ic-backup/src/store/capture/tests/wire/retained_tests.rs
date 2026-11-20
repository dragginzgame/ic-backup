//! Independently retained SDK/schema bytes; provenance is in tests/fixtures/README.md.

pub(in crate::store) fn frozen(name: &str) -> Vec<u8> {
    match name {
        "metadata-global-129-bits" => {
            include_bytes!("tests/fixtures/metadata-global-129-bits.bin").as_slice()
        }
        "metadata-null-fields" => {
            include_bytes!("tests/fixtures/metadata-null-fields.bin").as_slice()
        }
        "metadata-reserved-fields" => {
            include_bytes!("tests/fixtures/metadata-reserved-fields.bin").as_slice()
        }
        "metadata-bare-promotion" => {
            include_bytes!("tests/fixtures/metadata-bare-promotion.bin").as_slice()
        }
        "metadata-unknown-source" => {
            include_bytes!("tests/fixtures/metadata-unknown-source.bin").as_slice()
        }
        "metadata-unknown-global" => {
            include_bytes!("tests/fixtures/metadata-unknown-global.bin").as_slice()
        }
        "metadata-unknown-timer" => {
            include_bytes!("tests/fixtures/metadata-unknown-timer.bin").as_slice()
        }
        "metadata-unknown-hook" => {
            include_bytes!("tests/fixtures/metadata-unknown-hook.bin").as_slice()
        }
        "metadata-nonreserved-source" => {
            include_bytes!("tests/fixtures/metadata-nonreserved-source.bin").as_slice()
        }
        "metadata-global-count-over-limit" => {
            include_bytes!("tests/fixtures/metadata-global-count-over-limit.bin").as_slice()
        }
        "metadata-missing-required" => {
            include_bytes!("tests/fixtures/metadata-missing-required.bin").as_slice()
        }
        "metadata-unknown-zero-field" => {
            include_bytes!("tests/fixtures/metadata-unknown-zero-field.bin").as_slice()
        }
        "metadata-missing-optionals" => {
            include_bytes!("tests/fixtures/metadata-missing-optionals.bin").as_slice()
        }
        "upload-metadata-numeric-extremes" => {
            include_bytes!("tests/fixtures/upload-metadata-numeric-extremes.bin").as_slice()
        }
        "snapshot-extra-field" => {
            include_bytes!("tests/fixtures/snapshot-extra-field.bin").as_slice()
        }
        "snapshot-missing-size" => {
            include_bytes!("tests/fixtures/snapshot-missing-size.bin").as_slice()
        }
        "snapshot-signed-timestamp" => {
            include_bytes!("tests/fixtures/snapshot-signed-timestamp.bin").as_slice()
        }
        "snapshot-extra-argument" => {
            include_bytes!("tests/fixtures/snapshot-extra-argument.bin").as_slice()
        }
        "take-snapshot-arguments" => {
            include_bytes!("tests/fixtures/take-snapshot-arguments.bin").as_slice()
        }
        "readback-metadata-extra-argument" => {
            include_bytes!("tests/fixtures/readback-metadata-extra-argument.bin").as_slice()
        }
        "readback-u64-instead-of-record" => {
            include_bytes!("tests/fixtures/readback-u64-instead-of-record.bin").as_slice()
        }
        "unit-u8-argument" => include_bytes!("tests/fixtures/unit-u8-argument.bin").as_slice(),
        "unit-null-argument" => include_bytes!("tests/fixtures/unit-null-argument.bin").as_slice(),
        "status-full-maxima" => include_bytes!("tests/fixtures/status-full-maxima.bin").as_slice(),
        "status-minimal" => include_bytes!("tests/fixtures/status-minimal.bin").as_slice(),
        "status-running" => include_bytes!("tests/fixtures/status-running.bin").as_slice(),
        "status-stopping" => include_bytes!("tests/fixtures/status-stopping.bin").as_slice(),
        "status-no-module" => include_bytes!("tests/fixtures/status-no-module.bin").as_slice(),
        "status-wrong-module" => {
            include_bytes!("tests/fixtures/status-wrong-module.bin").as_slice()
        }
        "status-no-controller" => {
            include_bytes!("tests/fixtures/status-no-controller.bin").as_slice()
        }
        "status-duplicate-controller" => {
            include_bytes!("tests/fixtures/status-duplicate-controller.bin").as_slice()
        }
        "status-original-controller" => {
            include_bytes!("tests/fixtures/status-original-controller.bin").as_slice()
        }
        "status-alternate-controller" => {
            include_bytes!("tests/fixtures/status-alternate-controller.bin").as_slice()
        }
        "status-ten-controllers" => {
            include_bytes!("tests/fixtures/status-ten-controllers.bin").as_slice()
        }
        "status-eleven-controllers" => {
            include_bytes!("tests/fixtures/status-eleven-controllers.bin").as_slice()
        }
        "status-hostile-controller-count" => {
            include_bytes!("tests/fixtures/status-hostile-controller-count.bin").as_slice()
        }
        "status-hostile-hash-count" => {
            include_bytes!("tests/fixtures/status-hostile-hash-count.bin").as_slice()
        }
        "status-33-byte-hash" => {
            include_bytes!("tests/fixtures/status-33-byte-hash.bin").as_slice()
        }
        "status-empty-hash" => include_bytes!("tests/fixtures/status-empty-hash.bin").as_slice(),
        "status-31-byte-hash" => {
            include_bytes!("tests/fixtures/status-31-byte-hash.bin").as_slice()
        }
        "status-bare-hash" => include_bytes!("tests/fixtures/status-bare-hash.bin").as_slice(),
        "status-null-hash" => include_bytes!("tests/fixtures/status-null-hash.bin").as_slice(),
        "status-reserved-hash" => {
            include_bytes!("tests/fixtures/status-reserved-hash.bin").as_slice()
        }
        "status-absent-hash" => include_bytes!("tests/fixtures/status-absent-hash.bin").as_slice(),
        "status-text-hash" => include_bytes!("tests/fixtures/status-text-hash.bin").as_slice(),
        "status-empty-record" => {
            include_bytes!("tests/fixtures/status-empty-record.bin").as_slice()
        }
        "status-settings-only" => {
            include_bytes!("tests/fixtures/status-settings-only.bin").as_slice()
        }
        "status-state-only" => include_bytes!("tests/fixtures/status-state-only.bin").as_slice(),
        "status-empty-settings" => {
            include_bytes!("tests/fixtures/status-empty-settings.bin").as_slice()
        }
        "status-unknown-state" => {
            include_bytes!("tests/fixtures/status-unknown-state.bin").as_slice()
        }
        "status-text-state" => include_bytes!("tests/fixtures/status-text-state.bin").as_slice(),
        "status-optional-controllers" => {
            include_bytes!("tests/fixtures/status-optional-controllers.bin").as_slice()
        }
        "status-small-padding" => {
            include_bytes!("tests/fixtures/status-small-padding.bin").as_slice()
        }
        "status-outer-padding" => {
            include_bytes!("tests/fixtures/status-outer-padding.bin").as_slice()
        }
        "status-inner-padding" => {
            include_bytes!("tests/fixtures/status-inner-padding.bin").as_slice()
        }
        "status-combined-padding" => {
            include_bytes!("tests/fixtures/status-combined-padding.bin").as_slice()
        }
        "status-excess-padding" => {
            include_bytes!("tests/fixtures/status-excess-padding.bin").as_slice()
        }
        "status-valid-full" => include_bytes!("tests/fixtures/status-valid-full.bin").as_slice(),
        "status-extra-argument" => {
            include_bytes!("tests/fixtures/status-extra-argument.bin").as_slice()
        }
        "status-ignored-resource-maxima" => {
            include_bytes!("tests/fixtures/status-ignored-resource-maxima.bin").as_slice()
        }
        "status-viewer-without-controller" => {
            include_bytes!("tests/fixtures/status-viewer-without-controller.bin").as_slice()
        }
        "restore-readback-0" => include_bytes!("tests/fixtures/restore-readback-0.bin").as_slice(),
        "restore-readback-1" => include_bytes!("tests/fixtures/restore-readback-1.bin").as_slice(),
        "restore-readback-2" => include_bytes!("tests/fixtures/restore-readback-2.bin").as_slice(),
        "restore-readback-3" => include_bytes!("tests/fixtures/restore-readback-3.bin").as_slice(),
        "restore-readback-4" => include_bytes!("tests/fixtures/restore-readback-4.bin").as_slice(),
        "restore-readback-5" => include_bytes!("tests/fixtures/restore-readback-5.bin").as_slice(),
        "restore-readback-6" => include_bytes!("tests/fixtures/restore-readback-6.bin").as_slice(),
        _ => panic!("unknown retained fixture {name}"),
    }
    .to_vec()
}

///
/// Case
/// Deserializes the expected values for one frozen wire vector.
///

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    bytes: usize,
    sha256: String,
}

#[test]
fn retained_wire_vectors_match_original_lengths_and_commitments() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("tests/fixtures/manifest.json")).unwrap();

    for case in cases {
        let bytes = frozen(&case.name);
        assert_eq!(bytes.len(), case.bytes, "{}", case.name);
        assert_eq!(
            crate::store::BackupDigest::compute(&bytes).to_string(),
            case.sha256,
            "{}",
            case.name
        );
    }
}
