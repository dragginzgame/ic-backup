//! Original direct profile, durable signing and cold replay; no restore HTTP dispatch.
use super::fixtures::{admission, declaration, prepared};
use crate::store::{
    CaptureTiming, DirectRestoreStatus, DirectRestoreStatusError, PreparedStatus, RestorePlan,
    RestoreStatusPreparation, RestoreStatusPreparationError, RestoreStatusSigner,
    StatusAdmissionRecord, StatusCertificateError, StatusHttpError, StatusHttpTransport,
    capture::test_fixtures::{
        certificates::der,
        completion::{complete, settled},
        data::Fixture,
        http::{HttpProvider, Response, Server, now, runtime},
        signatures::{Scheme, change},
        source::maximum,
    },
    status_certificate_observation,
};
use ic_platform::{
    Principal,
    certification::historical::{TimePolicy, TimeWindow},
    ic_protocol::EnvelopeContent,
};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Duration,
};

struct Signer {
    scheme: Scheme,
    mode: &'static str,
    protocol: u8,
    ledger: PathBuf,
    calls: Arc<AtomicU32>,
    allow: Arc<AtomicBool>,
    output: Arc<Mutex<Vec<u8>>>,
}
impl RestoreStatusSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [self.protocol; 32]
    }
    fn admit(&self, _: &RestorePlan, _: &StatusAdmissionRecord) -> Result<(), ()> {
        if self.allow.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(())
        }
    }
    async fn sign_status(
        &mut self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
        _: &[u8],
    ) -> Result<PreparedStatus, ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let ledger = fs::read(&self.ledger).unwrap();
        assert!(
            ledger
                .windows(b"status_signing_reserved".len())
                .any(|s| s == b"status_signing_reserved")
        );
        if self.mode == "cancel" {
            return std::future::pending().await;
        }
        let reference = now() - if self.mode == "old" { 1_000_000_000 } else { 0 };
        let future = a.submit_deadline_ns - reference + 10;
        let mut r = prepared(plan, self.scheme);
        r.expires_ns = a.submit_deadline_ns + u64::from(self.mode == "expiry");
        let expiry = r.expires_ns;
        change(&mut r, |e| {
            if let EnvelopeContent::Call { ingress_expiry, .. } = e.content.to_mut() {
                *ingress_expiry = expiry;
            }
            e.sender_sig = Some(self.scheme.sign(&e.content.to_request_id().signable()));
        });
        r.observation = status_certificate_observation(TimePolicy {
            certificate: TimeWindow {
                reference_ns: reference,
                max_age_ns: 100 + u64::from(self.mode == "profile"),
                max_future_skew_ns: future,
            },
            delegation: TimeWindow {
                reference_ns: reference,
                max_age_ns: 200,
                max_future_skew_ns: future,
            },
        })
        .unwrap();
        if self.mode == "opaque" {
            r.observation = vec![1];
        }
        if self.mode == "revoked" {
            self.allow.store(false, Ordering::SeqCst);
        }
        *self.output.lock().unwrap() = r.encode(a.request_bytes).unwrap();
        Ok(r)
    }
}
fn timing() -> CaptureTiming {
    CaptureTiming {
        certificate_max_age_ns: 100,
        delegation_max_age_ns: 200,
        max_clock_skew_ns: 10,
    }
}
fn plan(f: &mut Fixture, p: &mut HttpProvider, scheme: Scheme) -> RestorePlan {
    let receipt = complete(f, p).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, p),
        )
        .unwrap();
    let mut request = declaration(&source);
    request.sender = Principal::self_authenticating(scheme.key());
    RestorePlan::new(&source, request).unwrap()
}
fn signer(f: &Fixture, plan: &RestorePlan, scheme: Scheme, mode: &'static str) -> Signer {
    Signer {
        scheme,
        mode,
        protocol: 17,
        ledger: f
            .root
            .join("operations")
            .join(&plan.request().operation_id)
            .join("ledger.bin"),
        calls: Arc::new(AtomicU32::new(0)),
        allow: Arc::new(AtomicBool::new(true)),
        output: Arc::new(Mutex::new(vec![])),
    }
}
fn live() -> StatusAdmissionRecord {
    StatusAdmissionRecord {
        prepare_deadline_ns: now() + 60_000_000_000,
        submit_deadline_ns: now() + 120_000_000_000,
        prepare_requests: 1,
        ..admission()
    }
}
fn transport(endpoint: &str, plan: &RestorePlan) -> StatusHttpTransport {
    StatusHttpTransport::new(
        endpoint,
        plan.request().target.network,
        der(1),
        Duration::from_secs(5),
    )
    .unwrap()
}

