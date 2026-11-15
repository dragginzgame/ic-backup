//! Native persistence and real loopback transport; synthetic certificates are not IC effects.
use super::*;
use crate::store::{
    BackupStore, CaptureMetadataAdvanceError, CaptureMetadataPhase, CaptureMetadataProgress,
    capture::{
        advance::tests::{
            fixtures::certificates::{reply_tree, signed},
            fixtures::{
                durable_lookup::reopen,
                http::{HttpProvider, Response, Server, runtime},
                metadata::{authority, captured, captured_with_lookup, input, response, signer},
            },
        },
        ledger::{EventRecord, EvidenceRecord},
    },
    tests::{root, write_file},
};
use std::{fs, time::Duration};

#[test]
fn exact_metadata_is_retained_and_replayed_without_live_signer_or_http() {
    let (p, s, r, e, a) = input();
    let root = root("metadata-replay");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(response(&e));
    let t = server.client(&p, Duration::from_secs(1));
    let original_t = server.client(&p, Duration::from_secs(1));
    let ma = authority(&t, &a, &r);
    let mut signer = signer(&r);
    let mut provider = HttpProvider {
        transport: original_t,
        prepared: None,
        replay: true,
    };
    let mut op = captured(
        &store,
        CapturePlan::new(p.request().clone()).unwrap(),
        &s,
        &r,
        &a,
    );
    let before = op.inspect_snapshot().unwrap();
    let CaptureMetadataProgress::Ready(m) = runtime()
        .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
        .unwrap()
    else {
        panic!("metadata")
    };
    assert_eq!(
        candid_codec::encode_one(m.as_ref(), 4096).unwrap(),
        e.response
    );
    assert_eq!(signer.calls, 1);
    assert_eq!(server.finish().len(), 1);
    assert_eq!(op.inspect_snapshot().unwrap(), before);
    let view = op.inspect_metadata().unwrap().unwrap();
    assert_eq!(
        (
            view.phase,
            view.signing_requests_reserved,
            view.network_requests_reserved
        ),
        (CaptureMetadataPhase::Settled, 1, 1)
    );
    let ledger = root
        .join("operations")
        .join(&p.request().operation_id)
        .join("ledger.bin");
    let bytes = fs::read(&ledger).unwrap();
    drop(op);
    let mut op = reopen(&store, &p);
    signer.allow = false;
    let admits = signer.admitted.get();
    assert!(matches!(
        runtime()
            .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
            .unwrap(),
        CaptureMetadataProgress::Ready(_)
    ));
    assert_eq!(signer.admitted.get(), admits);
    assert_eq!(signer.calls, 1);
    assert_eq!(fs::read(ledger).unwrap(), bytes);
}

#[test]
fn failed_cancelled_or_invalid_signing_remains_spent_and_unknown_http_never_repeats() {
    for fault in 0..4 {
        let (p, s, r, _, a) = input();
        let root = root("metadata-pending");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let ma = authority(&t, &a, &r);
        let mut signer = signer(&r);
        let mut provider = HttpProvider {
            transport: server.client(&p, Duration::from_secs(1)),
            prepared: None,
            replay: true,
        };
        signer.fail = fault == 0;
        signer.hang = fault == 1;
        if fault == 2 {
            signer.bytes[0] ^= 1;
        }
        let mut op = captured(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &s,
            &r,
            &a,
        );
        if fault == 1 {
            runtime().block_on(async {
                assert!(
                    tokio::time::timeout(
                        Duration::from_millis(30),
                        op.advance_metadata(&ma, &t, &mut signer, &mut provider)
                    )
                    .await
                    .is_err()
                );
            });
        } else {
            assert!(
                runtime()
                    .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
                    .is_err()
            );
        }
        let before = op.inspect_metadata().unwrap().unwrap();
        drop(op);
        let mut op = reopen(&store, &p);
        signer.allow = false;
        let outcome = runtime()
            .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
            .unwrap();
        assert!(matches!(
            (fault, outcome),
            (3, CaptureMetadataProgress::SubmissionPending)
                | (0..=2, CaptureMetadataProgress::SigningPending)
        ));
        assert_eq!(op.inspect_metadata().unwrap().unwrap(), before);
        assert_eq!(signer.calls, 1);
        assert_eq!(server.finish().len(), usize::from(fault == 3));
    }
}

