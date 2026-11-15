//! Native journal/real HTTP qualification. Synthetic certificates do not simulate IC effects.
use super::*;
use crate::store::{
    BackupStore, CaptureLookupAdmissionRecord, CaptureLookupAdvanceError, CaptureLookupPhase,
    CaptureLookupProgress, CaptureLookupSigner,
    capture::{
        advance::tests::{
            fixtures::certificates::{der, reply_tree, signed},
            fixtures::{
                durable_lookup::{reopen, reserve_original},
                http::{HttpProvider, Response, Server, http, input, now, runtime},
                signatures::{Scheme, change},
            },
        },
        ledger::{CaptureStatusPhase, EventRecord, EvidenceRecord},
        status::tests::status,
    },
    tests::{root, write_file},
};
use ic_host_artifacts::artifact::Sha256Digest;
use std::{cell::Cell, fs, time::Duration};

#[test]
fn durable_lookup_retains_originals_and_replays_without_signer_or_server() {
    let (p, r, a, _) = input();
    let root = root("durable-lookup");
    let store = BackupStore::initialize(&root).unwrap();
    let cert = signed(
        reply_tree(&r.request_id, b"original reply", b"replied"),
        1,
        None,
    );
    let server = Server::new(Response::Bytes(http(
        200,
        "",
        &crate::store::capture::advance::tests::fixtures::lookup::body(&cert),
    )));
    let t = server.client(&p, Duration::from_secs(1));
    let admission = authority(&t);
    let mut signer = signer(
        crate::store::capture::advance::tests::fixtures::lookup::lookup(
            &p,
            &r,
            Scheme::Ed25519,
            admission.expires_ns,
        ),
    );
    let mut op = reserve_original(
        &store,
        CapturePlan::new(p.request().clone()).unwrap(),
        &r,
        &a,
    );
    let original = op.inspect_status().unwrap();
    let journal = root
        .join("operations")
        .join(&p.request().operation_id)
        .join("ledger.bin");
    let prefix = fs::read(&journal).unwrap();
    assert!(
        matches!(runtime().block_on(op.advance_lookup(&admission,&t,&mut signer)).unwrap(),CaptureLookupProgress::Observed(CaptureObservation::Replied(ref b)) if b == b"original reply")
    );
    assert_eq!(signer.calls, 1);
    assert_eq!(server.finish().len(), 1);
    assert_eq!(op.inspect_status().unwrap(), original);
    let view = op.inspect_lookup().unwrap().unwrap();
    assert_eq!(view.phase, CaptureLookupPhase::Observed);
    assert_eq!(
        (
            view.signing_requests_reserved,
            view.network_requests_reserved,
            view.observations_reserved
        ),
        (1, 2, 1)
    );
    let bytes = fs::read(&journal).unwrap();
    assert!(bytes.starts_with(&prefix));
    drop(op);
    signer.allow = false;
    let admits = signer.admits.get();
    let mut op = reopen(&store, &p);
    assert!(matches!(
        runtime()
            .block_on(op.advance_lookup(&admission, &t, &mut signer))
            .unwrap(),
        CaptureLookupProgress::Observed(_)
    ));
    assert_eq!(signer.admits.get(), admits);
    assert_eq!(signer.calls, 1);
    assert_eq!(fs::read(&journal).unwrap(), bytes);
    assert_eq!(op.inspect_status().unwrap(), original);
    let mut changed = admission.clone();
    changed.expires_ns += 1;
    assert!(matches!(
        runtime().block_on(op.advance_lookup(&changed, &t, &mut signer)),
        Err(CaptureLookupAdvanceError::Binding)
    ));
    assert_eq!(fs::read(&journal).unwrap(), bytes);
}