#[test]
fn direct_preparation_retains_exact_both_key_profiles_and_cold_replay_never_reads_live_authority() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (mut f, capture_server, mut p) = settled();
        let plan = plan(&mut f, &mut p, scheme);
        let server = Server::new(Response::Disconnect);
        let s = signer(&f, &plan, scheme, "ok");
        let calls = Arc::clone(&s.calls);
        let allow = Arc::clone(&s.allow);
        let output = Arc::clone(&s.output);
        let ledger = s.ledger.clone();
        let mut direct = DirectRestoreStatus::new(transport(&server.endpoint, &plan), s, timing());
        let mut a = live();
        a.provider = direct.commitment();
        drop(f.op);
        let mut op = runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
            )
            .unwrap();
        assert_eq!(
            runtime()
                .block_on(op.prepare_status(&a, &mut direct))
                .unwrap(),
            RestoreStatusPreparation::Prepared
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let original = fs::read(&ledger).unwrap();
        let view = op.inspect_status().unwrap();
        assert_eq!(view.signing_requests_reserved, 1);
        assert_eq!(view.network_requests_reserved, 1);
        let request = fs::read(ledger.with_file_name("status-request.bin")).unwrap();
        assert_eq!(request, *output.lock().unwrap());
        allow.store(false, Ordering::SeqCst);
        drop(op);
        let mut op = runtime()
            .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
            .unwrap();
        assert_eq!(
            runtime()
                .block_on(op.prepare_status(&a, &mut direct))
                .unwrap(),
            RestoreStatusPreparation::Prepared
        );
        assert_eq!(op.inspect_status().unwrap(), view);
        assert_eq!(fs::read(ledger).unwrap(), original);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(server.count(), 0);
        assert_eq!(capture_server.count(), 1);
    }
}

