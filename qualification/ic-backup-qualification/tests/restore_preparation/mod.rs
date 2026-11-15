//! Signing-only qualification against an authenticated IC capture source.
use super::*;
use ic_backup::store::{PreparedStatus, RestorePlan, RestoreStatusSigner};

struct Signer {
    calls: usize,
    allowed: bool,
}
impl RestoreStatusSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [95; 32]
    }
    fn admit(&self, plan: &RestorePlan, a: &StatusAdmissionRecord) -> Result<(), ()> {
        if self.allowed
            && a.application == [96; 32]
            && a.prepare_requests == 0
            && plan.request().sender == Principal::self_authenticating(der())
        {
            Ok(())
        } else {
            Err(())
        }
    }
    async fn sign_status(
        &mut self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
        arguments: &[u8],
    ) -> Result<PreparedStatus, ()> {
        self.calls += 1;
        let content = EnvelopeContent::Call {
            nonce: None,
            ingress_expiry: a.submit_deadline_ns,
            sender: plan.request().sender.as_slice().try_into().unwrap(),
            canister_id: [].as_slice().try_into().unwrap(),
            method_name: "canister_status".into(),
            arg: arguments.to_vec(),
            sender_info: None,
        };
        let id = content.to_request_id();
        let signature = SigningKey::from_bytes(&[17; 32]).sign(&id.signable());
        let envelope = encode_envelope(
            &Envelope {
                content: Cow::Owned(content),
                sender_pubkey: Some(der()),
                sender_sig: Some(signature.to_bytes().to_vec()),
                sender_delegation: None,
            },
            a.request_bytes as usize,
        )
        .map_err(|_| ())?;
        Ok(PreparedStatus {
            request_id: *id,
            expires_ns: a.submit_deadline_ns,
            envelope,
            // Opaque retained bytes only: this fixture grants no HTTP observation policy.
            observation: vec![1],
        })
    }
}

pub(super) async fn check(
    store: &BackupStore,
    plan: &RestorePlan,
    mut op: ic_backup::store::RestoreOperation<'_>,
    transport: &StatusHttpTransport,
    provider: &mut DirectCaptureProvider<FixtureSigner, FixtureGuard>,
    ledger: &std::path::Path,
) {
    use ic_backup::store::RestoreStatusPreparation::Prepared;
    let now = now();
    let admission = StatusAdmissionRecord {
        provider: [95; 32],
        application: [96; 32],
        prepare_deadline_ns: now + 60_000_000_000,
        submit_deadline_ns: now + 120_000_000_000,
        prepare_requests: 0,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 4096,
    };
    let mut signer = Signer {
        calls: 0,
        allowed: true,
    };
    assert_eq!(
        op.prepare_status(&admission, &mut signer).await.unwrap(),
        Prepared
    );
    let view = op.inspect_status().unwrap();
    assert_eq!(view.signing_requests_reserved, 1);
    assert_eq!(view.network_requests_reserved, 0);
    assert!(view.request_id.is_some());
    let original = fs::read(ledger).unwrap();
    let request = ledger.with_file_name("status-request.bin");
    let bytes = fs::read(&request).unwrap();
    drop(op);
    signer.allowed = false;
    let mut reopened = store
        .open_restore(plan, plan.digest(), transport, provider)
        .await
        .unwrap();
    assert_eq!(
        reopened
            .prepare_status(&admission, &mut signer)
            .await
            .unwrap(),
        Prepared
    );
    assert_eq!(signer.calls, 1);
    assert_eq!(fs::read(ledger).unwrap(), original);
    assert_eq!(fs::read(request).unwrap(), bytes);
}
