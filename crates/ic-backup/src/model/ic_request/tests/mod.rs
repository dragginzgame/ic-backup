//! Exact local codec qualification; no management backend or lifecycle effects.

use super::*;
use crate::model::attempt_journal::OperationBindingRequest;
use serde_json::json;

#[derive(Deserialize)]
struct GoldenCase {
    method: IcManagementMethodRecord,
    target: String,
    snapshot_id: Option<Vec<u8>>,
    arguments_hex: String,
    digest: String,
}
fn cases() -> Vec<GoldenCase> {
    serde_json::from_str(include_str!("golden.json"))
        .expect("independently encoded registered wire cases")
}
fn request(
    method: IcManagementMethodRecord,
    target: &str,
    snapshot_id: Option<Vec<u8>>,
) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: target.into(),
        snapshot_id,
    })
    .expect("admitted typed request")
}
fn binding(record: &IcManagementRequestRecord) -> OperationBindingRecord {
    OperationBindingRecord::new(&OperationBindingRequest {
        intent: "ab".repeat(32),
        operation_sequence: 7,
        network: "cd".repeat(32),
        caller: "2vxsx-fae".into(),
        target: record.target().into(),
        release: "ef".repeat(32),
        request: record.digest().hash().into(),
    })
    .expect("original declared payload binding")
}

#[test]
fn matches_independent_candid_bytes_and_domain_separated_wire_goldens() {
    for case in cases() {
        let record = request(case.method, &case.target, case.snapshot_id.clone());
        assert_eq!(
            crate::hash::hex_bytes(record.arguments()),
            case.arguments_hex,
            "{}",
            case.method.name()
        );
        assert_eq!(
            record.digest().hash(),
            case.digest,
            "{}",
            case.method.name()
        );
        assert_eq!(record.receiver(), "aaaaa-aa");
        assert_eq!(record.snapshot_id(), case.snapshot_id.as_deref());
        let value = serde_json::to_value(&record).expect("declaration");
        assert_eq!(value["method"], case.method.name());
        assert!(value.get("arguments").is_none());
        assert!(value.get("target_bytes").is_none());
        let decoded: IcManagementRequestRecord =
            serde_json::from_value(value).expect("current v1 owner reconstructs bytes");
        assert_eq!(decoded, record);
    }
}

#[test]
fn decodes_exact_host_ingress_shapes_using_pinned_official_types() {
    for case in cases() {
        let record = request(case.method, &case.target, case.snapshot_id.clone());
        match case.method {
            IcManagementMethodRecord::TakeCanisterSnapshot => {
                let decoded: TakeCanisterSnapshotArgs =
                    candid::decode_one(record.arguments()).expect("official capture argument type");
                assert_eq!(decoded.canister_id.to_text(), record.target());
                assert_eq!(decoded.replace_snapshot, None);
                assert_eq!(decoded.uninstall_code, Some(false));
                assert_eq!(decoded.sender_canister_version, None);
                assert_eq!(case.method.effect(), IcRequestEffect::Mutation);
            }
            IcManagementMethodRecord::LoadCanisterSnapshot => {
                let decoded: LoadCanisterSnapshotArgs =
                    candid::decode_one(record.arguments()).expect("official load argument type");
                assert_eq!(decoded.canister_id.to_text(), record.target());
                assert_eq!(Some(decoded.snapshot_id), case.snapshot_id);
                assert_eq!(decoded.sender_canister_version, None);
                assert_eq!(case.method.effect(), IcRequestEffect::Mutation);
            }
            _ => {
                let decoded: CanisterIdRecord =
                    candid::decode_one(record.arguments()).expect("official target argument type");
                assert_eq!(decoded.canister_id.to_text(), record.target());
                assert_eq!(record.snapshot_id(), None);
            }
        }
    }
}

