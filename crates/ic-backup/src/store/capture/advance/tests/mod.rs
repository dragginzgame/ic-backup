//! Exact protocol and HTTP regressions. Call lifecycle tests live with the recorder.
use super::*;
use crate::store::BackupDigest;
use crate::store::capture::status::tests::{plan, status};
pub(in crate::store) use crate::store::capture::test_fixtures as fixtures;
use ic_platform::{candid_codec, ic_protocol::EnvelopeContent};
use std::borrow::Cow;
pub(in crate::store) fn admission() -> StatusAdmissionRecord {
    StatusAdmissionRecord {
        provider: [1; 32],
        application: [2; 32],
        prepare_deadline_ns: 10,
        submit_deadline_ns: 20,
        prepare_requests: 1,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 4096,
    }
}
pub(in crate::store) fn prepared() -> PreparedStatus {
    prepared_content(call_content())
}
pub(in crate::store) fn call_content() -> EnvelopeContent {
    let p = plan();
    let sender = p.request().sender;
    EnvelopeContent::Call {
        nonce: Some(vec![0, 255, 7]),
        ingress_expiry: 21,
        sender: sender.as_slice().try_into().unwrap(),
        canister_id: [].as_slice().try_into().unwrap(),
        method_name: "canister_status".into(),
        arg: arguments(&p).unwrap(),
        sender_info: None,
    }
}
pub(in crate::store) fn prepared_content(content: EnvelopeContent) -> PreparedStatus {
    let request_id = *content.to_request_id();
    let expires_ns = content.ingress_expiry();
    // Correct wire shape, deliberately unauthenticated. No IC effect is simulated.
    let envelope = ic_platform::ic_protocol::encode_envelope(
        &ic_platform::ic_protocol::Envelope {
            content: Cow::Owned(content),
            sender_pubkey: Some(vec![1]),
            sender_sig: Some(vec![2]),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap();
    PreparedStatus {
        request_id,
        expires_ns,
        envelope,
        observation: b"original observation binding".to_vec(),
    }
}
pub(in crate::store) fn evidence() -> StatusEvidence {
    StatusEvidence {
        rejected: false,
        response: candid_codec::encode_one(&status(&plan()), 4096).unwrap(),
        proof: b"native ordering fixture; not a certificate".to_vec(),
    }
}
pub(in crate::store) mod certificate_tests;
mod http_lookup_tests;
mod http_tests;
mod lookup_tests;
mod observation_tests;
mod request_tests;