#[test]
fn signing_failure_cancellation_and_invalid_output_remain_spent_across_reopen() {
    for fault in 0..3 {
        let (p, r, a, _) = input();
        let root = root("lookup-signing-pending");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let admission = authority(&t);
        let mut signer = signer(
            crate::store::capture::advance::tests::fixtures::lookup::lookup(
                &p,
                &r,
                Scheme::Ed25519,
                admission.expires_ns,
            ),
        );
        signer.fail = fault == 0;
        signer.hang = fault == 1;
        if fault == 2 {
            signer.bytes[0] ^= 1;
        }
        let mut op = reserve_original(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &r,
            &a,
        );
        runtime().block_on(async {
            let result = tokio::time::timeout(
                Duration::from_millis(50),
                op.advance_lookup(&admission, &t, &mut signer),
            )
            .await;
            assert!(result.is_err() || result.unwrap().is_err());
        });
        assert_eq!(signer.calls, 1);
        assert_eq!(
            op.inspect_lookup().unwrap().unwrap().phase,
            CaptureLookupPhase::SigningReserved
        );
        drop(op);
        let mut op = reopen(&store, &p);
        signer.allow = false;
        assert!(matches!(
            runtime()
                .block_on(op.advance_lookup(&admission, &t, &mut signer))
                .unwrap(),
            CaptureLookupProgress::SigningPending
        ));
        assert_eq!(signer.calls, 1);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn prepared_lookup_resumes_exact_bytes_but_unknown_http_never_repeats() {
    let (p, r, a, _) = input();
    let root = root("lookup-prepared");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let t = server.client(&p, Duration::from_secs(1));
    let admission = authority(&t);
    let bytes = crate::store::capture::advance::tests::fixtures::lookup::lookup(
        &p,
        &r,
        Scheme::Ed25519,
        admission.expires_ns,
    );
    let mut signer = signer(bytes.clone());
    signer.stop_prepared = true;
    let mut op = reserve_original(
        &store,
        CapturePlan::new(p.request().clone()).unwrap(),
        &r,
        &a,
    );
    assert!(
        runtime()
            .block_on(op.advance_lookup(&admission, &t, &mut signer))
            .is_err()
    );
    assert_eq!(
        op.inspect_lookup().unwrap().unwrap().phase,
        CaptureLookupPhase::Prepared
    );
    assert_eq!(server.count(), 0);
    drop(op);
    let mut op = reopen(&store, &p);
    signer.stop_prepared = false;
    assert!(
        runtime()
            .block_on(op.advance_lookup(&admission, &t, &mut signer))
            .is_err()
    );
    let view = op.inspect_lookup().unwrap().unwrap();
    assert_eq!(view.phase, CaptureLookupPhase::SubmissionReserved);
    assert_eq!(view.network_requests_reserved, 2);
    assert_eq!(signer.calls, 1);
    drop(op);
    let mut op = reopen(&store, &p);
    signer.allow = false;
    assert!(matches!(
        runtime()
            .block_on(op.advance_lookup(&admission, &t, &mut signer))
            .unwrap(),
        CaptureLookupProgress::LookupPending
    ));
    let sent = server.finish();
    assert_eq!(sent.len(), 1);
    assert!(sent[0].ends_with(&bytes));
    assert_eq!(signer.calls, 1);
}

#[test]
fn exhausted_original_budgets_refuse_before_any_signing_or_new_frame() {
    for bound in 0..4 {
        let (p, r, a, _) = input();
        let mut request = p.request().clone();
        request.limits.observations = 1;
        match bound {
            0 => request.limits.observations = 0,
            1 => request.limits.signing_requests = 1,
            2 => request.limits.network_requests = 2,
            3 => request.limits.evidence_bytes = 90_000,
            _ => unreachable!(),
        }
        let p = CapturePlan::new(request).unwrap();
        let root = root("lookup-bounds");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let admission = authority(&t);
        let mut signer = signer(
            crate::store::capture::advance::tests::fixtures::lookup::lookup(
                &p,
                &r,
                Scheme::Ed25519,
                admission.expires_ns,
            ),
        );
        let mut op = reserve_original(&store, p, &r, &a);
        let before = op.check().unwrap();
        assert!(
            runtime()
                .block_on(op.advance_lookup(&admission, &t, &mut signer))
                .is_err()
        );
        assert_eq!(signer.calls, 0);
        assert_eq!(op.check().unwrap(), before);
        assert!(op.inspect_lookup().unwrap().is_none());
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn lookup_evidence_damage_and_partial_journals_are_retained_and_refused() {
    for fault in 0..3 {
        let (p, r, a, _) = input();
        let root = root("lookup-damage");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let admission = authority(&t);
        let mut signer = signer(
            crate::store::capture::advance::tests::fixtures::lookup::lookup(
                &p,
                &r,
                Scheme::Ed25519,
                admission.expires_ns,
            ),
        );
        signer.stop_prepared = true;
        let mut op = reserve_original(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &r,
            &a,
        );
        assert!(
            runtime()
                .block_on(op.advance_lookup(&admission, &t, &mut signer))
                .is_err()
        );
        drop(op);
        let directory = root.join("operations").join(&p.request().operation_id);
        match fault {
            0 => {
                let path = directory.join("lookup-request.bin");
                let mut bytes = fs::read(&path).unwrap();
                bytes[0] ^= 1;
                fs::write(path, bytes).unwrap();
            }
            1 => {
                fs::OpenOptions::new()
                    .append(true)
                    .open(directory.join("ledger.bin"))
                    .unwrap()
                    .write_all(&[1])
                    .unwrap();
            }
            2 => {
                write_file(&directory.join("lookup-certificate.bin"), b"orphan");
            }
            _ => unreachable!(),
        }
        assert!(
            store
                .open_capture(&p.request().operation_id, p.digest())
                .is_err()
        );
        assert_eq!(signer.calls, 1);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn uncertain_lookup_frame_writes_preserve_complete_spending_and_refuse_partial_tails() {
    for stage in 0..4 {
        for partial in [false, true] {
            let (p, r, a, _) = input();
            let root = root("lookup-uncertain-frame");
            let store = BackupStore::initialize(&root).unwrap();
            let server = Server::new(Response::Disconnect);
            let t = server.client(&p, Duration::from_secs(1));
            let admission = authority(&t);
            let request = crate::store::capture::advance::tests::fixtures::lookup::lookup(
                &p,
                &r,
                Scheme::Ed25519,
                admission.expires_ns,
            );
            let cert = signed(
                reply_tree(&r.request_id, b"retained reply", b"replied"),
                1,
                None,
            );
            let mut op = reserve_original(
                &store,
                CapturePlan::new(p.request().clone()).unwrap(),
                &r,
                &a,
            );
            let events = [
                EventRecord::LookupStarted { admission },
                EventRecord::LookupPrepared {
                    request: EvidenceRecord::new(&request),
                },
                EventRecord::LookupSubmissionReserved,
                EventRecord::LookupObserved {
                    certificate: EvidenceRecord::new(&cert),
                },
            ];
            let evidence = [
                None,
                Some(("lookup-request.bin", request.as_slice())),
                None,
                Some(("lookup-certificate.bin", cert.as_slice())),
            ];
            for index in 0..stage {
                op.commit(&events[index], evidence[index]).unwrap();
            }
            assert!(
                op.commit_with(&events[stage], evidence[stage], |f, bytes| {
                    f.write_all(if partial {
                        &bytes[..bytes.len() / 2]
                    } else {
                        bytes
                    })?;
                    f.sync_all()?;
                    Err(std::io::Error::other("injected uncertain persistence"))
                })
                .is_err()
            );
            assert!(matches!(op.inspect_lookup(), Err(CaptureError::Unusable)));
            drop(op);
            let reopened = store.open_capture(&p.request().operation_id, p.digest());
            if partial {
                assert!(reopened.is_err());
            } else {
                let op = reopened.unwrap();
                let view = op.inspect_lookup().unwrap().unwrap();
                assert_eq!(
                    view.phase,
                    [
                        CaptureLookupPhase::SigningReserved,
                        CaptureLookupPhase::Prepared,
                        CaptureLookupPhase::SubmissionReserved,
                        CaptureLookupPhase::Observed
                    ][stage]
                );
                assert_eq!(view.signing_requests_reserved, 1);
                assert_eq!(
                    view.network_requests_reserved,
                    if stage >= 2 { 2 } else { 1 }
                );
            }
            assert_eq!(server.finish().len(), 0);
        }
    }
}

#[test]
fn later_snapshot_admission_cannot_reuse_lookup_allowance() {
    let (p, r, a, _) = input();
    let mut request = p.request().clone();
    request.limits.network_requests = 4;
    request.limits.observations = 1;
    let p = CapturePlan::new(request).unwrap();
    let root = root("lookup-future-budget");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let t = server.client(&p, Duration::from_secs(1));
    let admission = authority(&t);
    let mut op = reserve_original(&store, p, &r, &a);
    op.commit(&EventRecord::LookupStarted { admission }, None)
        .unwrap();
    let result = StatusEvidence {
        rejected: false,
        response: vec![1],
        proof: vec![2],
    }
    .encode(a.result_bytes)
    .unwrap();
    op.commit(
        &EventRecord::Settled {
            sequence: 4,
            result: EvidenceRecord::new(&result),
        },
        Some(("status-result.bin", &result)),
    )
    .unwrap();
    let before = op.check().unwrap();
    let snapshots = SnapshotAdmissionRecord {
        capacity: 2,
        list: StatusAdmissionRecord {
            prepare_requests: 0,
            ..a.clone()
        },
        take: StatusAdmissionRecord {
            prepare_requests: 0,
            ..a
        },
    };
    // Original calls alone fit four requests. Their two extra requests cannot reuse
    // the lookup's already committed two-request allowance.
    assert!(
        op.commit(
            &EventRecord::SnapshotStarted {
                sequence: 5,
                admission: snapshots,
                arguments_sha256: *Sha256Digest::compute(
                    &CaptureCall::ListSnapshots.arguments(&op.plan).unwrap()
                )
                .as_bytes()
            },
            None
        )
        .is_err()
    );
    assert_eq!(op.check().unwrap(), before);
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn lookup_live_binding_permission_and_clock_refuse_before_reservation() {
    for fault in 0..5 {
        let (p, r, a, _) = input();
        let root = root("lookup-live-refusal");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let mut admission = authority(&t);
        match fault {
            0 => admission.transport[0] ^= 1,
            1 => admission.signer[0] ^= 1,
            2 => {}
            3 => admission.observation = vec![0],
            4 => {
                admission.prepare_deadline_ns = now() + 600_000_000_000;
                admission.lookup_deadline_ns = admission.prepare_deadline_ns + 1;
                admission.expires_ns = admission.lookup_deadline_ns;
            }
            _ => unreachable!(),
        }
        let mut signer = signer(
            crate::store::capture::advance::tests::fixtures::lookup::lookup(
                &p,
                &r,
                Scheme::Ed25519,
                admission.expires_ns,
            ),
        );
        signer.allow = fault != 2;
        let mut op = reserve_original(&store, p, &r, &a);
        let before = op.check().unwrap();
        let result = runtime().block_on(op.advance_lookup(&admission, &t, &mut signer));
        match fault {
            0 | 1 => assert!(matches!(result, Err(CaptureLookupAdvanceError::Binding))),
            2 => assert!(matches!(result, Err(CaptureLookupAdvanceError::Signer(())))),
            3 => assert!(matches!(
                result,
                Err(CaptureLookupAdvanceError::Capture(
                    CaptureError::StatusAdmission
                ))
            )),
            4 => assert!(matches!(result, Err(CaptureLookupAdvanceError::Deadline))),
            _ => unreachable!(),
        }
        assert_eq!(op.check().unwrap(), before);
        assert_eq!(signer.calls, 0);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn observed_status_settlement_preserves_policy_spending_and_cold_replay() {
    for valid in [false, true] {
        let (p, mut r, a, _) = input();
        let mut original_time = crate::store::capture::advance::tests::fixtures::lookup::policy()
            .observation
            .time;
        original_time.certificate.reference_ns = 99;
        r.observation = status_certificate_observation(original_time).unwrap();
        let root = root("observed-status-settlement");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let response = if valid {
            candid_codec::encode_one(&status(&p), 4096).unwrap()
        } else {
            b"not candid".to_vec()
        };
        let cert = signed(reply_tree(&r.request_id, &response, b"replied"), 1, None);
        let mut op = reserve_original(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &r,
            &a,
        );
        retain_observation(&mut op, authority(&t), &r, &cert);
        let before = op.inspect_status().unwrap();
        let lookup_before = op.inspect_lookup().unwrap();
        op.settle_capture_lookup(&t).unwrap();
        let mut expected = before;
        expected.phase = CaptureStatusPhase::Settled;
        assert_eq!(op.inspect_status().unwrap(), expected);
        assert_eq!(op.inspect_lookup().unwrap(), lookup_before);
        let path = root
            .join("operations")
            .join(&p.request().operation_id)
            .join("ledger.bin");
        let settled = fs::read(&path).unwrap();
        drop(op);
        let mut op = reopen(&store, &p);
        op.settle_capture_lookup(&t).unwrap();
        assert_eq!(fs::read(&path).unwrap(), settled);
        let mut provider = HttpProvider {
            transport: t,
            prepared: None,
            replay: true,
        };
        let outcome = runtime().block_on(op.advance_status(&a, &mut provider));
        if valid {
            assert_eq!(outcome.unwrap(), StatusProgress::Verified);
        } else {
            assert!(matches!(outcome, Err(StatusAdvanceError::Status(_))));
        }
        assert_eq!(fs::read(&path).unwrap(), settled);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn nonterminal_or_missing_observation_cannot_settle_or_change_spending() {
    for status in [b"received".as_slice(), b"processing", b"done"] {
        let (p, r, a, _) = input();
        let root = root("unresolved-settlement");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let mut op = reserve_original(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &r,
            &a,
        );
        assert!(matches!(
            op.settle_capture_lookup(&t),
            Err(CaptureLookupSettlementError::NotObserved)
        ));
        let cert = signed(reply_tree(&r.request_id, b"", status), 1, None);
        retain_observation(&mut op, authority(&t), &r, &cert);
        let state = op.check().unwrap();
        assert!(matches!(
            op.settle_capture_lookup(&t),
            Err(CaptureLookupSettlementError::Unresolved)
        ));
        assert_eq!(op.check().unwrap(), state);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn lookup_settlement_uncertain_writes_reopen_complete_and_refuse_partial_or_orphan() {
    for fault in 0..3 {
        let (p, r, a, _) = input();
        let root = root("lookup-settlement-write");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let response = candid_codec::encode_one(&status(&p), 4096).unwrap();
        let cert = signed(reply_tree(&r.request_id, &response, b"replied"), 1, None);
        let mut op = reserve_original(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &r,
            &a,
        );
        retain_observation(&mut op, authority(&t), &r, &cert);
        let bytes = StatusEvidence {
            response,
            proof: cert,
            rejected: false,
        }
        .encode(a.result_bytes)
        .unwrap();
        let event = EventRecord::LookupSettled {
            root_key_der: der(1),
            result: EvidenceRecord::new(&bytes),
        };
        assert!(
            op.commit_with(&event, Some(("status-result.bin", &bytes)), |f, frame| {
                if fault < 2 {
                    f.write_all(if fault == 0 {
                        frame
                    } else {
                        &frame[..frame.len() / 2]
                    })?;
                    f.sync_all()?;
                }
                Err(std::io::Error::other("uncertain settlement write"))
            })
            .is_err()
        );
        assert!(matches!(op.inspect_status(), Err(CaptureError::Unusable)));
        drop(op);
        let result = store.open_capture(&p.request().operation_id, p.digest());
        if fault == 0 {
            let mut op = result.unwrap();
            op.settle_capture_lookup(&t).unwrap();
            assert_eq!(
                op.inspect_status().unwrap().phase,
                CaptureStatusPhase::Settled
            );
        } else {
            assert!(result.is_err());
        }
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn lookup_settlement_refuses_conflicting_original_result_and_wrong_root_without_writes() {
    for conflict in [false, true] {
        let (p, r, a, _) = input();
        let root = root("lookup-settlement-conflict");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let response = candid_codec::encode_one(&status(&p), 4096).unwrap();
        let cert = signed(reply_tree(&r.request_id, &response, b"replied"), 1, None);
        let mut op = reserve_original(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &r,
            &a,
        );
        retain_observation(&mut op, authority(&t), &r, &cert);
        let bytes = StatusEvidence {
            response,
            proof: cert,
            rejected: false,
        }
        .encode(a.result_bytes)
        .unwrap();
        let bad_event = EventRecord::LookupSettled {
            root_key_der: der(2),
            result: EvidenceRecord::new(&bytes),
        };
        let before = op.check().unwrap();
        assert!(
            op.commit(&bad_event, Some(("status-result.bin", &bytes)))
                .is_err()
        );
        assert_eq!(op.check().unwrap(), before);
        if conflict {
            op.commit(
                &EventRecord::Settled {
                    sequence: 4,
                    result: EvidenceRecord::new(&bytes),
                },
                Some(("status-result.bin", &bytes)),
            )
            .unwrap();
            let before = op.check().unwrap();
            assert!(matches!(
                op.settle_capture_lookup(&t),
                Err(CaptureLookupSettlementError::Conflict)
            ));
            assert_eq!(op.check().unwrap(), before);
        } else {
            let changed = StatusHttpTransport::new(
                &server.endpoint,
                p.request().target.network,
                der(2),
                Duration::from_secs(1),
            )
            .unwrap();
            assert!(matches!(
                op.settle_capture_lookup(&changed),
                Err(CaptureLookupSettlementError::Conflict)
            ));
            assert_eq!(op.check().unwrap(), before);
        }
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn observed_list_settlement_still_requires_live_authority_before_take() {
    let (p, r, a, _) = input();
    let root = root("observed-list-authority");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let t = server.client(&p, Duration::from_secs(1));
    let mut op = reserve_original(
        &store,
        CapturePlan::new(p.request().clone()).unwrap(),
        &r,
        &a,
    );
    let response = candid_codec::encode_one(&status(&p), 4096).unwrap();
    let proof = signed(reply_tree(&r.request_id, &response, b"replied"), 1, None);
    let bytes = StatusEvidence {
        response,
        proof,
        rejected: false,
    }
    .encode(a.result_bytes)
    .unwrap();
    op.commit(
        &EventRecord::Settled {
            sequence: 4,
            result: EvidenceRecord::new(&bytes),
        },
        Some(("status-result.bin", &bytes)),
    )
    .unwrap();
    let snapshots = SnapshotAdmissionRecord {
        capacity: 2,
        list: a.clone(),
        take: a.clone(),
    };
    op.commit(
        &EventRecord::SnapshotStarted {
            sequence: 5,
            admission: snapshots.clone(),
            arguments_sha256: *Sha256Digest::compute(
                &CaptureCall::ListSnapshots.arguments(&p).unwrap(),
            )
            .as_bytes(),
        },
        None,
    )
    .unwrap();
    let mut list = r;
    change(&mut list, |envelope| {
        if let EnvelopeContent::Call { method_name, .. } = envelope.content.to_mut() {
            *method_name = CaptureCall::ListSnapshots.method_name().into();
        }
        envelope.sender_sig =
            Some(Scheme::Ed25519.sign(&envelope.content.to_request_id().signable()));
    });
    let bytes = list.encode(a.request_bytes).unwrap();
    op.commit(
        &EventRecord::SnapshotPrepared {
            sequence: 6,
            call: CaptureCall::ListSnapshots,
            request: EvidenceRecord::new(&bytes),
        },
        Some(("snapshot-list-request.bin", &bytes)),
    )
    .unwrap();
    op.commit(
        &EventRecord::SnapshotSubmissionReserved {
            sequence: 7,
            call: CaptureCall::ListSnapshots,
        },
        None,
    )
    .unwrap();
    let response = candid_codec::encode_one(
        &Vec::<ic_platform::management_canister_types::Snapshot>::new(),
        4096,
    )
    .unwrap();
    let cert = signed(reply_tree(&list.request_id, &response, b"replied"), 1, None);
    let mut lookup = authority(&t);
    lookup.call = CaptureCall::ListSnapshots;
    retain_observation(&mut op, lookup, &list, &cert);
    op.settle_capture_lookup(&t).unwrap();
    drop(op);
    let mut op = reopen(&store, &p);
    let before = op.check().unwrap();
    let mut provider = HttpProvider {
        transport: t,
        prepared: None,
        replay: true,
    };
    assert!(matches!(
        runtime().block_on(op.advance_snapshot(&snapshots, &mut provider)),
        Err(SnapshotAdvanceError::Provider(StatusHttpError::Binding))
    ));
    assert_eq!(op.check().unwrap(), before);
    assert_eq!(
        op.inspect_snapshot().unwrap().phase,
        CaptureSnapshotPhase::ListSettled
    );
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn certified_lookup_rejection_settles_but_never_becomes_status_success() {
    use ic_platform::certification::{fork, labeled, leaf};
    let (p, r, a, _) = input();
    let root = root("observed-rejection");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let t = server.client(&p, Duration::from_secs(1));
    let tree = fork(
        labeled(
            b"request_status",
            labeled(
                r.request_id.to_vec(),
                fork(
                    labeled(b"reject_code", leaf(vec![5])),
                    fork(
                        labeled(b"reject_message", leaf(b"refused".to_vec())),
                        labeled(b"status", leaf(b"rejected".to_vec())),
                    ),
                ),
            ),
        ),
        labeled(b"time", leaf(vec![100])),
    );
    let cert = signed(tree, 1, None);
    let mut op = reserve_original(
        &store,
        CapturePlan::new(p.request().clone()).unwrap(),
        &r,
        &a,
    );
    retain_observation(&mut op, authority(&t), &r, &cert);
    op.settle_capture_lookup(&t).unwrap();
    drop(op);
    let mut op = reopen(&store, &p);
    let before = op.check().unwrap();
    let mut provider = HttpProvider {
        transport: t,
        prepared: None,
        replay: true,
    };
    assert_eq!(
        runtime()
            .block_on(op.advance_status(&a, &mut provider))
            .unwrap(),
        StatusProgress::Rejected
    );
    assert_eq!(op.check().unwrap(), before);
    assert_eq!(server.finish().len(), 0);
}

struct Signer {
    pub(in crate::store) bytes: Vec<u8>,
    pub(in crate::store) calls: usize,
    pub(in crate::store) admits: Cell<usize>,
    pub(in crate::store) fail: bool,
    pub(in crate::store) hang: bool,
    pub(in crate::store) allow: bool,
    pub(in crate::store) stop_prepared: bool,
}

impl CaptureLookupSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [73; 32]
    }
    fn admit_read(&self, _: &CapturePlan, _: &CaptureLookupAdmissionRecord) -> Result<(), ()> {
        let n = self.admits.get();
        self.admits.set(n + 1);
        if !self.allow || (self.stop_prepared && n == 1) {
            Err(())
        } else {
            Ok(())
        }
    }
    async fn sign_lookup(
        &mut self,
        _: &CapturePlan,
        a: &CaptureLookupAdmissionRecord,
        _: &PreparedStatus,
    ) -> Result<Vec<u8>, ()> {
        self.calls += 1;
        assert_eq!(a.prepare_requests, 1);
        if self.hang {
            pending::<()>().await;
        }
        if self.fail {
            Err(())
        } else {
            Ok(self.bytes.clone())
        }
    }
}

fn signer(bytes: Vec<u8>) -> Signer {
    Signer {
        bytes,
        calls: 0,
        admits: Cell::new(0),
        fail: false,
        hang: false,
        allow: true,
        stop_prepared: false,
    }
}

fn authority(t: &StatusHttpTransport) -> CaptureLookupAdmissionRecord {
    let policy = crate::store::capture::advance::tests::fixtures::lookup::policy();
    CaptureLookupAdmissionRecord {
        call: CaptureCall::Status,
        transport: t.lookup_commitment(),
        signer: [73; 32],
        read_authority: [74; 32],
        prepare_deadline_ns: now() + 10_000_000_000,
        lookup_deadline_ns: policy.deadline_ns,
        expires_ns: policy.request.expires_ns,
        prepare_requests: 1,
        request_bytes: 4096,
        result_bytes: 4096,
        observation: status_certificate_observation(policy.observation.time).unwrap(),
    }
}

fn retain_observation(
    op: &mut CaptureOperation,
    a: CaptureLookupAdmissionRecord,
    r: &PreparedStatus,
    cert: &[u8],
) {
    let bytes = crate::store::capture::advance::tests::fixtures::lookup::lookup(
        &op.plan,
        r,
        Scheme::Ed25519,
        a.expires_ns,
    );
    op.commit(&EventRecord::LookupStarted { admission: a }, None)
        .unwrap();
    op.commit(
        &EventRecord::LookupPrepared {
            request: EvidenceRecord::new(&bytes),
        },
        Some(("lookup-request.bin", &bytes)),
    )
    .unwrap();
    op.commit(&EventRecord::LookupSubmissionReserved, None)
        .unwrap();
    op.commit(
        &EventRecord::LookupObserved {
            certificate: EvidenceRecord::new(cert),
        },
        Some(("lookup-certificate.bin", cert)),
    )
    .unwrap();
}
