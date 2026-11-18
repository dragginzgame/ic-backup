use super::*;
use crate::store::capture::advance::tests::fixtures::{http::now, signatures::Scheme};
use ic_platform::{
    certificate::{TimePolicy, TimeWindow},
    ingress::{Envelope, encode_envelope},
};

pub(in crate::store) fn policy() -> CaptureLookupHttpPolicy {
    CaptureLookupHttpPolicy {
        request: CaptureLookupPolicy {
            expires_ns: now() + 30_000_000_000,
            envelope_bytes: 4096,
        },
        observation: CaptureObservationPolicy {
            evidence_bytes: 4096,
            time: TimePolicy {
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
            },
        },
        deadline_ns: now() + 20_000_000_000,
    }
}

pub(in crate::store) fn lookup(
    p: &CapturePolicy,
    r: &PreparedStatus,
    scheme: Scheme,
    expiry: u64,
) -> Vec<u8> {
    let sender = p.request().sender;
    let content = EnvelopeContent::ReadState {
        sender: sender.as_slice().try_into().unwrap(),
        ingress_expiry: expiry,
        paths: vec![vec![b"request_status".to_vec(), r.request_id.to_vec()]],
    };
    let signature = scheme.sign(&content.signable().unwrap());
    encode_envelope(
        &Envelope {
            content,
            sender_pubkey: Some(scheme.key()),
            sender_sig: Some(signature),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap()
}

pub(in crate::store) fn body(cert: &[u8]) -> Vec<u8> {
    let mut bytes = b"\xa1\x6bcertificate\x59".to_vec();
    bytes.extend_from_slice(&u16::try_from(cert.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(cert);
    bytes
}
