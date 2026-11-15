//! Real direct-key signatures, original source plans and no IO/dispatch authority.
use crate::store::{
    RestorePlan, StatusRequestError,
    capture::test_fixtures::{
        completion::{complete, settled},
        http::runtime,
        signatures::{Scheme, change},
        source::maximum,
    },
    restore::tests::fixtures::{admission, declaration, prepared},
};
use ic_platform::{
    Principal, candid_codec, ic_protocol::EnvelopeContent,
    management_canister_types::CanisterIdRecord,
};
use std::fs;

#[test]
fn restore_request_authenticates_both_direct_keys_and_preserves_original_bytes() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (mut f, server, mut provider) = settled();
        let receipt = complete(&mut f, &mut provider).unwrap();
        let ledger = f.ledger();
        let before = fs::read(&ledger).unwrap();
        let source = runtime()
            .block_on(f.store.open_capture_source(
                &mut f.op,
                &receipt,
                maximum(),
                &f.t,
                &mut provider,
            ))
            .unwrap();
        let mut declaration = declaration(&source);
        declaration.sender = Principal::self_authenticating(scheme.key());
        let plan = RestorePlan::new(&source, declaration).unwrap();
        let r = prepared(&plan, scheme);
        let envelope = r.envelope.clone();
        let observation = r.observation.clone();
        let plan_bytes = plan.encoded().to_vec();
        // The fixture's nanosecond expiry is historical. No live clock is consulted.
        plan.authenticate_status_request(&admission(), &r).unwrap();
        assert_eq!(r.envelope, envelope);
        assert_eq!(r.observation, observation);
        assert_eq!(plan.encoded(), plan_bytes);
        assert_eq!(fs::read(ledger).unwrap(), before);
        assert_eq!(server.count(), 1);
        assert!(
            !f.root
                .join("operations")
                .join(&plan.request().operation_id)
                .exists()
        );
        for field in ["method", "arguments", "destination", "sender"] {
            let mut r = prepared(&plan, scheme);
            change(&mut r, |e| {
                if let EnvelopeContent::Call {
                    method_name,
                    arg,
                    canister_id,
                    sender,
                    ..
                } = e.content.to_mut()
                {
                    match field {
                        "method" => *method_name = "load_canister_snapshot".into(),
                        "arguments" => {
                            *arg = candid_codec::encode_one(
                                &CanisterIdRecord {
                                    canister_id: Principal::anonymous(),
                                },
                                512,
                            )
                            .unwrap()
                        }
                        "destination" => {
                            *canister_id = plan
                                .request()
                                .target
                                .canister
                                .as_slice()
                                .try_into()
                                .unwrap()
                        }
                        _ => *sender = Principal::anonymous().as_slice().try_into().unwrap(),
                    }
                }
                e.sender_sig = Some(scheme.sign(&e.content.to_request_id().signable()));
            });
            assert!(
                matches!(
                    plan.authenticate_status_request(&admission(), &r),
                    Err(StatusRequestError::Admission(_))
                ),
                "{field}"
            );
        }
        for field in ["request_id", "expiry", "signature", "delegation"] {
            let mut r = prepared(&plan, scheme);
            match field {
                "request_id" => r.request_id[0] ^= 1,
                "expiry" => r.expires_ns += 1,
                "signature" => change(&mut r, |e| e.sender_sig.as_mut().unwrap()[0] ^= 1),
                _ => change(&mut r, |e| e.sender_delegation = Some(vec![])),
            }
            assert!(
                plan.authenticate_status_request(&admission(), &r).is_err(),
                "{field}"
            );
        }
    }
}

#[test]
fn restore_request_refuses_excessive_records_and_original_single_call_allowance() {
    let (mut f, _, mut provider) = settled();
    let receipt = complete(&mut f, &mut provider).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut provider),
        )
        .unwrap();
    let mut request = declaration(&source);
    request.sender = Principal::self_authenticating(Scheme::Ed25519.key());
    let plan = RestorePlan::new(&source, request.clone()).unwrap();
    let r = prepared(&plan, Scheme::Ed25519);
    for field in [
        "prepare_deadline",
        "submit_deadline",
        "network",
        "overflow",
        "zero_submit",
        "request",
        "result",
    ] {
        let mut a = admission();
        match field {
            "prepare_deadline" => a.prepare_deadline_ns = 0,
            "submit_deadline" => a.submit_deadline_ns = 0,
            "network" => a.prepare_requests = plan.request().limits.network_requests,
            "overflow" => a.prepare_requests = u32::MAX,
            "zero_submit" => a.submit_requests = 0,
            "request" => a.request_bytes = 1024 * 1024 + 1,
            _ => a.result_bytes = 2 * 1024 * 1024 + 1,
        }
        assert!(
            matches!(
                plan.authenticate_status_request(&a, &r),
                Err(StatusRequestError::Admission(_))
            ),
            "{field}"
        );
    }
    let mut oversized = prepared(&plan, Scheme::Ed25519);
    oversized.observation = vec![0; 4096];
    assert!(matches!(
        plan.authenticate_status_request(&admission(), &oversized),
        Err(StatusRequestError::Admission(_))
    ));
    request.limits.evidence_bytes = 1;
    let tiny = RestorePlan::new(&source, request).unwrap();
    assert!(matches!(
        tiny.authenticate_status_request(&admission(), &r),
        Err(StatusRequestError::Admission(_))
    ));
}
