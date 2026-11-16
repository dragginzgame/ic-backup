//! Direct-key request authentication regressions.
use super::{
    fixtures::signatures::{Scheme, change, fixture},
    *,
};
use ic_platform::{Principal, ecdsa};

fn authenticate(plan: &CapturePolicy, prepared: &PreparedStatus) -> Result<(), StatusRequestError> {
    authenticate_status_request(plan, &admission(), prepared)
}

#[test]
fn direct_key_signatures_authenticate_original_bytes_without_current_freshness() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (plan, prepared) = fixture(scheme);
        let before = prepared.encode(4096).unwrap();
        authenticate(&plan, &prepared).unwrap();
        authenticate(&plan, &prepared).unwrap();
        assert_eq!(prepared.encode(4096).unwrap(), before);
        assert_eq!(prepared.expires_ns, 21); // Historical authentication never renews expiry.
    }
}

#[test]
fn invalid_signature_key_domain_and_changed_signed_content_refuse() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        for case in 0..6 {
            let (plan, mut prepared) = fixture(scheme);
            change(&mut prepared, |envelope| match case {
                0 => envelope.sender_sig.as_mut().unwrap()[0] ^= 1,
                1 => {
                    envelope.sender_sig =
                        Some(scheme.sign(envelope.content.to_request_id().as_slice()))
                }
                2 => {
                    if let EnvelopeContent::Call { nonce, .. } = envelope.content.to_mut() {
                        *nonce = Some(vec![44]);
                    }
                }
                3 => {
                    let id = envelope.content.to_request_id();
                    let wrong = [b"\x0dic-state-root".as_slice(), id.as_slice()].concat();
                    envelope.sender_sig = Some(scheme.sign(&wrong));
                }
                4 => envelope.sender_sig = None,
                5 => envelope.sender_sig.as_mut().unwrap().push(0),
                _ => unreachable!(),
            });
            assert!(
                matches!(
                    authenticate(&plan, &prepared),
                    Err(StatusRequestError::Signature)
                ),
                "case {case}"
            );
        }
        let (plan, mut prepared) = fixture(scheme);
        change(&mut prepared, |envelope| {
            *envelope.sender_pubkey.as_mut().unwrap().last_mut().unwrap() ^= 1;
        });
        assert!(matches!(
            authenticate(&plan, &prepared),
            Err(StatusRequestError::Sender)
        ));
    }
}

#[test]
fn valid_signature_cannot_substitute_a_different_original_sender() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (plan, mut prepared) = fixture(scheme);
        let mut request = plan.request().clone();
        let sender = Principal::from_slice(&[9]);
        request.sender = sender;
        let changed = CapturePolicy::new(request).unwrap();
        change(&mut prepared, |envelope| {
            if let EnvelopeContent::Call { sender: caller, .. } = envelope.content.to_mut() {
                *caller = sender.as_slice().try_into().unwrap();
            }
            envelope.sender_sig = Some(scheme.sign(&envelope.content.to_request_id().signable()));
        });
        assert!(matches!(
            authenticate(&changed, &prepared),
            Err(StatusRequestError::Sender)
        ));
    }
}

#[test]
fn unsupported_key_profiles_and_delegations_refuse_without_fallback() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        for case in 0..8 {
            let (plan, mut prepared) = fixture(scheme);
            change(&mut prepared, |envelope| {
                match case {
                0 => envelope.sender_pubkey = None,
                1 => envelope.sender_pubkey.as_mut().unwrap().push(0),
                2 => { envelope.sender_pubkey.as_mut().unwrap().pop(); }
                3 => envelope.sender_pubkey.as_mut().unwrap()[1] ^= 1,
                4 => envelope.sender_delegation = Some(vec![]),
                5 => envelope.sender_delegation = Some(serde_json::from_value(serde_json::json!([
                    {"delegation": {"pubkey": [1], "expiration": 999, "targets": null}, "signature": [2]}
                ])).unwrap()),
                6 => envelope.sender_pubkey = Some(vec![0; 88]),
                7 => {
                    if let EnvelopeContent::Call { sender_info, .. } = envelope.content.to_mut() {
                        *sender_info = Some(serde_json::from_value(serde_json::json!(
                            {"info": [1], "signer": "aaaaa-aa", "sig": [2]}
                        )).unwrap());
                    }
                    envelope.sender_sig = Some(scheme.sign(&envelope.content.to_request_id().signable()));
                }
                _ => unreachable!(),
            }
            });
            assert!(
                matches!(
                    authenticate(&plan, &prepared),
                    Err(StatusRequestError::Identity)
                ),
                "case {case}"
            );
        }
    }
}