#[test]
fn prepared_metadata_reuses_exact_bytes_after_fresh_read_admission() {
    let (p, s, r, e, a) = input();
    let root = root("metadata-prepared");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(response(&e));
    let t = server.client(&p, Duration::from_secs(1));
    let ma = authority(&t, &a, &r);
    let mut signer = signer(&r);
    signer.stop_prepared = true;
    let mut provider = HttpProvider {
        transport: server.client(&p, Duration::from_secs(1)),
        prepared: None,
        replay: true,
    };
    let mut op = captured(
        &store,
        CapturePlan::new(p.request().clone()).unwrap(),
        &s,
        &r,
        &a,
    );
    assert!(matches!(
        runtime().block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider)),
        Err(CaptureMetadataAdvanceError::Signer(()))
    ));
    assert_eq!(
        op.inspect_metadata().unwrap().unwrap().phase,
        CaptureMetadataPhase::Prepared
    );
    drop(op);
    let mut op = reopen(&store, &p);
    signer.stop_prepared = false;
    signer.fail = true;
    assert!(matches!(
        runtime()
            .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
            .unwrap(),
        CaptureMetadataProgress::Ready(_)
    ));
    assert_eq!(signer.calls, 1);
    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].ends_with(&r.envelope));
}

#[test]
fn metadata_budget_binding_and_live_permission_refuse_before_spending() {
    for fault in 0..5 {
        let (p, s, r, _, a) = input();
        let mut request = p.request().clone();
        if fault == 0 {
            request.limits.signing_requests = 3;
        }
        if fault == 1 {
            request.limits.network_requests = 3;
            request.limits.observations = 1;
        }
        let p = CapturePlan::new(request).unwrap();
        let root = root("metadata-admission");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let mut ma = authority(&t, &a, &r);
        let mut signer = signer(&r);
        if fault == 2 {
            ma.transport[0] ^= 1;
        }
        if fault == 3 {
            ma.signer[0] ^= 1;
        }
        if fault == 4 {
            signer.allow = false;
        }
        let mut provider = HttpProvider {
            transport: server.client(&p, Duration::from_secs(1)),
            prepared: None,
            replay: true,
        };
        let mut op = captured(&store, p, &s, &r, &a);
        let before = op.check().unwrap();
        assert!(
            runtime()
                .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
                .is_err()
        );
        assert_eq!(op.check().unwrap(), before);
        assert_eq!(signer.calls, 0);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn complete_uncertain_metadata_frames_stay_spent_and_partial_frames_refuse() {
    for stage in 0..4 {
        for partial in [false, true] {
            let (p, s, r, e, a) = input();
            let root = root("metadata-frame-write");
            let store = BackupStore::initialize(&root).unwrap();
            let server = Server::new(Response::Disconnect);
            let t = server.client(&p, Duration::from_secs(1));
            let ma = authority(&t, &a, &r);
            let mut op = captured(
                &store,
                CapturePlan::new(p.request().clone()).unwrap(),
                &s,
                &r,
                &a,
            );
            let request = r.encode(a.request_bytes).unwrap();
            let result = e.encode(a.result_bytes).unwrap();
            let events = [
                EventRecord::MetadataStarted {
                    admission: ma.clone(),
                    snapshot_id: s.id.clone(),
                    timestamp: s.taken_at_timestamp,
                    total_size: s.total_size,
                },
                EventRecord::MetadataPrepared {
                    request: EvidenceRecord::new(&request),
                },
                EventRecord::MetadataSubmissionReserved,
                EventRecord::MetadataSettled {
                    result: EvidenceRecord::new(&result),
                },
            ];
            let evidence = [
                None,
                Some(("metadata-request.bin", request.as_slice())),
                None,
                Some(("metadata-result.bin", result.as_slice())),
            ];
            for i in 0..stage {
                op.commit(&events[i], evidence[i]).unwrap();
            }
            assert!(
                op.commit_with(&events[stage], evidence[stage], |f, bytes| {
                    f.write_all(if partial {
                        &bytes[..bytes.len() / 2]
                    } else {
                        bytes
                    })?;
                    f.sync_all()?;
                    Err(std::io::Error::other("uncertain metadata frame"))
                })
                .is_err()
            );
            assert!(matches!(op.inspect_metadata(), Err(CaptureError::Unusable)));
            drop(op);
            let reopened = store.open_capture(&p.request().operation_id, p.digest());
            if partial {
                assert!(reopened.is_err());
            } else {
                let mut op = reopened.unwrap();
                let view = op.inspect_metadata().unwrap().unwrap();
                assert_eq!(
                    view.phase,
                    [
                        CaptureMetadataPhase::SigningReserved,
                        CaptureMetadataPhase::Prepared,
                        CaptureMetadataPhase::SubmissionReserved,
                        CaptureMetadataPhase::Settled
                    ][stage]
                );
                assert_eq!(
                    (
                        view.signing_requests_reserved,
                        view.network_requests_reserved
                    ),
                    (1, u32::from(stage >= 2))
                );
                if stage == 3 {
                    let mut signer = signer(&r);
                    signer.allow = false;
                    let mut provider = HttpProvider {
                        transport: server.client(&p, Duration::from_secs(1)),
                        prepared: None,
                        replay: true,
                    };
                    assert!(matches!(
                        runtime()
                            .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
                            .unwrap(),
                        CaptureMetadataProgress::Ready(_)
                    ));
                }
            }
            assert_eq!(server.finish().len(), 0);
        }
    }
}

#[test]
fn authenticated_bad_metadata_stays_settled_without_repeated_remote_work() {
    let (p, s, r, mut e, a) = input();
    e.response = b"invalid candid".to_vec();
    e.proof = signed(reply_tree(&r.request_id, &e.response, b"replied"), 1, None);
    let root = root("metadata-invalid-policy");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(response(&e));
    let t = server.client(&p, Duration::from_secs(1));
    let ma = authority(&t, &a, &r);
    let mut signer = signer(&r);
    let mut provider = HttpProvider {
        transport: server.client(&p, Duration::from_secs(1)),
        prepared: None,
        replay: true,
    };
    let mut op = captured(
        &store,
        CapturePlan::new(p.request().clone()).unwrap(),
        &s,
        &r,
        &a,
    );
    assert!(matches!(
        runtime().block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider)),
        Err(CaptureMetadataAdvanceError::Metadata(
            MetadataReadError::Metadata(_)
        ))
    ));
    assert_eq!(
        op.inspect_metadata().unwrap().unwrap().phase,
        CaptureMetadataPhase::Settled
    );
    assert_eq!(server.finish().len(), 1);
    drop(op);
    let mut op = reopen(&store, &p);
    let before = op.check().unwrap();
    signer.allow = false;
    assert!(matches!(
        runtime().block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider)),
        Err(CaptureMetadataAdvanceError::Metadata(
            MetadataReadError::Metadata(_)
        ))
    ));
    assert_eq!(op.check().unwrap(), before);
    assert_eq!(signer.calls, 1);
}

