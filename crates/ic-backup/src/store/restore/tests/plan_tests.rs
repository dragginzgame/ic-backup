//! Original-byte restore plan admission against authenticated held sources.
use crate::store::{
    RestorePlan, RestorePlanError, RestoreTransferError,
    capture::test_fixtures::{
        completion::{complete, settled},
        http::runtime,
        source::maximum,
    },
    restore::tests::fixtures::declaration,
};
use ic_host_artifacts::artifact::Sha256Digest;
use std::fs;

#[test]
fn restore_plan_binds_original_source_and_retains_exact_replayed_bytes() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let ledger_path = f.ledger();
    let ledger = fs::read(&ledger_path).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    let plan = RestorePlan::new(&source, declaration(&source)).unwrap();
    assert_eq!(plan.source(), &receipt);
    assert_eq!(
        (
            plan.transfer().data_calls(),
            plan.transfer().minimum_management_calls()
        ),
        (4, 15)
    );
    let record: serde_json::Value = serde_json::from_slice(plan.encoded()).unwrap();
    assert_eq!(record["source_plan_sha256"], f.p.digest().to_string());
    assert_eq!(
        record["source_manifest_sha256"],
        receipt.manifest_sha256.to_string()
    );
    assert_eq!(record["module_sha256"], receipt.module_sha256.to_string());
    assert_eq!(record["sender"], "aaaaa-aa");
    let replay = RestorePlan::decode(&source, plan.encoded().to_vec()).unwrap();
    assert_eq!(replay.encoded(), plan.encoded());
    assert_eq!(replay.digest(), plan.digest());
    let pretty = serde_json::to_vec_pretty(&record).unwrap();
    let replay = RestorePlan::decode(&source, pretty.clone()).unwrap();
    assert_eq!(replay.encoded(), pretty);
    assert_ne!(replay.digest(), plan.digest());
    assert_eq!(replay.source(), &receipt);
    assert_eq!(fs::read(ledger_path).unwrap(), ledger);
    assert_eq!(server.count(), 1);
    assert!(!f.root.join("operations").join("cd".repeat(16)).exists());
}

#[test]
fn restore_original_bindings_and_closed_schema_refuse_substitutions() {
    let (mut f, _, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    let plan = RestorePlan::new(&source, declaration(&source)).unwrap();
    let record: serde_json::Value = serde_json::from_slice(plan.encoded()).unwrap();
    for field in [
        "module_sha256",
        "source_plan_sha256",
        "source_manifest_sha256",
        "source_requests_sha256",
        "source_operation_id",
        "data_calls",
        "minimum_management_calls",
    ] {
        let mut changed = record.clone();
        changed[field] = match field {
            "source_operation_id" => serde_json::json!("ab".repeat(16)),
            "data_calls" | "minimum_management_calls" => serde_json::json!(123),
            _ => serde_json::json!("ff".repeat(32)),
        };
        assert!(matches!(
            RestorePlan::decode(&source, serde_json::to_vec(&changed).unwrap()),
            Err(RestorePlanError::SourceBinding)
        ));
    }
    for field in ["sender", "format"] {
        let mut changed = record.clone();
        changed[field] = if field == "sender" {
            "AAAAA-AA".into()
        } else {
            "ic-backup/local-restore-plan/2".into()
        };
        assert!(matches!(
            RestorePlan::decode(&source, serde_json::to_vec(&changed).unwrap()),
            Err(RestorePlanError::InvalidPlan)
        ));
    }
    for fault in [
        "unknown",
        "nested",
        "missing",
        "duplicate",
        "truncated",
        "oversized",
    ] {
        let mut changed = record.clone();
        let bytes = match fault {
            "unknown" => {
                changed["unknown"] = true.into();
                serde_json::to_vec(&changed).unwrap()
            }
            "nested" => {
                changed["limits"]["unknown"] = true.into();
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
            "oversized" => vec![b' '; 1024 * 1024 + 1],
            _ => unreachable!(),
        };
        assert!(RestorePlan::decode(&source, bytes).is_err(), "{fault}");
    }
}

#[test]
fn each_new_identity_and_limit_changes_approval_without_changing_capture() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    let request = declaration(&source);
    let original = RestorePlan::new(&source, request.clone()).unwrap();
    for field in [
        "operation",
        "initiator",
        "sender",
        "payload",
        "extent",
        "network",
        "observations",
        "signing",
        "evidence",
    ] {
        let mut changed = request.clone();
        match field {
            "operation" => changed.operation_id = "ef".repeat(16),
            "initiator" => changed.initiator = request.sender,
            "sender" => changed.sender = request.initiator,
            "payload" => changed.limits.payload_bytes = 11,
            "extent" => changed.limits.extent_bytes = 5,
            "network" => changed.limits.network_requests = 16,
            "observations" => changed.limits.observations = 1,
            "signing" => changed.limits.signing_requests = 16,
            "evidence" => changed.limits.evidence_bytes += 1,
            _ => unreachable!(),
        }
        let plan = RestorePlan::new(&source, changed).unwrap();
        assert_ne!(plan.digest(), original.digest(), "{field}");
        assert_eq!(plan.source(), &receipt);
        if field == "extent" {
            let a: serde_json::Value = serde_json::from_slice(original.encoded()).unwrap();
            let b: serde_json::Value = serde_json::from_slice(plan.encoded()).unwrap();
            assert_ne!(a["source_requests_sha256"], b["source_requests_sha256"]);
        }
    }
    assert_eq!(server.count(), 1);
}

#[test]
fn restore_selection_and_finite_budget_refusals_precede_any_intent() {
    let (mut f, _, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    for field in [
        "id",
        "source-id",
        "extent",
        "evidence",
        "observations",
        "network-range",
        "signing-range",
    ] {
        let mut request = declaration(&source);
        match field {
            "id" => request.operation_id = "../other".into(),
            "source-id" => request.operation_id.clone_from(&receipt.bundle_id),
            "extent" => request.limits.extent_bytes = 0,
            "evidence" => request.limits.evidence_bytes = 0,
            "observations" => request.limits.observations = 16,
            "network-range" => request.limits.network_requests = 8193,
            "signing-range" => request.limits.signing_requests = 0,
            _ => unreachable!(),
        }
        assert!(
            matches!(
                RestorePlan::new(&source, request),
                Err(RestorePlanError::InvalidPlan)
            ),
            "{field}"
        );
    }
    let mut request = declaration(&source);
    request.trust_root = Sha256Digest::compute(b"other root");
    assert!(matches!(
        RestorePlan::new(&source, request),
        Err(RestorePlanError::SourceBinding)
    ));
    let mut request = declaration(&source);
    request.release = Sha256Digest::compute(b"other release");
    assert!(matches!(
        RestorePlan::new(&source, request),
        Err(RestorePlanError::Transfer(RestoreTransferError::Selection))
    ));
    let mut request = declaration(&source);
    request.limits.signing_requests = 14;
    assert!(matches!(
        RestorePlan::new(&source, request),
        Err(RestorePlanError::SigningBudget {
            required: 15,
            limit: 14
        })
    ));
    let mut request = declaration(&source);
    request.limits.network_requests = 14;
    assert!(matches!(
        RestorePlan::new(&source, request),
        Err(RestorePlanError::Transfer(RestoreTransferError::Calls {
            required: 15,
            limit: 14
        }))
    ));
}