#[test]
fn complete_request_limits_and_original_content_are_checked_before_signatures() {
    let (plan, prepared) = fixture(Scheme::Ed25519);
    let mut exact = admission();
    exact.request_bytes = prepared.encode(4096).unwrap().len() as u32;
    authenticate_status_request(&plan, &exact, &prepared).unwrap();
    exact.request_bytes -= 1;
    assert!(matches!(
        authenticate_status_request(&plan, &exact, &prepared),
        Err(StatusRequestError::Admission(_))
    ));
    for case in 0..6 {
        let (plan, mut prepared) = fixture(Scheme::Ed25519);
        match case {
            0 => prepared.request_id[0] ^= 1,
            1 => prepared.expires_ns += 1,
            2 => prepared.observation.clear(),
            3 => prepared.envelope.push(0),
            4 => change(&mut prepared, |e| {
                if let EnvelopeContent::Call { method_name, .. } = e.content.to_mut() {
                    *method_name = "start_canister".into();
                }
            }),
            5 => change(&mut prepared, |e| {
                if let EnvelopeContent::Call { arg, .. } = e.content.to_mut() {
                    arg.push(0);
                }
            }),
            _ => unreachable!(),
        }
        assert!(
            matches!(
                authenticate(&plan, &prepared),
                Err(StatusRequestError::Admission(_))
            ),
            "case {case}"
        );
    }
}

#[test]
fn weak_ed25519_and_high_s_secp256k1_signatures_refuse() {
    let (_, mut prepared) = fixture(Scheme::Ed25519);
    let mut weak_der = Scheme::Ed25519.key();
    weak_der[12..].fill(0);
    weak_der[12] = 1; // Identity point.
    let mut request = plan().request().clone();
    let sender = Principal::self_authenticating(&weak_der);
    request.sender = sender;
    let plan = CapturePolicy::new(request).unwrap();
    change(&mut prepared, |envelope| {
        envelope.sender_pubkey = Some(weak_der);
        if let EnvelopeContent::Call { sender: caller, .. } = envelope.content.to_mut() {
            *caller = sender.as_slice().try_into().unwrap();
        }
        let mut signature = vec![0; 64];
        signature[0] = 1;
        envelope.sender_sig = Some(signature);
    });
    assert!(matches!(
        authenticate(&plan, &prepared),
        Err(StatusRequestError::Signature)
    ));

    let (plan, mut prepared) = fixture(Scheme::Secp256k1);
    change(&mut prepared, |envelope| {
        let signature =
            ecdsa::Signature::from_slice(envelope.sender_sig.as_ref().unwrap()).unwrap();
        let (r, s) = signature.split_scalars();
        let high = ecdsa::Signature::from_scalars(r.to_bytes(), (-s).to_bytes()).unwrap();
        assert_ne!(high, high.normalize_s());
        envelope.sender_sig = Some(high.to_bytes().to_vec());
    });
    assert!(matches!(
        authenticate(&plan, &prepared),
        Err(StatusRequestError::Signature)
    ));
}

#[test]
fn malformed_authentication_field_refuses_after_valid_content_inspection() {
    let (plan, mut prepared) = fixture(Scheme::Ed25519);
    // Corrupt only the external authentication field: valid CBOR integer instead of blob.
    let marker = b"\x6asender_sig\x58\x40";
    let offset = prepared
        .envelope
        .windows(marker.len())
        .position(|bytes| bytes == marker)
        .unwrap();
    let start = offset + marker.len() - 2;
    prepared.envelope.splice(start..start + 66, [0]);
    prepared.validate_call(&plan).unwrap();
    assert!(matches!(
        authenticate(&plan, &prepared),
        Err(StatusRequestError::Envelope)
    ));
}

#[test]
fn malformed_signature_scalars_and_public_points_refuse() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (plan, mut prepared) = fixture(scheme);
        change(&mut prepared, |envelope| {
            envelope.sender_sig.as_mut().unwrap()[32..].fill(255)
        });
        assert!(matches!(
            authenticate(&plan, &prepared),
            Err(StatusRequestError::Signature)
        ));
    }
    let (plan, mut prepared) = fixture(Scheme::Secp256k1);
    let mut request = plan.request().clone();
    let mut bad_der = Scheme::Secp256k1.key();
    bad_der[24..].fill(0); // Correct framing, off-curve uncompressed point (0, 0).
    let sender = Principal::self_authenticating(&bad_der);
    request.sender = sender;
    let plan = CapturePolicy::new(request).unwrap();
    change(&mut prepared, |envelope| {
        envelope.sender_pubkey = Some(bad_der);
        if let EnvelopeContent::Call { sender: caller, .. } = envelope.content.to_mut() {
            *caller = sender.as_slice().try_into().unwrap();
        }
    });
    assert!(matches!(
        authenticate(&plan, &prepared),
        Err(StatusRequestError::Signature)
    ));
}
