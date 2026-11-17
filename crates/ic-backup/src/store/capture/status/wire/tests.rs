//! Re-authored frozen legacy lifecycle cases; see the capture-intent contract.
use super::super::tests::plan;
use super::*;
use crate::store::BackupDigest;
use candid::{CandidType, Principal, Reserved};
use ic_management_canister_types::{
    CanisterStatusResult, CanisterStatusType, DefiniteCanisterSettings, MemoryMetrics, QueryStats,
};
use serde::Deserialize;

fn encode(value: &impl CandidType) -> Vec<u8> {
    candid::encode_one(value).unwrap()
}

// Test-only wire alternatives with independently specified protocol labels.
#[derive(CandidType, serde::Deserialize)]
struct Settings<C, P> {
    controllers: C,
    padding: P,
}
#[derive(CandidType, serde::Deserialize)]
struct Status<S, C, H, P> {
    status: S,
    settings: C,
    #[serde(rename = "module_hash")]
    hash: H,
    padding: P,
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
            BackupDigest::compute(&raw).to_string(),
            case.payload_checksum,
            "{}",
            case.name
        );
        let expected = if case.status == CanisterStatusType::Stopped {
            CaptureStatusError::ModuleMismatch
        } else {
            CaptureStatusError::NotStopped(match case.status {
                CanisterStatusType::Running => {
                    ic_platform_protocol::management_snapshots::Status::Running
                }
                CanisterStatusType::Stopping => {
                    ic_platform_protocol::management_snapshots::Status::Stopping
                }
                CanisterStatusType::Stopped => unreachable!(),
            })
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
            p.validate_status(&ic_platform_protocol::management_snapshots::CaptureStatus {
                status: match v.status {
                    CanisterStatusType::Stopped =>
                        ic_platform_protocol::management_snapshots::Status::Stopped,
                    CanisterStatusType::Running =>
                        ic_platform_protocol::management_snapshots::Status::Running,
                    CanisterStatusType::Stopping =>
                        ic_platform_protocol::management_snapshots::Status::Stopping,
                },
                settings: ic_platform_protocol::management_snapshots::Settings {
                    controllers: v
                        .settings
                        .controllers
                        .iter()
                        .map(|p| ic_platform_types::Principal::from_slice(p.as_slice()))
                        .collect()
                },
                module_hash: v.module_hash.clone(),
            })
            .map_err(CaptureStatusReplyError::Status)
        );
    }
    let mut request = p.request().clone();
    request.sender = ic_platform_types::Principal::from_slice(&[17, 1]);
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
    impl CandidType for HostileControllers {
        fn _ty() -> candid::types::Type {
            <Vec<Principal>>::ty()
        }
        fn idl_serialize<S: candid::types::Serializer>(
            &self,
            serializer: S,
        ) -> Result<(), S::Error> {
            let _ = serializer.serialize_vec(u32::MAX as usize)?;
            Ok(())
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
    impl CandidType for HostileHash {
        fn _ty() -> candid::types::Type {
            <Vec<u8>>::ty()
        }
        fn idl_serialize<S: candid::types::Serializer>(
            &self,
            serializer: S,
        ) -> Result<(), S::Error> {
            let _ = serializer.serialize_vec(u32::MAX as usize)?;
            Ok(())
        }
    }
    assert_eq!(
        p.validate_status_reply(&encode(&minimal(Some(HostileHash)))),
        Err(CaptureStatusReplyError::Decode(DecodeError::Limit))
    );
    assert_eq!(
        p.validate_status_reply(&encode(&minimal(Some(vec![0x44_u8; 33])))),
        Err(CaptureStatusReplyError::Decode(DecodeError::Limit))
    );
    for len in [0, 31] {
        assert_eq!(
            p.validate_status_reply(&encode(&minimal(Some(vec![0x44_u8; len])))),
            Err(CaptureStatusReplyError::Status(
                CaptureStatusError::ModuleMismatch
            ))
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
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Empty {}

    #[derive(candid::CandidType, serde::Deserialize)]

    struct SettingsOnly {
        settings: DefiniteCanisterSettings,
    }

    #[derive(candid::CandidType, serde::Deserialize)]

    struct StatusOnly {
        status: CanisterStatusType,
    }

    #[derive(candid::CandidType, serde::Deserialize)]

    enum Unknown {
        Deleted,
    }

    let p = plan();
    let settings = || Settings {
        controllers: status(&p).settings.controllers,
        padding: (),
    };
    for raw in [
        encode(&Empty {}),
        encode(&SettingsOnly {
            settings: status(&p).settings,
        }),
        encode(&StatusOnly {
            status: CanisterStatusType::Stopped,
        }),
        encode(&Status {
            status: CanisterStatusType::Stopped,
            settings: Empty {},
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
        candid::encode_args((&v, 0_u8)).unwrap(),
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

pub(in crate::store) fn status(plan: &CapturePolicy) -> CanisterStatusResult {
    CanisterStatusResult {
        status: CanisterStatusType::Stopped,
        ready_for_migration: false,
        version: 23,
        settings: DefiniteCanisterSettings {
            controllers: vec![Principal::from_slice(plan.request().sender.as_slice())],
            ..DefiniteCanisterSettings::default()
        },
        module_hash: Some(plan.request().module_sha256.as_bytes().to_vec()),
        memory_size: 0_u64.into(),
        memory_metrics: MemoryMetrics {
            wasm_memory_size: 0_u64.into(),
            stable_memory_size: 0_u64.into(),
            global_memory_size: 0_u64.into(),
            wasm_binary_size: 0_u64.into(),
            custom_sections_size: 0_u64.into(),
            canister_history_size: 0_u64.into(),
            wasm_chunk_store_size: 0_u64.into(),
            snapshots_size: 0_u64.into(),
            log_memory_store_size: 0_u64.into(),
        },
        cycles: 0_u64.into(),
        reserved_cycles: 0_u64.into(),
        idle_cycles_burned_per_day: 0_u64.into(),
        query_stats: QueryStats {
            num_calls_total: 0_u64.into(),
            num_instructions_total: 0_u64.into(),
            request_payload_bytes_total: 0_u64.into(),
            response_payload_bytes_total: 0_u64.into(),
        },
    }
}

#[test]
fn resource_statistics_and_version_do_not_become_extra_capture_policy() {
    let p = plan();
    let mut observed = status(&p);
    observed.version = u64::MAX;
    observed.cycles = u128::MAX.into();
    observed.ready_for_migration = true;
    observed.settings.compute_allocation = u128::MAX.into();
    let original = observed.clone();
    assert_eq!(p.validate_status_reply(&encode(&observed)), Ok(()));
    assert_eq!(observed, original);
}

#[test]
fn status_visibility_does_not_grant_management_controller_authority() {
    #[derive(CandidType, serde::Deserialize)]
    enum Visibility {
        #[serde(rename = "allowed_viewers")]
        AllowedViewers(Vec<Principal>),
    }
    #[derive(CandidType)]
    struct VisibleSettings {
        controllers: Vec<Principal>,
        status_visibility: Visibility,
    }
    let p = plan();
    let observed = Status {
        status: CanisterStatusType::Stopped,
        settings: VisibleSettings {
            controllers: vec![],
            status_visibility: Visibility::AllowedViewers(vec![Principal::from_slice(
                p.request().sender.as_slice(),
            )]),
        },
        hash: Some(p.request().module_sha256.as_bytes().to_vec()),
        padding: (),
    };
    assert_eq!(
        p.validate_status_reply(&encode(&observed)),
        Err(CaptureStatusReplyError::Status(
            CaptureStatusError::NotController
        ))
    );
}
