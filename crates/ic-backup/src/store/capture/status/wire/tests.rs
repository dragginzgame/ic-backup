//! Independently retained wire cases; see the transfer contract and fixture manifest.

use super::{super::tests::plan, *};
use crate::store::{
    BackupDigest,
    capture::{CapturePolicy, tests::wire::frozen},
};
use ic_platform_protocol::management_snapshots::Status;
use serde::Deserialize;

#[test]
fn full_supplier_dto_and_minimal_projection_apply_the_same_original_policy() {
    for name in ["status-full-maxima", "status-minimal"] {
        assert_eq!(plan().validate_status_reply(&frozen(name)), Ok(()));
    }
}

#[test]
fn frozen_legacy_vectors_keep_exact_bytes_and_do_not_gain_missing_module_authority() {
    #[derive(Deserialize)]
    struct Case {
        name: String,
        status: String,
        reply_hex: String,
        payload_checksum: String,
    }
    let cases: Vec<Case> = serde_json::from_str(include_str!("golden.json")).unwrap();
    for case in cases {
        let raw: Vec<u8> = case
            .reply_hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect();
        assert_eq!(
            BackupDigest::compute(&raw).to_string(),
            case.payload_checksum,
            "{}",
            case.name
        );
        let expected = match case.status.as_str() {
            "stopped" => CaptureStatusError::ModuleMismatch,
            "running" => CaptureStatusError::NotStopped(Status::Running),
            "stopping" => CaptureStatusError::NotStopped(Status::Stopping),
            other => panic!("unknown golden status {other}"),
        };
        assert_eq!(
            plan().validate_status_reply(&raw),
            Err(CaptureStatusReplyError::Status(expected)),
            "{}",
            case.name
        );
    }
}

#[test]
fn wire_admission_keeps_original_state_module_and_caller_checks() {
    let p = plan();
    for (name, error) in [
        (
            "status-running",
            CaptureStatusError::NotStopped(Status::Running),
        ),
        (
            "status-stopping",
            CaptureStatusError::NotStopped(Status::Stopping),
        ),
        ("status-no-module", CaptureStatusError::ModuleMismatch),
        ("status-wrong-module", CaptureStatusError::ModuleMismatch),
        ("status-no-controller", CaptureStatusError::NotController),
        (
            "status-duplicate-controller",
            CaptureStatusError::DuplicateController,
        ),
    ] {
        assert_eq!(
            p.validate_status_reply(&frozen(name)),
            Err(CaptureStatusReplyError::Status(error)),
            "{name}"
        );
    }
    let mut request = p.request().clone();
    request.sender = ic_platform_types::Principal::from_slice(&[17, 1]);
    let selected = CapturePolicy::new(request).unwrap();
    assert_eq!(
        selected.validate_status_reply(&frozen("status-original-controller")),
        Err(CaptureStatusReplyError::Status(
            CaptureStatusError::NotController
        ))
    );
    assert_eq!(
        selected.validate_status_reply(&frozen("status-alternate-controller")),
        Ok(())
    );
}

#[test]
fn controllers_are_bounded_before_elements_and_hash_length_before_bytes() {
    let p = plan();
    assert_eq!(
        p.validate_status_reply(&frozen("status-ten-controllers")),
        Ok(())
    );
    // Hostile vectors declare u32::MAX elements but retain no element body.
    for name in [
        "status-eleven-controllers",
        "status-hostile-controller-count",
        "status-hostile-hash-count",
        "status-33-byte-hash",
    ] {
        assert_eq!(
            p.validate_status_reply(&frozen(name)),
            Err(CaptureStatusReplyError::Decode(DecodeError::Limit)),
            "{name}"
        );
    }
    for name in ["status-empty-hash", "status-31-byte-hash"] {
        assert_eq!(
            p.validate_status_reply(&frozen(name)),
            Err(CaptureStatusReplyError::Status(
                CaptureStatusError::ModuleMismatch
            )),
            "{name}"
        );
    }
}

#[test]
fn hash_option_promotion_and_absence_never_invent_the_approved_hash() {
    let p = plan();
    assert_eq!(p.validate_status_reply(&frozen("status-bare-hash")), Ok(()));
    for name in [
        "status-null-hash",
        "status-reserved-hash",
        "status-absent-hash",
        "status-text-hash",
    ] {
        assert!(p.validate_status_reply(&frozen(name)).is_err(), "{name}");
    }
}

#[test]
fn required_status_settings_controllers_and_known_status_variant_cannot_disappear() {
    for name in [
        "status-empty-record",
        "status-settings-only",
        "status-state-only",
        "status-empty-settings",
        "status-unknown-state",
        "status-text-state",
        "status-optional-controllers",
    ] {
        assert_eq!(
            plan().validate_status_reply(&frozen(name)),
            Err(CaptureStatusReplyError::Decode(DecodeError::Type)),
            "{name}"
        );
    }
}

#[test]
fn skipped_work_is_cumulative_across_outer_and_nested_fields() {
    for name in [
        "status-small-padding",
        "status-outer-padding",
        "status-inner-padding",
    ] {
        assert_eq!(
            plan().validate_status_reply(&frozen(name)),
            Ok(()),
            "{name}"
        );
    }
    for name in ["status-combined-padding", "status-excess-padding"] {
        assert_eq!(
            plan().validate_status_reply(&frozen(name)),
            Err(CaptureStatusReplyError::Decode(DecodeError::Limit)),
            "{name}"
        );
    }
}

#[test]
fn malformed_prefixes_extra_arguments_trailing_and_raw_header_limits_refuse() {
    let p = plan();
    let raw = frozen("status-valid-full");
    for end in 0..raw.len() {
        assert!(
            p.validate_status_reply(&raw[..end]).is_err(),
            "prefix {end}"
        );
    }
    assert_eq!(
        p.validate_status_reply(&[raw.as_slice(), &[0]].concat()),
        Err(CaptureStatusReplyError::Decode(DecodeError::Trailing))
    );
    for raw in [b"DIDL\0\0".to_vec(), frozen("status-extra-argument")] {
        assert_eq!(
            p.validate_status_reply(&raw),
            Err(CaptureStatusReplyError::Decode(DecodeError::Arity))
        );
    }
    assert_eq!(
        p.validate_status_reply(&vec![0; MAX_REPLY_BYTES + 1]),
        Err(CaptureStatusReplyError::ReplyTooLarge)
    );
    for raw in [b"DIDL\x41".as_slice(), b"DIDL\x01\x6c\xff\xff\xff\xff\x0f"] {
        assert_eq!(
            p.validate_status_reply(raw),
            Err(CaptureStatusReplyError::Decode(DecodeError::Limit))
        );
    }
}

#[test]
fn resource_statistics_and_version_do_not_become_extra_capture_policy() {
    assert_eq!(
        plan().validate_status_reply(&frozen("status-ignored-resource-maxima")),
        Ok(())
    );
}

#[test]
fn status_visibility_does_not_grant_management_controller_authority() {
    assert_eq!(
        plan().validate_status_reply(&frozen("status-viewer-without-controller")),
        Err(CaptureStatusReplyError::Status(
            CaptureStatusError::NotController
        ))
    );
}