#[test]
fn binds_method_routing_target_and_exact_snapshot_bytes_with_canonical_aliases() {
    let target = "renrk-eyaaa-aaaaa-aaada-cai";
    let stop = request(IcManagementMethodRecord::StopCanister, target, None);
    let equivalent = request(
        IcManagementMethodRecord::StopCanister,
        &target.to_uppercase(),
        None,
    );
    assert_eq!(stop, equivalent);
    let start = request(IcManagementMethodRecord::StartCanister, target, None);
    assert_eq!(
        stop.arguments(),
        start.arguments(),
        "method identity cannot rely on argument identity"
    );
    assert_ne!(stop.digest(), start.digest());
    let other = request(IcManagementMethodRecord::StopCanister, "aaaaa-aa", None);
    assert_ne!(stop.arguments(), other.arguments());
    assert_ne!(stop.digest(), other.digest());
    let first = request(
        IcManagementMethodRecord::LoadCanisterSnapshot,
        target,
        Some(vec![0, 255]),
    );
    let changed = request(
        IcManagementMethodRecord::LoadCanisterSnapshot,
        target,
        Some(vec![255, 0]),
    );
    assert_ne!(first.arguments(), changed.arguments());
    assert_ne!(first.digest(), changed.digest());
}

#[test]
fn validates_original_mutation_and_separately_reserved_observation_payloads() {
    let mutation = request(
        IcManagementMethodRecord::TakeCanisterSnapshot,
        "aaaaa-aa",
        None,
    );
    let original = binding(&mutation);
    mutation
        .validate_mutation_binding(&original)
        .expect("exact original payload");
    let observer = request(
        IcManagementMethodRecord::ListCanisterSnapshots,
        "aaaaa-aa",
        None,
    );
    observer
        .validate_observation_binding(&original, &observer.digest())
        .expect("same target and separate observation reservation");
    assert_eq!(
        observer.validate_mutation_binding(&original),
        Err(IcRequestError::EffectMismatch {
            expected: IcRequestEffect::Mutation
        })
    );
    assert_eq!(
        mutation.validate_observation_binding(&original, &mutation.digest()),
        Err(IcRequestError::EffectMismatch {
            expected: IcRequestEffect::Observation
        })
    );
    assert_eq!(
        observer.validate_observation_binding(&original, &mutation.digest()),
        Err(IcRequestError::DigestMismatch)
    );
    let wrong_target = request(
        IcManagementMethodRecord::StopCanister,
        "renrk-eyaaa-aaaaa-aaada-cai",
        None,
    );
    assert_eq!(
        wrong_target.validate_mutation_binding(&original),
        Err(IcRequestError::TargetMismatch)
    );
    let wrong_method = request(IcManagementMethodRecord::StartCanister, "aaaaa-aa", None);
    assert_eq!(
        wrong_method.validate_mutation_binding(&original),
        Err(IcRequestError::DigestMismatch)
    );
    let wrong_observer = request(
        IcManagementMethodRecord::CanisterStatus,
        "renrk-eyaaa-aaaaa-aaada-cai",
        None,
    );
    assert_eq!(
        wrong_observer.validate_observation_binding(&original, &wrong_observer.digest()),
        Err(IcRequestError::TargetMismatch)
    );
    let status = request(IcManagementMethodRecord::CanisterStatus, "aaaaa-aa", None);
    assert_ne!(status.digest(), observer.digest());
    assert_eq!(status.method().effect(), IcRequestEffect::Observation);
}