#[test]
fn changed_direct_configuration_and_invalid_time_or_capacity_refuse_before_signing_reservation() {
    let (mut f, _, mut p) = settled();
    let plan = plan(&mut f, &mut p, Scheme::Ed25519);
    let server = Server::new(Response::Disconnect);
    let original = DirectRestoreStatus::new(
        transport(&server.endpoint, &plan),
        signer(&f, &plan, Scheme::Ed25519, "ok"),
        timing(),
    )
    .commitment();
    let ledger = f
        .root
        .join("operations")
        .join(&plan.request().operation_id)
        .join("ledger.bin");
    let mut signers = Vec::new();
    for field in [
        "route",
        "network",
        "root",
        "timeout",
        "protocol",
        "certificate",
        "delegation",
        "skew",
        "guard",
        "capacity",
        "time",
        "overflow",
    ] {
        let mut s = signer(&f, &plan, Scheme::Ed25519, "ok");
        if field == "protocol" {
            s.protocol += 1;
        }
        if field == "guard" {
            s.allow.store(false, Ordering::SeqCst);
        }
        let calls = Arc::clone(&s.calls);
        let mut t = timing();
        match field {
            "certificate" => t.certificate_max_age_ns += 1,
            "delegation" => t.delegation_max_age_ns += 1,
            "skew" => t.max_clock_skew_ns += 1,
            "overflow" => t.max_clock_skew_ns = u64::MAX,
            _ => {}
        }
        let network = if field == "network" {
            ic_host_artifacts::artifact::Sha256Digest::compute(b"other")
        } else {
            plan.request().target.network
        };
        let transport = StatusHttpTransport::new(
            if field == "route" {
                "http://127.0.0.1:1"
            } else {
                &server.endpoint
            },
            network,
            der(if field == "root" { 2 } else { 1 }),
            Duration::from_secs(if field == "timeout" { 6 } else { 5 }),
        )
        .unwrap();
        let direct = DirectRestoreStatus::new(transport, s, t);
        let mut a = live();
        a.provider = original;
        if field == "capacity" {
            a.request_bytes = 123;
        }
        if field == "time" {
            a.submit_deadline_ns = now() + 600_000_000_000;
        }
        if matches!(field, "overflow" | "network" | "root") {
            a.provider = direct.commitment();
        }
        signers.push((field, direct, a, calls));
    }
    drop(f.op);
    let mut op = runtime()
        .block_on(
            f.store
                .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
        )
        .unwrap();
    let before = fs::read(&ledger).unwrap();
    for (field, mut direct, a, calls) in signers {
        let error = runtime()
            .block_on(op.prepare_status(&a, &mut direct))
            .expect_err(field);
        match field {
            "network" | "root" => assert!(matches!(
                error,
                RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Http(
                    StatusHttpError::Binding
                ))
            )),
            "guard" => assert!(matches!(
                error,
                RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Signer(()))
            )),
            "time" | "overflow" => assert!(matches!(
                error,
                RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Deadline)
            )),
            "capacity" => assert!(matches!(
                error,
                RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Binding)
            )),
            _ => assert!(matches!(error, RestoreStatusPreparationError::Capture(_))),
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0, "{field}");
        assert_eq!(op.inspect_status().unwrap().signing_requests_reserved, 0);
        assert_eq!(fs::read(&ledger).unwrap(), before);
    }
    assert_eq!(server.count(), 0);
}

#[test]
fn changed_signer_profile_or_authority_stays_spent_without_request_replacement_or_http() {
    for mode in ["opaque", "profile", "old", "expiry", "revoked", "cancel"] {
        let (mut f, _, mut p) = settled();
        let plan = plan(&mut f, &mut p, Scheme::Ed25519);
        let server = Server::new(Response::Disconnect);
        let s = signer(&f, &plan, Scheme::Ed25519, mode);
        let ledger = s.ledger.clone();
        let calls = Arc::clone(&s.calls);
        let mut direct = DirectRestoreStatus::new(transport(&server.endpoint, &plan), s, timing());
        let mut a = live();
        a.provider = direct.commitment();
        drop(f.op);
        let mut op = runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
            )
            .unwrap();
        if mode == "cancel" {
            assert!(
                runtime()
                    .block_on(async {
                        tokio::time::timeout(
                            Duration::from_millis(100),
                            op.prepare_status(&a, &mut direct),
                        )
                        .await
                    })
                    .is_err()
            );
        } else {
            let error = runtime()
                .block_on(op.prepare_status(&a, &mut direct))
                .unwrap_err();
            match mode {
                "opaque" => assert!(matches!(
                    error,
                    RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Certificate(
                        StatusCertificateError::Observation
                    ))
                )),
                "profile" => assert!(matches!(
                    error,
                    RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Binding)
                )),
                "revoked" => assert!(matches!(
                    error,
                    RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Signer(()))
                )),
                _ => assert!(matches!(
                    error,
                    RestoreStatusPreparationError::Signer(DirectRestoreStatusError::Deadline)
                )),
            }
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!ledger.with_file_name("status-request.bin").exists());
        let original = fs::read(&ledger).unwrap();
        drop(op);
        let mut op = runtime()
            .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
            .unwrap();
        assert_eq!(
            runtime()
                .block_on(op.prepare_status(&a, &mut direct))
                .unwrap(),
            RestoreStatusPreparation::SigningPending
        );
        assert_eq!(op.inspect_status().unwrap().network_requests_reserved, 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(fs::read(ledger).unwrap(), original);
        assert_eq!(server.count(), 0);
    }
}
