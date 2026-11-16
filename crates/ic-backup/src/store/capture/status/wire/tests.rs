//! Re-authored frozen legacy lifecycle cases; see the capture-intent contract.
use super::super::tests::{plan, status};
use super::*;
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::candid_codec::{self, Reserved, Writer, arguments::encode_args};
use serde::Deserialize;

fn encode(value: &impl Encode) -> Vec<u8> {
    candid_codec::encode_one(value, MAX_REPLY_BYTES).unwrap()
}

// Test-only wire alternatives with independently specified protocol labels.
struct Settings<C, P> {
    controllers: C,
    padding: P,
}
impl<C: Encode, P: Encode> Encode for Settings<C, P> {
    const TYPE: &'static Type = &Type::Record(&[(570880087, C::TYPE), (3497875921, P::TYPE)]);
    fn write(&self, out: &mut Writer) -> Result<(), candid_codec::Error> {
        self.controllers.write(out)?;
        self.padding.write(out)
    }
}
struct Status<S, C, H, P> {
    status: S,
    settings: C,
    hash: H,
    padding: P,
}
impl<S: Encode, C: Encode, H: Encode, P: Encode> Encode for Status<S, C, H, P> {
    const TYPE: &'static Type = &Type::Record(&[
        (100394802, S::TYPE),
        (2336062691, C::TYPE),
        (2928387969, H::TYPE),
        (3497875921, P::TYPE),
    ]);
    fn write(&self, out: &mut Writer) -> Result<(), candid_codec::Error> {
        self.status.write(out)?;
        self.settings.write(out)?;
        self.hash.write(out)?;
        self.padding.write(out)
    }
}
fn minimal<H>(hash: H) -> Status<CanisterStatusType, Settings<Vec<Principal>, ()>, H, ()> {
    Status {
        status: CanisterStatusType::Stopped,
        settings: Settings {
            controllers: status(&plan()).settings.controllers,
            padding: (),
        },
        hash,
        padding: (),
    }
}

#[test]
fn full_supplier_dto_and_minimal_projection_apply_the_same_original_policy() {
    let p = plan();
    let mut observed = status(&p);
    observed.version = u64::MAX;
    observed.cycles = u128::MAX.into();
    let raw = encode(&observed);
    assert_eq!(p.validate_status_reply(&raw), Ok(()));
    assert_eq!(encode(&observed), raw);
    assert_eq!(
        p.validate_status_reply(&encode(&minimal(observed.module_hash))),
        Ok(())
    );
}

