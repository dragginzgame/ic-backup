//! Signed read-state policy checks; no signing service, remote lookup or IC effects.
use super::fixtures::signatures::{Scheme, change, fixture};
use super::*;
use ic_platform::ic_protocol::{Envelope, decode_envelope, encode_envelope};

fn policy() -> CaptureLookupPolicy {
    CaptureLookupPolicy {
        expires_ns: 200,
        envelope_bytes: 4096,
    }
}
fn original(scheme: Scheme, call: CaptureCall) -> (CapturePlan, PreparedStatus) {
    let (p, mut r) = fixture(scheme);
    change(&mut r, |envelope| {
        if let EnvelopeContent::Call {
            method_name, arg, ..
        } = envelope.content.to_mut()
        {
            *method_name = call.method_name().into();
            *arg = call.arguments(&p).unwrap();
        }
        envelope.sender_sig = Some(scheme.sign(&envelope.content.to_request_id().signable()));
    });
    (p, r)
}
fn lookup(p: &CapturePlan, r: &PreparedStatus, scheme: Scheme) -> Vec<u8> {
    let CaptureExecution::Direct { sender } = p.request().execution else {
        panic!("direct fixture")
    };
    let content = EnvelopeContent::ReadState {
        ingress_expiry: 200,
        sender: sender.as_slice().try_into().unwrap(),
        paths: vec![vec![
            b"request_status".to_vec().into(),
            r.request_id.to_vec().into(),
        ]],
    };
    let message = [
        b"\x0aic-request".as_slice(),
        content.to_request_id().as_slice(),
    ]
    .concat();
    encode_envelope(
        &Envelope {
            content: Cow::Owned(content),
            sender_pubkey: Some(scheme.key()),
            sender_sig: Some(scheme.sign(&message)),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap()
}
fn modify(bytes: &[u8], scheme: Scheme, action: impl FnOnce(&mut EnvelopeContent)) -> Vec<u8> {
    let mut envelope = decode_envelope(bytes, 4096).unwrap();
    action(envelope.content.to_mut());
    envelope.sender_sig = Some(scheme.sign(&envelope.content.to_request_id().signable()));
    encode_envelope(&envelope, 4096).unwrap()
}
fn authenticate(
    p: &CapturePlan,
    r: &PreparedStatus,
    bytes: &[u8],
) -> Result<[u8; 32], CaptureLookupError> {
    authenticate_capture_lookup(p, &admission(), r, CaptureCall::Status, bytes, policy())
}

#[test]
fn lookup_binds_all_original_calls_and_preserves_both_signed_envelopes() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        for call in [
            CaptureCall::Status,
            CaptureCall::ListSnapshots,
            CaptureCall::TakeSnapshot,
        ] {
            let (p, r) = original(scheme, call);
            let before = r.encode(4096).unwrap();
            let bytes = lookup(&p, &r, scheme);
            let copy = bytes.clone();
            let expected = *decode_envelope(&bytes, 4096)
                .unwrap()
                .content
                .to_request_id();
            assert_ne!(expected, r.request_id);
            let exact = CaptureLookupPolicy {
                envelope_bytes: bytes.len().try_into().unwrap(),
                ..policy()
            };
            for _ in 0..2 {
                assert_eq!(
                    authenticate_capture_lookup(&p, &admission(), &r, call, &bytes, exact).unwrap(),
                    expected
                );
            }
            assert_eq!(bytes, copy);
            assert_eq!(r.encode(4096).unwrap(), before);
            assert_eq!(r.expires_ns, 21); // Lookup expiry cannot replace the old call's expiry.
            let too_small = CaptureLookupPolicy {
                envelope_bytes: exact.envelope_bytes - 1,
                ..exact
            };
            assert!(matches!(
                authenticate_capture_lookup(&p, &admission(), &r, call, &bytes, too_small),
                Err(CaptureLookupError::Bounds)
            ));
        }
    }
}

#[test]
fn valid_lookup_signatures_cannot_expand_or_replace_the_original_status_path() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (p, r) = original(scheme, CaptureCall::Status);
        let bytes = lookup(&p, &r, scheme);
        for case in 0..8 {
            let changed = modify(&bytes, scheme, |content| {
                let EnvelopeContent::ReadState { paths, .. } = content else {
                    panic!("read-state")
                };
                match case {
                    0 => paths.clear(),
                    1 => paths[0].clear(),
                    2 => {
                        paths[0].pop();
                    }
                    3 => paths[0][0] = b"time".to_vec().into(),
                    4 => paths[0][1] = vec![77; 32].into(),
                    5 => paths[0].push(b"reply".to_vec().into()),
                    6 => paths.push(paths[0].clone()),
                    7 => paths.push(vec![b"time".to_vec().into()]),
                    _ => unreachable!(),
                }
            });
            assert!(
                matches!(
                    authenticate(&p, &r, &changed),
                    Err(CaptureLookupError::Binding)
                ),
                "case {case}"
            );
        }
    }
}

