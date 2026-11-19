//! Strict original source-bound declaration; derived schedules are never persisted.

use super::*;
use crate::store::capture::{
    intent::tests::profile,
    tests::{
        completion::{complete, settled},
        source::maximum,
    },
};

#[test]
fn restore_intent_is_canonical_source_bound_and_keeps_capture_spending() {
    let (mut f, server, mut connection) = settled();
    let receipt = complete(&mut f, &mut connection).unwrap();
    let original = std::fs::read(f.ledger()).unwrap();
    let source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    let request = fixtures::declaration(&source);
    let mut direct = profile();
    direct.request_bytes = 8192;
    let plan = RestorePlan::new(&source, request.clone(), direct.clone()).unwrap();
    assert_eq!(
        plan.validate_source(&source)
            .unwrap()
            .minimum_management_calls(),
        15
    );
    let replay = RestorePlan::decode(plan.encoded()).unwrap();
    assert_eq!(replay.digest(), plan.digest());
    replay.validate_source(&source).unwrap();
    assert_eq!(plan.source_operation_id(), receipt.bundle_id);
    assert_eq!(plan.source_manifest_sha256(), receipt.manifest_sha256);
    let record: serde_json::Value = serde_json::from_slice(plan.encoded()).unwrap();
    for field in [
        "source_requests_sha256",
        "data_calls",
        "minimum_management_calls",
    ] {
        assert!(record.get(field).is_none());
    }
    assert!(RestorePlan::decode(&serde_json::to_vec_pretty(&record).unwrap()).is_err());
    for field in [
        "source_plan_sha256",
        "source_manifest_sha256",
        "module_sha256",
        "source_operation_id",
    ] {
        let mut changed = record.clone();
        changed[field] = if field == "source_operation_id" {
            "ef".repeat(16).into()
        } else {
            "ff".repeat(32).into()
        };
        let bytes = serde_json::to_vec(&changed).unwrap();
        let bytes = canonical(&bytes);
        assert!(
            RestorePlan::decode(&bytes)
                .unwrap()
                .validate_source(&source)
                .is_err()
        );
    }
    for fault in [
        "unknown",
        "nested",
        "missing",
        "duplicate",
        "truncated",
        "old-format",
    ] {
        let mut changed = record.clone();
        let bytes = match fault {
            "unknown" => {
                changed["data_calls"] = 4.into();

                serde_json::to_vec(&changed).unwrap()
            }
            "nested" => {
                changed["limits"]["new"] = true.into();

                serde_json::to_vec(&changed).unwrap()
            }
            "missing" => {
                changed.as_object_mut().unwrap().remove("sender");

                serde_json::to_vec(&changed).unwrap()
            }
            "duplicate" => [
                b"{\"sender\":\"aaaaa-aa\",".as_slice(),
                &plan.encoded()[1..],
            ]
            .concat(),
            "truncated" => plan.encoded()[..plan.encoded().len() - 1].to_vec(),
            _ => {
                changed["format"] = "ic-backup/local-restore-plan/1".into();

                serde_json::to_vec(&changed).unwrap()
            }
        };
        assert!(RestorePlan::decode(&bytes).is_err(), "{fault}");
    }
    for field in [
        "operation",
        "sender",
        "initiator",
        "extent",
        "payload",
        "network",
        "observations",
        "signing",
        "evidence",
    ] {
        let mut changed = request.clone();
        match field {
            "operation" => changed.operation_id = "ef".repeat(16),
            "sender" => changed.sender = request.initiator,
            "initiator" => changed.initiator = request.sender,
            "extent" => changed.limits.extent_bytes = 5,
            "payload" => changed.limits.payload_bytes += 1,
            "network" => changed.limits.network_requests += 1,
            "observations" => changed.limits.observations = 1,
            "signing" => changed.limits.signing_requests += 1,
            _ => changed.limits.evidence_bytes += 1,
        }
        assert_ne!(
            RestorePlan::new(&source, changed, direct.clone())
                .unwrap()
                .digest(),
            plan.digest(),
            "{field}"
        );
    }
    for field in [
        "id",
        "source-id",
        "extent",
        "payload",
        "network",
        "signing",
        "evidence",
        "release",
        "target",
    ] {
        let mut changed = request.clone();
        match field {
            "id" => changed.operation_id = "../bad".into(),
            "source-id" => changed.operation_id = receipt.bundle_id.clone(),
            "extent" => changed.limits.extent_bytes = 0,
            "payload" => changed.limits.payload_bytes = 9,
            "network" => changed.limits.network_requests = 14,
            "signing" => changed.limits.signing_requests = 14,
            "evidence" => changed.limits.evidence_bytes = 0,
            "release" => changed.release = BackupDigest::compute(b"other"),
            _ => changed.target.network = BackupDigest::compute(b"other"),
        }
        assert!(
            RestorePlan::new(&source, changed, direct.clone()).is_err(),
            "{field}"
        );
    }
    direct.request_bytes = 4096;
    assert!(matches!(
        RestorePlan::new(&source, request, direct),
        Err(RestorePlanError::RequestCapacity)
    ));
    assert_eq!(server.count(), 0);
    assert_eq!(std::fs::read(f.ledger()).unwrap(), original);
}

fn canonical(bytes: &[u8]) -> Vec<u8> {
    let record: super::super::PlanRecord = serde_json::from_slice(bytes).unwrap();

    serde_json::to_vec(&record).unwrap()
}