#[test]
fn rejects_non_neutral_snapshot_arguments_and_bounds_raw_bytes() {
    for case in cases() {
        if case.method != IcManagementMethodRecord::LoadCanisterSnapshot {
            assert_eq!(
                IcManagementRequestRecord::new(IcManagementRequest {
                    method: case.method,
                    target: case.target,
                    snapshot_id: Some(vec![1])
                })
                .expect_err("unused snapshot cannot be silently ignored"),
                IcRequestError::UnexpectedSnapshot
            );
        }
    }
    for bytes in [vec![], vec![1; MAX_IC_SNAPSHOT_ID_BYTES + 1]] {
        assert_eq!(
            IcManagementRequestRecord::new(IcManagementRequest {
                method: IcManagementMethodRecord::LoadCanisterSnapshot,
                target: "aaaaa-aa".into(),
                snapshot_id: Some(bytes)
            })
            .expect_err("snapshot bound"),
            IcRequestError::InvalidSnapshotId
        );
    }
    assert_eq!(
        IcManagementRequestRecord::new(IcManagementRequest {
            method: IcManagementMethodRecord::LoadCanisterSnapshot,
            target: "aaaaa-aa".into(),
            snapshot_id: None
        })
        .expect_err("snapshot required"),
        IcRequestError::SnapshotRequired
    );
    let max = request(
        IcManagementMethodRecord::LoadCanisterSnapshot,
        "aaaaa-aa",
        Some(vec![255; MAX_IC_SNAPSHOT_ID_BYTES]),
    );
    assert_eq!(
        max.snapshot_id().expect("snapshot").len(),
        MAX_IC_SNAPSHOT_ID_BYTES
    );
    assert!(max.arguments().len() <= MAX_IC_ARGUMENT_BYTES);
    assert!(
        serde_json::to_vec_pretty(&max)
            .expect("maximum canonical record")
            .len() as u64
            <= MAX_IC_REQUEST_RECORD_BYTES
    );
    for target in ["not-a-principal".into(), "a".repeat(64)] {
        assert_eq!(
            IcManagementRequestRecord::new(IcManagementRequest {
                method: IcManagementMethodRecord::StopCanister,
                target,
                snapshot_id: None
            })
            .expect_err("bounded principal"),
            IcRequestError::InvalidTarget
        );
    }
}

#[test]
fn admits_only_closed_required_v1_fields_and_bounded_byte_arrays() {
    let record = request(
        IcManagementMethodRecord::LoadCanisterSnapshot,
        "aaaaa-aa",
        Some(vec![1, 2]),
    );
    let value = serde_json::to_value(&record).expect("record");
    for field in ["version", "method", "target", "snapshot_id"] {
        let mut changed = value.clone();
        changed.as_object_mut().expect("object").remove(field);
        assert!(
            serde_json::from_value::<IcManagementRequestRecord>(changed).is_err(),
            "{field}"
        );
    }
    for field in [
        "arguments",
        "target_bytes",
        "receiver",
        "query",
        "replace_snapshot",
        "uninstall_code",
        "sender_canister_version",
        "preflight_accepted",
        "digest",
    ] {
        let mut changed = value.clone();
        changed[field] = true.into();
        assert!(
            serde_json::from_value::<IcManagementRequestRecord>(changed).is_err(),
            "{field}"
        );
    }
    for snapshot in [
        json!([]),
        json!(vec![1; 257]),
        json!([256]),
        json!([-1]),
        json!([1.5]),
        json!("0102"),
        json!(null),
    ] {
        let mut changed = value.clone();
        changed["snapshot_id"] = snapshot;
        assert!(serde_json::from_value::<IcManagementRequestRecord>(changed).is_err());
    }
    for method in [
        "delete_canister",
        "delete_canister_snapshot",
        "upload_canister_snapshot_data",
        "create_snapshot",
        "stop",
        " arbitrary",
    ] {
        let mut changed = value.clone();
        changed["method"] = method.into();
        assert!(
            serde_json::from_value::<IcManagementRequestRecord>(changed).is_err(),
            "{method}"
        );
    }
    let mut changed = value;
    changed["version"] = 2.into();
    assert!(serde_json::from_value::<IcManagementRequestRecord>(changed).is_err());
    let duplicate = r#"{"version":1,"method":"stop_canister","target":"aaaaa-aa","snapshot_id":null,"snapshot_id":null}"#;
    assert!(serde_json::from_str::<IcManagementRequestRecord>(duplicate).is_err());
    let missing_nullable = r#"{"version":1,"method":"stop_canister","target":"aaaaa-aa"}"#;
    assert!(serde_json::from_str::<IcManagementRequestRecord>(missing_nullable).is_err());
}