#[test]
fn frozen_legacy_vectors_keep_exact_bytes_and_do_not_gain_missing_module_authority() {
    #[derive(Deserialize)]
    struct Case {
        name: String,
        status: CanisterStatusType,
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
            Sha256Digest::compute(&raw).to_string(),
            case.payload_checksum,
            "{}",
            case.name
        );
        let expected = if case.status == CanisterStatusType::Stopped {
            CaptureStatusError::ModuleMismatch
        } else {
            CaptureStatusError::NotStopped(case.status)
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
    for alter in [
        |v: &mut CanisterStatusResult| v.status = CanisterStatusType::Running,
        |v: &mut CanisterStatusResult| v.status = CanisterStatusType::Stopping,
        |v: &mut CanisterStatusResult| v.module_hash = None,
        |v: &mut CanisterStatusResult| v.module_hash = Some(vec![0; 32]),
        |v: &mut CanisterStatusResult| v.settings.controllers.clear(),
        |v: &mut CanisterStatusResult| v.settings.controllers.push(v.settings.controllers[0]),
    ] {
        let mut v = status(&p);
        alter(&mut v);
        assert_eq!(
            p.validate_status_reply(&encode(&v)),
            p.validate_status(&v)
                .map_err(CaptureStatusReplyError::Status)
        );
    }
    let mut request = p.request().clone();
    request.sender = Principal::from_slice(&[17, 1]);
    let selected = CapturePolicy::new(request).unwrap();
    assert_eq!(
        selected.validate_status_reply(&encode(&status(&p))),
        Err(CaptureStatusReplyError::Status(
            CaptureStatusError::NotController
        ))
    );
    assert_eq!(
        selected.validate_status_reply(&encode(&status(&selected))),
        Ok(())
    );
}

#[test]
fn controllers_are_bounded_before_elements_and_hash_length_before_bytes() {
    let p = plan();
    let mut v = status(&p);
    v.settings
        .controllers
        .extend((0_u32..9).map(|n| Principal::from_slice(&n.to_be_bytes())));
    assert_eq!(p.validate_status_reply(&encode(&v)), Ok(()));
    v.settings.controllers.push(Principal::anonymous());
    assert_eq!(
        p.validate_status_reply(&encode(&v)),
        Err(CaptureStatusReplyError::Decode(DecodeError::Limit))
    );
    struct HostileControllers;
    impl Encode for HostileControllers {
        const TYPE: &'static Type = <Vec<Principal>>::TYPE;
        fn write(&self, out: &mut Writer) -> Result<(), candid_codec::Error> {
            // u32::MAX controllers, without any elements.
            out.bytes(&[0xff, 0xff, 0xff, 0xff, 0x0f])
        }
    }
    let raw = encode(&Status {
        status: CanisterStatusType::Stopped,
        settings: Settings {
            controllers: HostileControllers,
            padding: (),
        },
        hash: Some([0x44_u8; 32]),
        padding: (),
    });
    assert_eq!(
        p.validate_status_reply(&raw),
        Err(CaptureStatusReplyError::Decode(DecodeError::Limit))
    );
    struct HostileHash;
    impl Encode for HostileHash {
        const TYPE: &'static Type = <Vec<u8>>::TYPE;
        fn write(&self, out: &mut Writer) -> Result<(), candid_codec::Error> {
            out.bytes(&[0xff, 0xff, 0xff, 0xff, 0x0f])
        }
    }
    assert_eq!(
        p.validate_status_reply(&encode(&minimal(Some(HostileHash)))),
        Err(CaptureStatusReplyError::Decode(DecodeError::Malformed))
    );
    for len in [0, 31, 33] {
        assert_eq!(
            p.validate_status_reply(&encode(&minimal(Some(vec![0x44_u8; len])))),
            Err(CaptureStatusReplyError::Decode(DecodeError::Malformed))
        );
    }
}

#[test]
fn hash_option_promotion_and_absence_never_invent_the_approved_hash() {
    let p = plan();
    assert_eq!(
        p.validate_status_reply(&encode(&minimal([0x44_u8; 32]))),
        Ok(())
    );
    for raw in [
        encode(&minimal(())),
        encode(&minimal(Reserved)),
        encode(&minimal(None::<Vec<u8>>)),
        encode(&minimal(Some("wrong-type"))),
    ] {
        assert!(p.validate_status_reply(&raw).is_err());
    }
}

#[test]
fn required_status_settings_controllers_and_known_status_variant_cannot_disappear() {
    struct Empty;
    ic_platform::candid_record!(Empty {});
    struct SettingsOnly {
        settings: DefiniteCanisterSettings,
    }
    ic_platform::candid_record!(SettingsOnly { 2336062691 => settings: DefiniteCanisterSettings });
    struct StatusOnly {
        status: CanisterStatusType,
    }
    ic_platform::candid_record!(StatusOnly { 100394802 => status: CanisterStatusType });
    enum Unknown {
        Deleted,
    }
    ic_platform::candid_variant!(Unknown { 315581913 => Deleted });
    let p = plan();
    let settings = || Settings {
        controllers: status(&p).settings.controllers,
        padding: (),
    };
    for raw in [
        encode(&Empty),
        encode(&SettingsOnly {
            settings: status(&p).settings,
        }),
        encode(&StatusOnly {
            status: CanisterStatusType::Stopped,
        }),
        encode(&Status {
            status: CanisterStatusType::Stopped,
            settings: Empty,
            hash: Some([0x44_u8; 32]),
            padding: (),
        }),
        encode(&Status {
            status: Unknown::Deleted,
            settings: settings(),
            hash: Some([0x44_u8; 32]),
            padding: (),
        }),
        encode(&Status {
            status: "stopped",
            settings: settings(),
            hash: Some([0x44_u8; 32]),
            padding: (),
        }),
        encode(&Status {
            status: CanisterStatusType::Stopped,
            settings: Settings {
                controllers: Some(status(&p).settings.controllers),
                padding: (),
            },
            hash: Some([0x44_u8; 32]),
            padding: (),
        }),
    ] {
        assert_eq!(
            p.validate_status_reply(&raw),
            Err(CaptureStatusReplyError::Decode(DecodeError::Type))
        );
    }
}

#[test]
fn skipped_work_is_cumulative_across_outer_and_nested_fields() {
    let p = plan();
    let raw = |outer, inner| {
        encode(&Status {
            status: CanisterStatusType::Stopped,
            settings: Settings {
                controllers: status(&p).settings.controllers,
                padding: vec![0_u8; inner],
            },
            hash: Some([0x44_u8; 32]),
            padding: vec![0_u8; outer],
        })
    };
    assert_eq!(p.validate_status_reply(&raw(128, 128)), Ok(()));
    // Each field fits alone. Their aggregate traversal exceeds the one shared budget.
    assert_eq!(p.validate_status_reply(&raw(20_000, 0)), Ok(()));
    assert_eq!(p.validate_status_reply(&raw(0, 20_000)), Ok(()));
    assert_eq!(
        p.validate_status_reply(&raw(20_000, 20_000)),
        Err(CaptureStatusReplyError::Decode(DecodeError::Limit))
    );
    assert_eq!(
        p.validate_status_reply(&raw(MAX_SKIPPED_WORK, 0)),
        Err(CaptureStatusReplyError::Decode(DecodeError::Limit))
    );
}

#[test]
fn malformed_prefixes_extra_arguments_trailing_and_raw_header_limits_refuse() {
    let p = plan();
    let v = status(&p);
    let raw = encode(&v);
    for end in 0..raw.len() {
        assert!(
            p.validate_status_reply(&raw[..end]).is_err(),
            "prefix {end}"
        );
    }
    let mut trailing = raw;
    trailing.push(0);
    assert_eq!(
        p.validate_status_reply(&trailing),
        Err(CaptureStatusReplyError::Decode(DecodeError::Trailing))
    );
    for raw in [
        b"DIDL\0\0".to_vec(),
        encode_args(&(&v, 0_u8), MAX_REPLY_BYTES).unwrap(),
    ] {
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
