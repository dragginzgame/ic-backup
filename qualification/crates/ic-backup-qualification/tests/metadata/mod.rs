//! Product-journal metadata read and historical replay through real IC ingress.
use super::{der, now};
use ic_backup::store::{
    CaptureMetadataAdmissionRecord, CaptureMetadataPhase, CaptureMetadataProgress,
    CaptureMetadataSigner, CaptureOperation, CapturePlan, CaptureProvider, StatusAdmissionRecord,
    StatusHttpTransport, status_certificate_observation,
};
use ic_platform::{
    candid_codec,
    certification::historical::{TimePolicy, TimeWindow},
    ed25519::{Signer as _, SigningKey},
    ic_protocol::{Envelope, EnvelopeContent, encode_envelope},
    management_canister_types::Snapshot,
};
use std::{borrow::Cow, fmt::Debug, fs, path::Path};
struct Signer {
    calls: usize,
    allowed: bool,
}
impl CaptureMetadataSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [91; 32]
    }
    fn admit_read(
        &self,
        _: &CapturePlan,
        a: &CaptureMetadataAdmissionRecord,
        _: &Snapshot,
    ) -> Result<(), ()> {
        assert!(self.allowed, "offline replay must not read live permission");
        assert_eq!(a.read_authority, [92; 32]);
        Ok(())
    }
    async fn sign_metadata(
        &mut self,
        p: &CapturePlan,
        a: &CaptureMetadataAdmissionRecord,
        s: &Snapshot,
    ) -> Result<Vec<u8>, ()> {
        assert!(self.allowed);
        self.calls += 1;
        let ic_backup::store::CaptureExecution::Direct { sender } = p.request().execution else {
            panic!("direct")
        };
        let content = EnvelopeContent::Call {
            nonce: Some(vec![91]),
            ingress_expiry: a.expires_ns,
            sender: sender.as_slice().try_into().unwrap(),
            canister_id: (&[][..]).try_into().unwrap(),
            method_name: "read_canister_snapshot_metadata".into(),
            arg: p.metadata_read_arguments(s).unwrap(),
            sender_info: None,
        };
        let signature = SigningKey::from_bytes(&[17; 32])
            .sign(&content.to_request_id().signable())
            .to_bytes()
            .to_vec();
        Ok(encode_envelope(
            &Envelope {
                content: Cow::Owned(content),
                sender_pubkey: Some(der()),
                sender_sig: Some(signature),
                sender_delegation: None,
            },
            4096,
        )
        .unwrap())
    }
}
pub(super) async fn qualify<P: CaptureProvider>(
    op: &mut CaptureOperation,
    t: &StatusHttpTransport,
    a: &StatusAdmissionRecord,
    p: &mut P,
    s: &Snapshot,
    root: &Path,
) -> CaptureMetadataAdmissionRecord
where
    P::Error: Debug,
{
    let n = now();
    let admission = CaptureMetadataAdmissionRecord {
        call: StatusAdmissionRecord {
            prepare_deadline_ns: n + 30_000_000_000,
            submit_deadline_ns: n + 60_000_000_000,
            prepare_requests: 0,
            ..a.clone()
        },
        transport: t.lookup_commitment(),
        signer: [91; 32],
        read_authority: [92; 32],
        expires_ns: n + 60_000_000_000,
        observation: status_certificate_observation(TimePolicy {
            certificate: TimeWindow {
                reference_ns: n,
                max_age_ns: 60_000_000_000,
                max_future_skew_ns: 60_000_000_000,
            },
            delegation: TimeWindow {
                reference_ns: n,
                max_age_ns: n,
                max_future_skew_ns: 60_000_000_000,
            },
        })
        .unwrap(),
    };
    let mut signer = Signer {
        calls: 0,
        allowed: true,
    };
    let original = op.inspect_snapshot().unwrap();
    let CaptureMetadataProgress::Ready(m) = op
        .advance_metadata(&admission, t, &mut signer, p)
        .await
        .unwrap()
    else {
        panic!("metadata")
    };
    assert_eq!(m.taken_at_timestamp, s.taken_at_timestamp);
    assert_eq!(m.wasm_module_size, 8);
    assert_eq!(m.wasm_memory_size, 0);
    assert_eq!(m.stable_memory_size, 0);
    assert!(m.wasm_chunk_store.is_empty());
    assert!(m.globals.is_empty());
    assert!(m.certified_data.is_empty());
    assert_eq!(signer.calls, 1);
    assert_eq!(op.inspect_snapshot().unwrap(), original);
    let view = op.inspect_metadata().unwrap().unwrap();
    assert_eq!(view.phase, CaptureMetadataPhase::Settled);
    assert_eq!(
        (
            view.signing_requests_reserved,
            view.network_requests_reserved
        ),
        (1, 1)
    );
    let bound = op.bind_transfer(t, p).await.unwrap();
    assert_eq!((bound.data_calls, bound.payload_ceiling), (1, 8));
    assert_eq!(op.inspect_transfer().unwrap(), Some(bound));
    assert_eq!(signer.calls, 1);
    fs::write(
        root.join("metadata-oracle.bin"),
        candid_codec::encode_one(m.as_ref(), 1024 * 1024).unwrap(),
    )
    .unwrap();
    admission
}
pub(super) async fn replay<P: CaptureProvider>(
    op: &mut CaptureOperation,
    t: &StatusHttpTransport,
    a: &CaptureMetadataAdmissionRecord,
    p: &mut P,
    root: &Path,
) where
    P::Error: Debug,
{
    let path = root.join("store/operations/00000000000000000000000000000001/ledger.bin");
    let before = fs::read(&path).unwrap();
    let mut signer = Signer {
        calls: 0,
        allowed: false,
    };
    let CaptureMetadataProgress::Ready(m) =
        op.advance_metadata(a, t, &mut signer, p).await.unwrap()
    else {
        panic!("offline metadata")
    };
    assert_eq!(
        candid_codec::encode_one(m.as_ref(), 1024 * 1024).unwrap(),
        fs::read(root.join("metadata-oracle.bin")).unwrap()
    );
    let retained = op.inspect_transfer().unwrap().unwrap();
    assert_eq!(op.bind_transfer(t, p).await.unwrap(), retained);
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(signer.calls, 0);
}
