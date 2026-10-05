//! Independent wire fixtures and native bounded projection/rejection cases.

use super::*;
use crate::model::{control_authority::MAX_CONTROLLERS, ic_request::IcManagementRequest};
use candid::{CandidType, Principal};
use ic_management_canister_types::{
    CanisterStatusResult, DefiniteCanisterSettings, MemoryMetrics, QueryStats,
};
use serde::Deserialize;

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";

#[derive(Deserialize)]
struct GoldenCase {
    name: String,
    method: IcManagementMethodRecord,
    target: String,
    snapshot_id: Option<Vec<u8>>,
    request_digest: String,
    reply_hex: String,
    payload_checksum: String,
    digest: String,
    status: Option<CanisterStatusType>,
    controllers: Vec<String>,
}

#[derive(CandidType, Deserialize)]
struct Settings {
    controllers: Vec<Principal>,
}
#[derive(CandidType, Deserialize)]
struct Status {
    status: CanisterStatusType,
    settings: Settings,
}

fn request(method: IcManagementMethodRecord, target: &str) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: target.into(),
        snapshot_id: (method == IcManagementMethodRecord::LoadCanisterSnapshot)
            .then(|| vec![0, 255, 17]),
    })
    .expect("existing request owner")
}

fn status(controllers: Vec<Principal>) -> Status {
    Status {
        status: CanisterStatusType::Stopping,
        settings: Settings { controllers },
    }
}

fn bytes(hex: &str) -> Vec<u8> {
    let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
    assert_eq!(remainder, b"", "complete golden bytes");
    pairs
        .iter()
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16).expect("hex byte")
        })
        .collect()
}

#[test]
fn every_registered_independent_wire_and_hash_fixture_uses_production_admission() {
    let cases: Vec<GoldenCase> =
        serde_json::from_str(include_str!("golden.json")).expect("registry");
    for case in cases {
        let request = IcManagementRequestRecord::new(IcManagementRequest {
            method: case.method,
            target: case.target,
            snapshot_id: case.snapshot_id,
        })
        .expect("golden request");
        assert_eq!(
            request.digest().hash(),
            case.request_digest,
            "{}",
            case.name
        );
        let raw = bytes(&case.reply_hex);
        let reply = IcLifecycleReply::decode(&request, &raw).expect("production decoder");
        assert_eq!(
            reply.payload_checksum().hash(),
            case.payload_checksum,
            "{}",
            case.name
        );
        assert_eq!(reply.digest().hash(), case.digest, "{}", case.name);
        assert_eq!(reply.request(), &request);
        if let Some(expected_status) = case.status {
            let official: Status =
                candid::decode_one(&raw).expect("upstream status enum/projection");
            assert_eq!(official.status, expected_status);
            let IcLifecycleReplyKind::Status(info) = reply.kind() else {
                panic!("status shape")
            };
            assert_eq!(info.status(), expected_status);
            let mut expected = case.controllers;
            expected.sort();
            assert_eq!(info.controllers().principals(), expected);
            let mut official = official
                .settings
                .controllers
                .into_iter()
                .map(|p| p.to_text())
                .collect::<Vec<_>>();
            official.sort();
            assert_eq!(official, expected);
        } else {
            candid::decode_args::<()>(&raw).expect("official empty tuple");
            assert_eq!(reply.kind(), &IcLifecycleReplyKind::Acknowledgement);
        }
    }
}