#[test]
fn metadata_needs_settled_capture_and_preserves_orphan_or_damaged_evidence() {
    for fault in 0..3 {
        let (p, s, r, _, a) = input();
        let root = root("metadata-custody");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let ma = authority(&t, &a, &r);
        let mut signer = signer(&r);
        let mut provider = HttpProvider {
            transport: server.client(&p, Duration::from_secs(1)),
            prepared: None,
            replay: true,
        };
        let mut op = if fault == 0 {
            store
                .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
                .unwrap()
        } else {
            captured(
                &store,
                CapturePlan::new(p.request().clone()).unwrap(),
                &s,
                &r,
                &a,
            )
        };
        if fault == 0 {
            let before = op.check().unwrap();
            assert!(matches!(
                runtime().block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider)),
                Err(CaptureMetadataAdvanceError::SnapshotRequired)
            ));
            assert_eq!(op.check().unwrap(), before);
        } else {
            signer.stop_prepared = true;
            assert!(
                runtime()
                    .block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider))
                    .is_err()
            );
            drop(op);
            let directory = root.join("operations").join(&p.request().operation_id);
            if fault == 1 {
                let path = directory.join("metadata-request.bin");
                let mut bytes = fs::read(&path).unwrap();
                bytes[0] ^= 1;
                fs::write(path, bytes).unwrap();
            } else {
                write_file(&directory.join("metadata-result.bin"), b"orphan");
            }
            assert!(
                store
                    .open_capture(&p.request().operation_id, p.digest())
                    .is_err()
            );
        }
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn metadata_cannot_reuse_allowance_committed_to_an_original_lookup() {
    let (p, s, r, _, a) = input();
    let mut request = p.request().clone();
    request.limits.signing_requests = 4;
    request.limits.network_requests = 4;
    request.limits.observations = 1;
    let p = CapturePlan::new(request).unwrap();
    let root = root("metadata-lookup-budget");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let t = server.client(&p, Duration::from_secs(1));
    let ma = authority(&t, &a, &r);
    let mut signer = signer(&r);
    let mut provider = HttpProvider {
        transport: server.client(&p, Duration::from_secs(1)),
        prepared: None,
        replay: true,
    };
    let mut op = captured_with_lookup(&store, p, &s, &r, &a, true);
    let before = op.check().unwrap();
    assert!(matches!(
        runtime().block_on(op.advance_metadata(&ma, &t, &mut signer, &mut provider)),
        Err(CaptureMetadataAdvanceError::Capture(CaptureError::Ledger(
            _
        )))
    ));
    assert_eq!(op.check().unwrap(), before);
    assert!(op.inspect_metadata().unwrap().is_none());
    assert_eq!(signer.calls, 0);
    assert_eq!(server.finish().len(), 0);
}
