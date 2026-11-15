//! Real read-state qualification with fixture reservations and the durable product workflow.
use super::{der, now};
use ic_backup::store::{
    CaptureCall, CaptureLookupHttpPolicy, CaptureLookupPolicy, CaptureObservation,
    CaptureObservationPolicy, CapturePlan, PreparedStatus, StatusAdmissionRecord,
    StatusHttpTransport, decode_taken_snapshot, observe_capture_certificate,
};
use ic_platform::{
    certification::historical::{TimePolicy, TimeWindow},
    ed25519::{Signer as _, SigningKey},
    ic_protocol::{Envelope, EnvelopeContent, encode_envelope},
    management_canister_types::Snapshot,
};
use std::{borrow::Cow, fs, io::Write, os::unix::fs::OpenOptionsExt, path::Path};

fn retain(root: &Path, name: &str, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(name))
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
    fs::File::open(root).unwrap().sync_all().unwrap();
}

pub(super) async fn qualify(
    transport: &StatusHttpTransport,
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    original: &PreparedStatus,
    root: &Path,
    expected: &Snapshot,
) {
    let reference = now();
    let policy = CaptureLookupHttpPolicy {
        request: CaptureLookupPolicy {
            expires_ns: reference + 60_000_000_000,
            envelope_bytes: 4096,
        },
        deadline_ns: reference + 30_000_000_000,
        observation: CaptureObservationPolicy {
            evidence_bytes: admission.result_bytes,
            time: TimePolicy {
                certificate: TimeWindow {
                    reference_ns: reference,
                    max_age_ns: 60_000_000_000,
                    max_future_skew_ns: 60_000_000_000,
                },
                delegation: TimeWindow {
                    reference_ns: reference,
                    max_age_ns: reference,
                    max_future_skew_ns: 60_000_000_000,
                },
            },
        },
    };
    let journal = root.join("store/operations/00000000000000000000000000000001/ledger.bin");
    let before = fs::read(&journal).unwrap();
    // Caller-owned fixture authority, explicitly separate from the product journal.
    // One additional signature/read fits the original 4-signature/4-network/1-observation plan.
    assert_eq!(plan.request().limits.signing_requests, 4);
    retain(root, "lookup-authority.txt", format!("fixture only; one signing invocation; one read-state; original={:?}; policy={policy:?}\n", original.request_id).as_bytes());
    let content = EnvelopeContent::ReadState {
        ingress_expiry: policy.request.expires_ns,
        sender: ic_platform::Principal::self_authenticating(der())
            .as_slice()
            .try_into()
            .unwrap(),
        paths: vec![vec![
            b"request_status".to_vec().into(),
            original.request_id.to_vec().into(),
        ]],
    };
    let signature = SigningKey::from_bytes(&[17; 32]).sign(&content.to_request_id().signable());
    let bytes = encode_envelope(
        &Envelope {
            content: Cow::Owned(content),
            sender_pubkey: Some(der()),
            sender_sig: Some(signature.to_bytes().to_vec()),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap();
    retain(root, "lookup-request.bin", &bytes);
    retain(
        root,
        "lookup-dispatch-reserved.txt",
        b"exact retained lookup; one HTTP request; no retry\n",
    );
    let evidence = transport
        .lookup_once(
            plan,
            admission,
            original,
            CaptureCall::TakeSnapshot,
            &bytes,
            policy,
        )
        .await
        .unwrap();
    retain(root, "lookup-certificate.bin", &evidence.certificate);
    let CaptureObservation::Replied(reply) = evidence.outcome else {
        panic!("actual take reply")
    };
    assert_eq!(&decode_taken_snapshot(&reply, &[]).unwrap(), expected);
    retain(root, "lookup-reply.bin", &reply);
    assert_eq!(fs::read(&journal).unwrap(), before);
    let root_key = fs::read(root.join("root.der")).unwrap();
    let replay = observe_capture_certificate(
        plan,
        admission,
        original,
        CaptureCall::TakeSnapshot,
        &evidence.certificate,
        &root_key,
        policy.observation,
    )
    .unwrap();
    assert!(matches!(replay, CaptureObservation::Replied(ref actual) if actual == &reply));
}

use ic_backup::store::{
    CaptureLookupAdmissionRecord, CaptureLookupPhase, CaptureLookupProgress, CaptureLookupSigner,
    CaptureOperation, status_certificate_observation,
};

struct LookupSigner {
    calls: usize,
    allowed: bool,
}
impl CaptureLookupSigner for LookupSigner {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [81; 32]
    }
    fn admit_read(&self, _: &CapturePlan, a: &CaptureLookupAdmissionRecord) -> Result<(), ()> {
        assert!(self.allowed, "historical replay cannot call live admission");
        if a.read_authority == [82; 32] {
            Ok(())
        } else {
            Err(())
        }
    }
    async fn sign_lookup(
        &mut self,
        plan: &CapturePlan,
        a: &CaptureLookupAdmissionRecord,
        original: &PreparedStatus,
    ) -> Result<Vec<u8>, ()> {
        assert!(self.allowed);
        self.calls += 1;
        assert_eq!(a.prepare_requests, 0);
        assert!(now() < a.prepare_deadline_ns);
        let ic_backup::store::CaptureExecution::Direct { sender } = plan.request().execution else {
            panic!("direct")
        };
        let content = EnvelopeContent::ReadState {
            ingress_expiry: a.expires_ns,
            sender: sender.as_slice().try_into().unwrap(),
            paths: vec![vec![
                b"request_status".to_vec().into(),
                original.request_id.to_vec().into(),
            ]],
        };
        let signature = SigningKey::from_bytes(&[17; 32]).sign(&content.to_request_id().signable());
        encode_envelope(
            &Envelope {
                content: Cow::Owned(content),
                sender_pubkey: Some(der()),
                sender_sig: Some(signature.to_bytes().to_vec()),
                sender_delegation: None,
            },
            a.request_bytes as usize,
        )
        .map_err(|_| ())
    }
}

pub(super) async fn qualify_durable(
    op: &mut CaptureOperation,
    transport: &StatusHttpTransport,
    original: &StatusAdmissionRecord,
    root: &Path,
    expected: &Snapshot,
) -> CaptureLookupAdmissionRecord {
    let reference = now();
    let time = TimePolicy {
        certificate: TimeWindow {
            reference_ns: reference,
            max_age_ns: 60_000_000_000,
            max_future_skew_ns: 60_000_000_000,
        },
        delegation: TimeWindow {
            reference_ns: reference,
            max_age_ns: reference,
            max_future_skew_ns: 60_000_000_000,
        },
    };
    let admission = CaptureLookupAdmissionRecord {
        call: CaptureCall::TakeSnapshot,
        transport: transport.lookup_commitment(),
        signer: [81; 32],
        read_authority: [82; 32],
        prepare_deadline_ns: reference + 30_000_000_000,
        lookup_deadline_ns: reference + 60_000_000_000,
        expires_ns: reference + 60_000_000_000,
        prepare_requests: 0,
        request_bytes: 4096,
        result_bytes: original.result_bytes,
        observation: status_certificate_observation(time).unwrap(),
    };
    let mut signer = LookupSigner {
        calls: 0,
        allowed: true,
    };
    let before = op.inspect_snapshot().unwrap();
    let journal = root.join("store/operations/00000000000000000000000000000001/ledger.bin");
    let prefix = fs::read(&journal).unwrap();
    let CaptureLookupProgress::Observed(CaptureObservation::Replied(reply)) = op
        .advance_lookup(&admission, transport, &mut signer)
        .await
        .unwrap()
    else {
        panic!("actual retained take lookup")
    };
    assert_eq!(&decode_taken_snapshot(&reply, &[]).unwrap(), expected);
    assert_eq!(signer.calls, 1);
    assert_eq!(op.inspect_snapshot().unwrap(), before);
    let view = op.inspect_lookup().unwrap().unwrap();
    assert_eq!(view.phase, CaptureLookupPhase::Observed);
    assert_eq!(
        (
            view.signing_requests_reserved,
            view.network_requests_reserved,
            view.observations_reserved
        ),
        (1, 1, 1)
    );
    assert!(fs::read(&journal).unwrap().starts_with(&prefix));
    retain(root, "durable-lookup-reply.bin", &reply);
    admission
}

pub(super) async fn replay_durable(
    op: &mut CaptureOperation,
    transport: &StatusHttpTransport,
    admission: &CaptureLookupAdmissionRecord,
    root: &Path,
) {
    let journal = root.join("store/operations/00000000000000000000000000000001/ledger.bin");
    let before = fs::read(&journal).unwrap();
    let mut signer = LookupSigner {
        calls: 0,
        allowed: false,
    };
    let CaptureLookupProgress::Observed(CaptureObservation::Replied(reply)) = op
        .advance_lookup(admission, transport, &mut signer)
        .await
        .unwrap()
    else {
        panic!("historical lookup reply")
    };
    assert_eq!(
        reply,
        fs::read(root.join("durable-lookup-reply.bin")).unwrap()
    );
    assert_eq!(fs::read(&journal).unwrap(), before);
    assert_eq!(signer.calls, 0);
}