#[test]
fn accepts_complete_pinned_sdk_status_without_claiming_unprojected_metadata() {
    let controllers = vec![Principal::anonymous(), Principal::management_canister()];
    let value = CanisterStatusResult {
        status: CanisterStatusType::Stopped,
        ready_for_migration: false,
        version: u64::MAX,
        settings: DefiniteCanisterSettings {
            controllers,
            ..DefiniteCanisterSettings::default()
        },
        module_hash: Some(vec![0; 32]),
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
        cycles: u128::MAX.into(),
        reserved_cycles: 0_u64.into(),
        idle_cycles_burned_per_day: 0_u64.into(),
        query_stats: QueryStats {
            num_calls_total: 0_u64.into(),
            num_instructions_total: 0_u64.into(),
            request_payload_bytes_total: 0_u64.into(),
            response_payload_bytes_total: 0_u64.into(),
        },
    };
    let request = request(IcManagementMethodRecord::CanisterStatus, TARGET);
    let raw = candid::encode_one(&value).expect("SDK result");
    assert_eq!(
        candid::decode_one::<CanisterStatusResult>(&raw).expect("official full result"),
        value
    );
    let reply = IcLifecycleReply::decode(&request, &raw).expect("required projection");
    let IcLifecycleReplyKind::Status(info) = reply.kind() else {
        panic!("status")
    };
    assert_eq!(info.status(), CanisterStatusType::Stopped);
    assert_eq!(info.controllers().principals(), ["2vxsx-fae", "aaaaa-aa"]);
}

