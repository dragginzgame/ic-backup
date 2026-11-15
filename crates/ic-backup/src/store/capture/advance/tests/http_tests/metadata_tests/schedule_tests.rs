//! Schedule persistence is local; the enclosing fixture owns authenticated evidence.
use super::*;
use crate::store::CaptureTransferBindError;

pub(super) fn settle(
    op: &mut CaptureOperation,
    s: &Snapshot,
    r: &PreparedStatus,
    e: &StatusEvidence,
    ma: &CaptureMetadataAdmissionRecord,
) {
    let request = r.encode(ma.call.request_bytes).unwrap();
    let result = e.encode(ma.call.result_bytes).unwrap();
    op.commit(
        &EventRecord::MetadataStarted {
            admission: ma.clone(),
            snapshot_id: s.id.clone(),
            timestamp: s.taken_at_timestamp,
            total_size: s.total_size,
        },
        None,
    )
    .unwrap();
    op.commit(
        &EventRecord::MetadataPrepared {
            request: EvidenceRecord::new(&request),
        },
        Some(("metadata-request.bin", &request)),
    )
    .unwrap();
    op.commit(&EventRecord::MetadataSubmissionReserved, None)
        .unwrap();
    op.commit(
        &EventRecord::MetadataSettled {
            result: EvidenceRecord::new(&result),
        },
        Some(("metadata-result.bin", &result)),
    )
    .unwrap();
}

#[test]
fn binding_authenticates_once_and_cold_replay_preserves_bytes_and_spending() {
    let (p, s, r, e, a) = input();
    let root = root("schedule-binding");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let t = server.client(&p, Duration::from_secs(1));
    let ma = authority(&t, &a, &r);
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
        runtime().block_on(op.bind_transfer(&t, &mut provider)),
        Err(CaptureTransferBindError::MetadataRequired)
    ));
    settle(&mut op, &s, &r, &e, &ma);
    let before = op.inspect_metadata().unwrap();
    let view = runtime()
        .block_on(op.bind_transfer(&t, &mut provider))
        .unwrap();
    let metadata = p.decode_metadata(&s.id, &e.response).unwrap();
    let schedule = p.plan_transfer(&s.id, &metadata).unwrap();
    assert_eq!(view.data_calls, schedule.data_calls());
    assert_eq!(view.payload_ceiling, schedule.payload_ceiling());
    assert_eq!(op.inspect_transfer().unwrap(), Some(view.clone()));
    let ledger = root
        .join("operations")
        .join(&p.request().operation_id)
        .join("ledger.bin");
    let bytes = fs::read(&ledger).unwrap();
    drop(op);
    let mut op = reopen(&store, &p);
    assert_eq!(
        runtime()
            .block_on(op.bind_transfer(&t, &mut provider))
            .unwrap(),
        view
    );
    assert_eq!(fs::read(&ledger).unwrap(), bytes);
    assert_eq!(op.inspect_metadata().unwrap(), before);
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn binding_refuses_conflicting_schedule_and_uncertain_frames_preserve_custody() {
    for fault in 0..4 {
        let (p, s, r, e, a) = input();
        let root = root("schedule-fault");
        let store = BackupStore::initialize(&root).unwrap();
        let server = Server::new(Response::Disconnect);
        let t = server.client(&p, Duration::from_secs(1));
        let ma = authority(&t, &a, &r);
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
        settle(&mut op, &s, &r, &e, &ma);
        let metadata = p.decode_metadata(&s.id, &e.response).unwrap();
        let state = op.check().unwrap();
        let m = state.metadata.as_ref().unwrap();
        let mut binding = p
            .plan_transfer(&s.id, &metadata)
            .unwrap()
            .binding(m.request.as_ref().unwrap(), m.result.as_ref().unwrap());
        if fault == 0 {
            binding.commitment[0] ^= 1;
        }
        if fault == 3 {
            binding.version = 2;
        }
        let event = EventRecord::TransferBound { binding };
        if fault == 0 {
            op.commit(&event, None).unwrap();
        } else if fault == 3 {
            assert!(op.commit(&event, None).is_err());
            assert_eq!(op.check().unwrap(), state);
            assert_eq!(server.finish().len(), 0);
            continue;
        } else {
            assert!(
                op.commit_with(&event, None, |f, bytes| {
                    f.write_all(if fault == 2 {
                        &bytes[..bytes.len() / 2]
                    } else {
                        bytes
                    })?;
                    f.sync_all()?;
                    Err(std::io::Error::other("uncertain binding"))
                })
                .is_err()
            );
            assert!(matches!(op.inspect_transfer(), Err(CaptureError::Unusable)));
        }
        drop(op);
        let reopened = store.open_capture(&p.request().operation_id, p.digest());
        if fault == 2 {
            assert!(reopened.is_err());
        } else {
            let mut op = reopened.unwrap();
            let before = op.check().unwrap();
            let result = runtime().block_on(op.bind_transfer(&t, &mut provider));
            if fault == 0 {
                assert!(matches!(result, Err(CaptureTransferBindError::Binding)));
            } else {
                assert!(result.is_ok());
            }
            assert_eq!(op.check().unwrap(), before);
        }
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn schedule_does_not_bypass_original_authentication_or_evidence_budget() {
    let (p, s, r, mut e, a) = input();
    let root = root("schedule-auth");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let t = server.client(&p, Duration::from_secs(1));
    let ma = authority(&t, &a, &r);
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
    e.response[0] ^= 1;
    settle(&mut op, &s, &r, &e, &ma);
    let state = op.check().unwrap();
    assert!(matches!(
        runtime().block_on(op.bind_transfer(&t, &mut provider)),
        Err(CaptureTransferBindError::Metadata(_))
    ));
    assert_eq!(op.check().unwrap(), state);
    // Admit a candidate against a deliberately smaller immutable evidence cap;
    // no new signing or HTTP authority is involved in this local frame.
    let mut request = p.request().clone();
    request.limits.evidence_bytes = state.bytes;
    let small = CapturePlan::new(request).unwrap();
    let event = EventRecord::TransferBound {
        binding: crate::store::capture::transfer::TransferBindingRecord {
            version: 1,
            commitment: [0; 32],
            data_calls: 1,
            payload_ceiling: 1,
        },
    };
    assert!(state.candidate(&event, &small).is_err());
    assert_eq!(server.finish().len(), 0);
}
