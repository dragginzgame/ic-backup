//! Shared real BLS signatures over synthetic certificate trees; no IC effects.

use crate::store::{PreparedStatus, StatusEvidence};
use ic_platform::{
    certificate::{
        PreparedRootKey, TimePolicy, TimeWindow,
        construction::{
            Certificate, Delegation, HashTree, encode_certificate, fork, labeled, leaf,
        },
    },
    fixture_signing::PrivateKey,
};

fn key(value: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = value;

    PrivateKey::deserialize(&bytes).unwrap()
}

pub(in crate::store) fn der(value: u8) -> Vec<u8> {
    PreparedRootKey::from_raw(&key(value).public_key().serialize())
        .unwrap()
        .as_der()
        .to_vec()
}

pub(in crate::store) fn times() -> TimePolicy {
    TimePolicy {
        certificate: TimeWindow {
            reference_ns: 100,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        },
        delegation: TimeWindow {
            reference_ns: 10,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        },
    }
}

pub(in crate::store) fn signed(
    tree: HashTree,
    signer: u8,
    delegation: Option<Delegation>,
) -> Vec<u8> {
    let message = [b"\x0dic-state-root".as_slice(), &tree.digest()].concat();

    encode_certificate(
        &Certificate {
            tree,
            signature: key(signer).sign(&message).serialize().to_vec(),
            delegation,
        },
        2 * 1024 * 1024,
    )
    .unwrap()
}

pub(in crate::store) fn reply_tree(
    request: &[u8; 32],
    response: &[u8],
    outcome: &[u8],
) -> HashTree {
    fork(
        labeled(
            b"request_status",
            labeled(
                request.to_vec(),
                fork(
                    labeled(b"reply", leaf(response.to_vec())),
                    labeled(b"status", leaf(outcome.to_vec())),
                ),
            ),
        ),
        labeled(b"time", leaf(vec![100])),
    )
}

pub(in crate::store) fn delegated(
    canister: &[u8],
    r: &PreparedStatus,
    e: &StatusEvidence,
) -> Vec<u8> {
    assert!(canister.len() < 24);
    // CBOR [[h'canister', h'canister']]: an exact, single-canister certified range.
    let mut range = vec![0x81, 0x82, 0x40 + canister.len() as u8];
    range.extend_from_slice(canister);
    range.push(0x40 + canister.len() as u8);
    range.extend_from_slice(canister);
    let subnet = vec![9, 1];
    let authority = signed(
        fork(
            labeled(
                b"subnet",
                labeled(
                    subnet.clone(),
                    fork(
                        labeled(b"canister_ranges", leaf(range)),
                        labeled(b"public_key", leaf(der(2))),
                    ),
                ),
            ),
            labeled(b"time", leaf(vec![10])),
        ),
        1,
        None,
    );

    signed(
        reply_tree(&r.request_id, &e.response, b"replied"),
        2,
        Some(Delegation {
            subnet_id: subnet,
            certificate: authority,
        }),
    )
}
