//! Durable signing only: synthetic capture evidence, real signatures, no management calls.
use crate::store::{
    PreparedStatus, RestorePlan, RestoreStatusPreparation, RestoreStatusPreparationError,
    RestoreStatusSigner, StatusAdmissionRecord,
    capture::test_fixtures::{
        completion::{complete, settled},
        data::Fixture,
        http::{HttpProvider, runtime},
        signatures::{Scheme, change},
        source::maximum,
    },
    restore::tests::fixtures::{admission, declaration, prepared},
    *,
};
use ic_platform::{Principal, ic_protocol::EnvelopeContent};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct Signer {
    output: Option<PreparedStatus>,
    calls: u32,
    allow: bool,
    mode: &'static str,
    ledger: PathBuf,
}
impl RestoreStatusSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [1; 32]
    }
    fn admit(&self, _: &RestorePlan, _: &StatusAdmissionRecord) -> Result<(), ()> {
        if self.allow { Ok(()) } else { Err(()) }
    }
    async fn sign_status(
        &mut self,
        _: &RestorePlan,
        _: &StatusAdmissionRecord,
        _: &[u8],
    ) -> Result<PreparedStatus, ()> {
        self.calls += 1;
        // The original reservation must already be durable before signer entry.
        let bytes = fs::read(&self.ledger).unwrap();
        assert!(
            bytes
                .windows(b"status_signing_reserved".len())
                .any(|b| b == b"status_signing_reserved")
        );
        match self.mode {
            "fail" => Err(()),
            "hang" => std::future::pending().await,
            _ => Ok(self.output.take().unwrap()),
        }
    }
}
fn setup_plan(
    f: &mut Fixture,
    p: &mut HttpProvider,
) -> (RestorePlan, StatusAdmissionRecord, Signer) {
    let receipt = complete(f, p).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, p),
        )
        .unwrap();
    let mut r = declaration(&source);
    r.sender = Principal::self_authenticating(Scheme::Ed25519.key());
    let plan = RestorePlan::new(&source, r).unwrap();
    let mut a = admission();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    a.prepare_deadline_ns = now + 60_000_000_000;
    a.submit_deadline_ns = now + 120_000_000_000;
    a.prepare_requests = 1;
    let mut signed = prepared(&plan, Scheme::Ed25519);
    signed.expires_ns = a.submit_deadline_ns;
    change(&mut signed, |e| {
        if let EnvelopeContent::Call { ingress_expiry, .. } = e.content.to_mut() {
            *ingress_expiry = a.submit_deadline_ns;
        }
        e.sender_sig = Some(Scheme::Ed25519.sign(&e.content.to_request_id().signable()));
    });
    let ledger = f
        .root
        .join("operations")
        .join(&plan.request().operation_id)
        .join("ledger.bin");
    (
        plan,
        a,
        Signer {
            output: Some(signed),
            calls: 0,
            allow: true,
            mode: "ok",
            ledger,
        },
    )
}
#[test]
fn restore_signing_is_reserved_once_and_prepared_replay_is_effect_free() {
    let (mut f, server, mut p) = settled();
    let (plan, a, mut signer) = setup_plan(&mut f, &mut p);
    let source_ledger = f.ledger();
    let original = fs::read(&source_ledger).unwrap();
    let expected = signer
        .output
        .as_ref()
        .unwrap()
        .encode(a.request_bytes)
        .unwrap();
    drop(f.op);
    let mut op = runtime()
        .block_on(
            f.store
                .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
        )
        .unwrap();
    assert_eq!(op.inspect_status().unwrap().signing_requests_reserved, 0);
    assert_eq!(
        runtime()
            .block_on(op.prepare_status(&a, &mut signer))
            .unwrap(),
        RestoreStatusPreparation::Prepared
    );
    assert_eq!(signer.calls, 1);
    let view = op.inspect_status().unwrap();
    assert_eq!(view.admission, Some(a.clone()));
    assert_eq!(
        (
            view.signing_requests_reserved,
            view.network_requests_reserved
        ),
        (1, 1)
    );
    assert!(view.request_id.is_some());
    let ledger = fs::read(&signer.ledger).unwrap();
    assert_eq!(
        fs::read(signer.ledger.with_file_name("status-request.bin")).unwrap(),
        expected
    );
    drop(op);
    signer.allow = false;
    let mut op = runtime()
        .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
        .unwrap();
    assert_eq!(
        runtime()
            .block_on(op.prepare_status(&a, &mut signer))
            .unwrap(),
        RestoreStatusPreparation::Prepared
    );
    assert_eq!(op.inspect_status().unwrap(), view);
    assert_eq!(signer.calls, 1);
    assert_eq!(fs::read(&signer.ledger).unwrap(), ledger);
    assert_eq!(fs::read(source_ledger).unwrap(), original);
    assert_eq!(server.count(), 1);
}
#[test]
fn failed_cancelled_or_invalid_signer_stays_pending_across_reopen() {
    for mode in ["fail", "hang", "invalid"] {
        let (mut f, _, mut p) = settled();
        let (plan, a, mut signer) = setup_plan(&mut f, &mut p);
        signer.mode = mode;
        if mode == "invalid" {
            change(signer.output.as_mut().unwrap(), |e| {
                e.sender_sig.as_mut().unwrap()[0] ^= 1
            });
        }
        drop(f.op);
        let mut op = runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
            )
            .unwrap();
        if mode == "hang" {
            assert!(
                runtime()
                    .block_on(async {
                        tokio::time::timeout(
                            Duration::from_millis(10),
                            op.prepare_status(&a, &mut signer),
                        )
                        .await
                    })
                    .is_err()
            );
        } else {
            assert!(
                runtime()
                    .block_on(op.prepare_status(&a, &mut signer))
                    .is_err()
            );
        }
        assert_eq!(signer.calls, 1);
        let view = op.inspect_status().unwrap();
        assert_eq!(view.signing_requests_reserved, 1);
        assert!(view.request_id.is_none());
        let ledger = fs::read(&signer.ledger).unwrap();
        drop(op);
        signer.allow = false;
        let mut op = runtime()
            .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
            .unwrap();
        assert_eq!(
            runtime()
                .block_on(op.prepare_status(&a, &mut signer))
                .unwrap(),
            RestoreStatusPreparation::SigningPending
        );
        assert_eq!(signer.calls, 1);
        assert_eq!(fs::read(&signer.ledger).unwrap(), ledger);
        let mut changed = a.clone();
        changed.application = [9; 32];
        assert!(matches!(
            runtime().block_on(op.prepare_status(&changed, &mut signer)),
            Err(RestoreStatusPreparationError::Capture(
                CaptureError::StatusAdmission
            ))
        ));
        assert_eq!(fs::read(&signer.ledger).unwrap(), ledger);
    }
}
#[test]
fn preparation_refuses_live_authority_and_bounds_before_spending() {
    let (mut f, _, mut p) = settled();
    let (plan, a, mut signer) = setup_plan(&mut f, &mut p);
    drop(f.op);
    let mut op = runtime()
        .block_on(
            f.store
                .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
        )
        .unwrap();
    let before = fs::read(&signer.ledger).unwrap();
    for fault in ["provider", "time", "budget", "guard"] {
        let mut changed = a.clone();
        match fault {
            "provider" => changed.provider = [9; 32],
            "time" => changed.prepare_deadline_ns = 1,
            "budget" => changed.prepare_requests = u32::MAX,
            _ => signer.allow = false,
        }
        assert!(
            runtime()
                .block_on(op.prepare_status(&changed, &mut signer))
                .is_err()
        );
        assert_eq!(op.inspect_status().unwrap().signing_requests_reserved, 0);
        assert_eq!(fs::read(&signer.ledger).unwrap(), before);
    }
    assert_eq!(signer.calls, 0);
}
#[test]
fn partial_preparation_and_orphan_request_evidence_refuse_without_repair() {
    for fault in ["partial", "orphan", "changed_request", "suffix"] {
        let (mut f, _, mut p) = settled();
        let (plan, a, mut signer) = setup_plan(&mut f, &mut p);
        drop(f.op);
        let mut op = runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
            )
            .unwrap();
        if fault == "orphan" {
            signer.mode = "fail";
        }
        let _ = runtime().block_on(op.prepare_status(&a, &mut signer));
        drop(op);
        let path = if matches!(fault, "orphan" | "changed_request") {
            signer.ledger.with_file_name("status-request.bin")
        } else {
            signer.ledger.clone()
        };
        if matches!(fault, "orphan" | "changed_request") {
            fs::write(&path, b"retained failure").unwrap();
        } else {
            let mut bytes = fs::read(&path).unwrap();
            if fault == "partial" {
                bytes.pop();
            } else {
                bytes.push(0);
            }
            fs::write(&path, bytes).unwrap();
        }
        let before = fs::read(&path).unwrap();
        assert!(
            runtime()
                .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
                .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), before);
    }
}
