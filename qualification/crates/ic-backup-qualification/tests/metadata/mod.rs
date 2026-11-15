//! Product-journal metadata read and historical replay through real IC ingress.
use super::{der, now};
use ic_backup::store::{
    CaptureDataAdmissionRecord, CaptureDataPhase, CaptureDataProgress, CaptureDataSigner,
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
        Ok(sign(
            p,
            a.expires_ns,
            "read_canister_snapshot_metadata",
            p.metadata_read_arguments(s).unwrap(),
        ))
    }
}
fn sign(p: &CapturePlan, expires_ns: u64, method: &str, arguments: Vec<u8>) -> Vec<u8> {
    let ic_backup::store::CaptureExecution::Direct { sender } = p.request().execution else {
        panic!("direct")
    };
    let content = EnvelopeContent::Call {
        nonce: Some(vec![91]),
        ingress_expiry: expires_ns,
        sender: sender.as_slice().try_into().unwrap(),
        canister_id: (&[][..]).try_into().unwrap(),
        method_name: method.into(),
        arg: arguments,
        sender_info: None,
    };
    let signature = SigningKey::from_bytes(&[17; 32])
        .sign(&content.to_request_id().signable())
        .to_bytes()
        .to_vec();
    encode_envelope(
        &Envelope {
            content: Cow::Owned(content),
            sender_pubkey: Some(der()),
            sender_sig: Some(signature),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap()
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
    let data = data_admission(&admission);
    let CaptureDataProgress::Ready(bytes) =
        op.advance_data(0, &data, t, &mut signer, p).await.unwrap()
    else {
        panic!("payload")
    };
    assert_eq!(bytes, b"\0asm\x01\0\0\0");
    assert_eq!(
        op.inspect_data(0).unwrap().unwrap().phase,
        CaptureDataPhase::Accepted
    );
    assert_eq!(signer.calls, 2);
    assert_eq!(op.verify_data(t, p).await.unwrap(), 8);
    assert_eq!(op.assemble_data(t, p).await.unwrap(), 8);
    let payload = root.join("store/bundles/00000000000000000000000000000001/payload");
    assert_eq!(fs::read(payload.join("module.wasm")).unwrap(), bytes);
    assert!(fs::read(payload.join("heap.bin")).unwrap().is_empty());
    assert!(fs::read(payload.join("stable.bin")).unwrap().is_empty());
    fs::write(root.join("payload-oracle.bin"), bytes).unwrap();
    let final_admission = StatusAdmissionRecord {
        prepare_deadline_ns: now() + 30_000_000_000,
        submit_deadline_ns: now() + 60_000_000_000,
        ..a.clone()
    };
    assert_eq!(
        op.advance_final_status(&final_admission, t, p)
            .await
            .unwrap(),
        ic_backup::store::StatusProgress::Verified
    );
    let receipt = op.complete_capture(t, p).await.unwrap();
    assert_eq!(receipt.payload_bytes, 8);
    assert_eq!(receipt.captured_at_ns, s.taken_at_timestamp);
    let store = ic_backup::store::BackupStore::open(&root.join("store")).unwrap();
    let limits = ic_backup::store::ScanLimits {
        entries: 10.try_into().unwrap(),
        bytes: (1024 * 1024).try_into().unwrap(),
        results: 10.try_into().unwrap(),
    };
    assert_eq!(
        store
            .latest_completed_backup(&receipt.target, limits)
            .unwrap(),
        Some(receipt.clone())
    );
    assert_eq!(
        store
            .verify_payload(
                &receipt.bundle_id,
                receipt.manifest_sha256,
                8.try_into().unwrap()
            )
            .unwrap(),
        receipt
    );
    let manifest =
        fs::read(root.join("store/bundles/00000000000000000000000000000001/manifest.json"))
            .unwrap();
    assert_eq!(
        ic_host_artifacts::artifact::Sha256Digest::compute(&manifest),
        receipt.manifest_sha256
    );
    fs::write(root.join("completion-oracle.bin"), manifest).unwrap();
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
    let CaptureDataProgress::Ready(bytes) = op
        .advance_data(0, &data_admission(a), t, &mut signer, p)
        .await
        .unwrap()
    else {
        panic!("offline payload")
    };
    assert_eq!(bytes, fs::read(root.join("payload-oracle.bin")).unwrap());
    assert_eq!(op.verify_data(t, p).await.unwrap(), 8);
    assert_eq!(op.assemble_data(t, p).await.unwrap(), 8);
    let final_admission = op.inspect_final_status().unwrap().admission.unwrap();
    assert_eq!(
        op.advance_final_status(&final_admission, t, p)
            .await
            .unwrap(),
        ic_backup::store::StatusProgress::Verified
    );
    let receipt = op.complete_capture(t, p).await.unwrap();
    assert_eq!(
        op.verify_completion(receipt.manifest_sha256, t, p)
            .await
            .unwrap(),
        receipt
    );
    let store = ic_backup::store::BackupStore::open(&root.join("store")).unwrap();
    let snapshot_id = receipt
        .snapshot_id_hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|hex| u8::from_str_radix(std::str::from_utf8(hex).unwrap(), 16).unwrap())
        .collect();
    let mut source = store
        .open_capture_source(op, &receipt, 8.try_into().unwrap(), t, p)
        .await
        .unwrap();
    let source_request = ic_platform::management_canister_types::ReadCanisterSnapshotDataArgs {
        canister_id: receipt.target.canister,
        snapshot_id,
        kind: ic_platform::management_canister_types::SnapshotDataKind::WasmModule {
            offset: 0,
            size: 8,
        },
    };
    assert_eq!(
        source.read_data(&source_request).unwrap(),
        fs::read(root.join("payload-oracle.bin")).unwrap()
    );
    assert_eq!(source.summary(), &receipt);
    drop(source);
    let original = fs::read(root.join("completion-oracle.bin")).unwrap();
    assert_eq!(
        fs::read(root.join("store/bundles/00000000000000000000000000000001/manifest.json"))
            .unwrap(),
        original
    );
    assert_eq!(
        ic_host_artifacts::artifact::Sha256Digest::compute(&original),
        receipt.manifest_sha256
    );
    assert_eq!(fs::read(&path).unwrap(), before);

    assert_eq!(signer.calls, 0);
}

fn data_admission(a: &CaptureMetadataAdmissionRecord) -> CaptureDataAdmissionRecord {
    CaptureDataAdmissionRecord {
        call: StatusAdmissionRecord {
            result_bytes: 2 * 1024 * 1024,
            ..a.call.clone()
        },
        transport: a.transport,
        signer: [93; 32],
        read_authority: [94; 32],
        expires_ns: a.expires_ns,
        observation: a.observation.clone(),
    }
}
impl CaptureDataSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [93; 32]
    }
    fn admit_read(
        &self,
        _: &CapturePlan,
        a: &CaptureDataAdmissionRecord,
        n: u32,
        _: &Snapshot,
    ) -> Result<(), ()> {
        assert!(
            self.allowed,
            "offline payload replay must not admit live work"
        );
        assert_eq!(a.read_authority, [94; 32]);
        assert_eq!(n, 0);
        Ok(())
    }
    async fn sign_data(
        &mut self,
        p: &CapturePlan,
        a: &CaptureDataAdmissionRecord,
        _: u32,
        args: &[u8],
    ) -> Result<Vec<u8>, ()> {
        assert!(self.allowed);
        self.calls += 1;
        Ok(sign(
            p,
            a.expires_ns,
            "read_canister_snapshot_data",
            args.to_vec(),
        ))
    }
}