#[test]
fn lookup_sender_expiry_kind_and_original_authentication_remain_exact() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (p, mut r) = original(scheme, CaptureCall::Status);
        let bytes = lookup(&p, &r, scheme);
        for case in 0..2 {
            let changed = modify(&bytes, scheme, |content| {
                let EnvelopeContent::ReadState {
                    sender,
                    ingress_expiry,
                    ..
                } = content
                else {
                    panic!("read-state")
                };
                if case == 0 {
                    *sender = [4].as_slice().try_into().unwrap();
                } else {
                    *ingress_expiry += 1;
                }
            });
            assert!(matches!(
                authenticate(&p, &r, &changed),
                Err(CaptureLookupError::Binding)
            ));
        }
        assert!(matches!(
            authenticate(&p, &r, &r.envelope),
            Err(CaptureLookupError::Envelope)
        ));
        assert!(matches!(
            authenticate_capture_lookup(
                &p,
                &admission(),
                &r,
                CaptureCall::TakeSnapshot,
                &bytes,
                policy()
            ),
            Err(CaptureLookupError::Original(StatusRequestError::Admission(
                _
            )))
        ));
        let mut wrong_plan = p.request().clone();
        wrong_plan.target.canister = "rrkah-fqaaa-aaaaa-aaaaq-cai".parse().unwrap();
        if wrong_plan.target.canister == p.request().target.canister {
            wrong_plan.target.canister = "ryjl3-tyaaa-aaaaa-aaaba-cai".parse().unwrap();
        }
        let wrong_plan = CapturePlan::new(wrong_plan).unwrap();
        assert!(matches!(
            authenticate(&wrong_plan, &r, &bytes),
            Err(CaptureLookupError::Original(StatusRequestError::Admission(
                _
            )))
        ));
        r.request_id[0] ^= 1;
        assert!(matches!(
            authenticate(&p, &r, &bytes),
            Err(CaptureLookupError::Original(StatusRequestError::Admission(
                _
            )))
        ));
        r.request_id[0] ^= 1;
        change(&mut r, |e| {
            e.sender_sig.as_mut().unwrap()[0] ^= 1;
        });
        assert!(matches!(
            authenticate(&p, &r, &bytes),
            Err(CaptureLookupError::Original(StatusRequestError::Signature))
        ));
    }
}

#[test]
fn lookup_uses_the_existing_direct_key_signature_and_identity_profile() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (p, r) = original(scheme, CaptureCall::Status);
        let bytes = lookup(&p, &r, scheme);
        for case in 0..7 {
            let mut envelope = decode_envelope(&bytes, 4096).unwrap();
            match case {
                0 => envelope.sender_sig.as_mut().unwrap()[0] ^= 1,
                1 => {
                    envelope.sender_sig =
                        Some(scheme.sign(envelope.content.to_request_id().as_slice()))
                }
                2 => envelope.sender_sig = None,
                3 => envelope.sender_sig.as_mut().unwrap().push(0),
                4 => envelope.sender_pubkey = None,
                5 => envelope.sender_delegation = Some(vec![]),
                6 => *envelope.sender_pubkey.as_mut().unwrap().last_mut().unwrap() ^= 1,
                _ => unreachable!(),
            }
            let changed = encode_envelope(&envelope, 4096).unwrap();
            let failure = authenticate(&p, &r, &changed).unwrap_err();
            match case {
                0..=3 => assert!(matches!(
                    failure,
                    CaptureLookupError::Authentication(StatusRequestError::Signature)
                )),
                4..=5 => assert!(matches!(
                    failure,
                    CaptureLookupError::Authentication(StatusRequestError::Identity)
                )),
                6 => assert!(matches!(
                    failure,
                    CaptureLookupError::Authentication(StatusRequestError::Sender)
                )),
                _ => unreachable!(),
            }
            assert!(!format!("{failure:?}").contains("sender_pubkey"));
        }
    }
}

#[test]
fn lookup_bounds_precede_decode_and_malformed_or_ambiguous_content_refuses() {
    let scheme = Scheme::Ed25519;
    let (p, r) = original(scheme, CaptureCall::Status);
    let bytes = lookup(&p, &r, scheme);
    for invalid in [
        CaptureLookupPolicy {
            expires_ns: 0,
            ..policy()
        },
        CaptureLookupPolicy {
            envelope_bytes: 0,
            ..policy()
        },
        CaptureLookupPolicy {
            envelope_bytes: 1024 * 1024 + 1,
            ..policy()
        },
    ] {
        assert!(matches!(
            authenticate_capture_lookup(
                &p,
                &admission(),
                &r,
                CaptureCall::Status,
                &[0xff],
                invalid
            ),
            Err(CaptureLookupError::Bounds)
        ));
    }
    assert!(matches!(
        authenticate(&p, &r, &[]),
        Err(CaptureLookupError::Bounds)
    ));
    for truncated in [&bytes[..1], &bytes[..bytes.len() - 1], &[0xff], &[0xa0]] {
        assert!(matches!(
            authenticate(&p, &r, truncated),
            Err(CaptureLookupError::Envelope)
        ));
    }
    let mut trailing = bytes.clone();
    trailing.push(0xf6);
    assert!(matches!(
        authenticate(&p, &r, &trailing),
        Err(CaptureLookupError::Envelope)
    ));
    // Insert unknown or duplicate fields into the actual supplier-encoded content map.
    // A permissive decoder would still compute the old request ID for these bytes.
    let marker = b"\x67content\xa4";
    let at = bytes
        .windows(marker.len())
        .position(|w| w == marker)
        .unwrap()
        + marker.len()
        - 1;
    for extra in [b"\x65extra\xf6".as_slice(), b"\x6eingress_expiry\x18\xc8"] {
        let mut changed = bytes.clone();
        changed[at] = 0xa5;
        changed.splice(at + 1..at + 1, extra.iter().copied());
        assert!(matches!(
            authenticate(&p, &r, &changed),
            Err(CaptureLookupError::Envelope)
        ));
    }
    let mut text_label = bytes.clone();
    let at = bytes
        .windows(15)
        .position(|w| w == b"\x4erequest_status")
        .unwrap();
    text_label[at] = 0x6e;
    assert!(matches!(
        authenticate(&p, &r, &text_label),
        Err(CaptureLookupError::Envelope)
    ));
}