#[test]
fn retains_explicit_empty_and_maximum_controllers_and_rejects_excess_or_duplicates() {
    let request = request(IcManagementMethodRecord::CanisterStatus, TARGET);
    for count in [0, MAX_CONTROLLERS] {
        let controllers = (0..count)
            .map(|i| Principal::from_slice(&i.to_be_bytes()))
            .collect();
        let raw = candid::encode_one(status(controllers)).expect("bounded controllers");
        let reply = IcLifecycleReply::decode(&request, &raw).expect("bounded projection");
        let IcLifecycleReplyKind::Status(info) = reply.kind() else {
            panic!("status")
        };
        assert_eq!(info.controllers().principals().len(), count);
    }
    let too_many = (0..=MAX_CONTROLLERS)
        .map(|i| Principal::from_slice(&i.to_be_bytes()))
        .collect();
    let raw = candid::encode_one(status(too_many)).expect("excess controllers");
    assert_eq!(
        IcLifecycleReply::decode(&request, &raw).unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
    let raw = candid::encode_one(status(vec![Principal::anonymous(); 2])).expect("duplicates");
    assert_eq!(
        IcLifecycleReply::decode(&request, &raw).unwrap_err(),
        IcLifecycleReplyError::Controllers(ControlObservationError::DuplicateController)
    );
}

#[test]
fn acknowledgements_require_exact_empty_tuple_for_each_supported_mutation() {
    for method in [
        IcManagementMethodRecord::StopCanister,
        IcManagementMethodRecord::StartCanister,
        IcManagementMethodRecord::LoadCanisterSnapshot,
    ] {
        let request = request(method, TARGET);
        let canonical = b"DIDL\0\0";
        assert_eq!(
            IcLifecycleReply::decode(&request, canonical)
                .expect("empty tuple")
                .kind(),
            &IcLifecycleReplyKind::Acknowledgement
        );
        for end in 0..canonical.len() {
            assert_eq!(
                IcLifecycleReply::decode(&request, &canonical[..end]).unwrap_err(),
                IcLifecycleReplyError::InvalidReply
            );
        }
        // A Candid null argument and an unused type table are not canonical empty acknowledgements.
        for invalid in [
            b"DIDL\0\x01\x7f".to_vec(),
            b"DIDL\x01\x6d\x7b\0".to_vec(),
            b"DIDL\0\0\0".to_vec(),
            candid::encode_one(0_u8).expect("extra argument"),
        ] {
            assert_eq!(
                IcLifecycleReply::decode(&request, &invalid).unwrap_err(),
                IcLifecycleReplyError::InvalidReply
            );
        }
    }
}

#[test]
fn rejects_unsupported_methods_oversize_input_and_wrong_reply_shapes() {
    for method in [
        IcManagementMethodRecord::TakeCanisterSnapshot,
        IcManagementMethodRecord::ListCanisterSnapshots,
    ] {
        let request = request(method, TARGET);
        assert_eq!(
            IcLifecycleReply::decode(&request, b"DIDL\0\0").unwrap_err(),
            IcLifecycleReplyError::UnsupportedMethod { method }
        );
    }
    let oversized = vec![0; MAX_IC_LIFECYCLE_REPLY_BYTES + 1];
    for method in [
        IcManagementMethodRecord::CanisterStatus,
        IcManagementMethodRecord::StopCanister,
    ] {
        let request = request(method, TARGET);
        assert_eq!(
            IcLifecycleReply::decode(&request, &oversized).unwrap_err(),
            IcLifecycleReplyError::ReplyTooLarge
        );
    }
    let request = request(IcManagementMethodRecord::CanisterStatus, TARGET);
    assert_eq!(
        IcLifecycleReply::decode(&request, b"DIDL\0\0").unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
}

#[test]
fn requires_status_settings_and_nonoptional_controller_fields() {
    #[derive(CandidType)]
    struct MissingSettings {
        status: CanisterStatusType,
    }
    #[derive(CandidType)]
    struct MissingStatus {
        settings: Settings,
    }
    #[derive(CandidType)]
    struct EmptySettings {}
    #[derive(CandidType)]
    struct MissingControllers {
        status: CanisterStatusType,
        settings: EmptySettings,
    }
    #[derive(CandidType)]
    struct OptionalSettings {
        controllers: Option<Vec<Principal>>,
    }
    #[derive(CandidType)]
    struct OptionalControllers {
        status: CanisterStatusType,
        settings: OptionalSettings,
    }
    #[derive(CandidType)]
    struct WrongStatus {
        status: String,
        settings: Settings,
    }
    #[derive(CandidType, Deserialize)]
    enum UnknownStatus {
        #[serde(rename = "deleted")]
        Deleted,
    }
    #[derive(CandidType)]
    struct Unknown {
        status: UnknownStatus,
        settings: Settings,
    }
    let request = request(IcManagementMethodRecord::CanisterStatus, TARGET);
    for invalid in [
        candid::encode_one(MissingSettings {
            status: CanisterStatusType::Running,
        })
        .expect("missing settings"),
        candid::encode_one(MissingStatus {
            settings: Settings {
                controllers: vec![],
            },
        })
        .expect("missing status"),
        candid::encode_one(MissingControllers {
            status: CanisterStatusType::Running,
            settings: EmptySettings {},
        })
        .expect("missing controllers"),
        candid::encode_one(OptionalControllers {
            status: CanisterStatusType::Running,
            settings: OptionalSettings { controllers: None },
        })
        .expect("optional controllers"),
        candid::encode_one(WrongStatus {
            status: "running".into(),
            settings: Settings {
                controllers: vec![],
            },
        })
        .expect("string status"),
        candid::encode_one(Unknown {
            status: UnknownStatus::Deleted,
            settings: Settings {
                controllers: vec![],
            },
        })
        .expect("unknown status"),
    ] {
        assert_eq!(
            IcLifecycleReply::decode(&request, &invalid).unwrap_err(),
            IcLifecycleReplyError::InvalidReply
        );
    }
}

#[test]
fn rejects_status_truncations_extra_arguments_and_trailing_bytes() {
    let request = request(IcManagementMethodRecord::CanisterStatus, TARGET);
    let value = status(vec![Principal::anonymous()]);
    let raw = candid::encode_one(&value).expect("fixture");
    for end in 0..raw.len() {
        assert_eq!(
            IcLifecycleReply::decode(&request, &raw[..end]).unwrap_err(),
            IcLifecycleReplyError::InvalidReply
        );
    }
    let extra = candid::encode_args((&value, 0_u8)).expect("extra argument");
    let mut trailing = raw.clone();
    trailing.push(0);
    for invalid in [extra, trailing] {
        assert_eq!(
            IcLifecycleReply::decode(&request, &invalid).unwrap_err(),
            IcLifecycleReplyError::InvalidReply
        );
    }
}

#[test]
fn skips_unprojected_fields_with_finite_work_and_preserves_exact_raw_evidence() {
    #[derive(CandidType)]
    struct Extended {
        status: CanisterStatusType,
        settings: Settings,
        padding: Vec<u8>,
    }
    let request = request(IcManagementMethodRecord::CanisterStatus, TARGET);
    let base = candid::encode_one(status(vec![])).expect("minimal projection");
    let extended = candid::encode_one(Extended {
        status: CanisterStatusType::Stopping,
        settings: Settings {
            controllers: vec![],
        },
        padding: vec![7; 128],
    })
    .expect("unprojected bytes");
    let left = IcLifecycleReply::decode(&request, &base).expect("base");
    let right = IcLifecycleReply::decode(&request, &extended).expect("bounded skipped field");
    assert_eq!(left.kind(), right.kind());
    assert_ne!(left.payload_checksum(), right.payload_checksum());
    assert_ne!(left.digest(), right.digest());
    let excessive = candid::encode_one(Extended {
        status: CanisterStatusType::Stopping,
        settings: Settings {
            controllers: vec![],
        },
        padding: vec![7; 64 * 1024],
    })
    .expect("excess skipped work");
    assert!(excessive.len() < MAX_IC_LIFECYCLE_REPLY_BYTES);
    assert_eq!(
        IcLifecycleReply::decode(&request, &excessive).unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
}

#[test]
fn acknowledgement_hashes_bind_exact_target_method_and_load_snapshot_bytes() {
    let stop = request(IcManagementMethodRecord::StopCanister, TARGET);
    let start = request(IcManagementMethodRecord::StartCanister, TARGET);
    let other = request(IcManagementMethodRecord::StopCanister, "aaaaa-aa");
    let load = request(IcManagementMethodRecord::LoadCanisterSnapshot, TARGET);
    let rebound = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::LoadCanisterSnapshot,
        target: TARGET.into(),
        snapshot_id: Some(vec![1]),
    })
    .expect("other exact raw ID");
    let original = IcLifecycleReply::decode(&stop, b"DIDL\0\0").expect("stop");
    for request in [&start, &other, &load, &rebound] {
        let reply = IcLifecycleReply::decode(request, b"DIDL\0\0").expect("declared association");
        assert_eq!(original.payload_checksum(), reply.payload_checksum());
        assert_ne!(original.digest(), reply.digest());
    }
    assert_ne!(
        IcLifecycleReply::decode(&load, b"DIDL\0\0")
            .expect("load")
            .digest(),
        IcLifecycleReply::decode(&rebound, b"DIDL\0\0")
            .expect("other load")
            .digest()
    );
}

#[test]
fn rejects_excess_type_tables_and_huge_declared_controller_count() {
    let request = request(IcManagementMethodRecord::CanisterStatus, TARGET);
    let raw = candid::encode_one(status(vec![])).expect("minimal status projection");
    // Four definitions, two one-byte argument declarations and a two-byte body:
    // status variant index and empty controller count. Added definitions leave
    // existing references intact and are valid Candid, but exceed the local cap.
    assert_eq!(raw[4], 4);
    let definitions_end = raw.len() - 4;
    let mut excessive = raw[..definitions_end].to_vec();
    excessive[4] = 65;
    for _ in 0..61 {
        excessive.extend_from_slice(&[0x6d, 0x7b]);
    }
    excessive.extend_from_slice(&raw[definitions_end..]);
    assert_eq!(
        candid::decode_one::<Status>(&excessive)
            .expect("valid unused types")
            .status,
        CanisterStatusType::Stopping
    );
    assert_eq!(
        IcLifecycleReply::decode(&request, &excessive).unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
    // Replace the empty vector length in the minimal body with a hostile u32::MAX
    // declaration; no matching data and no allocation from its length are admitted.
    let body = raw.len() - 2;
    let controller_offset = (body..raw.len())
        .find(|offset| raw[*offset] == 0)
        .expect("empty controller count; stopping variant is nonzero");
    let mut huge = raw[..controller_offset].to_vec();
    huge.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
    huge.extend_from_slice(&raw[controller_offset + 1..]);
    assert_eq!(
        IcLifecycleReply::decode(&request, &huge).unwrap_err(),
        IcLifecycleReplyError::InvalidReply
    );
}
