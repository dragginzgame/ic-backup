//! Native transport/persistence tests use explicit synthetic certificates, not IC emulation.
use super::*;
use crate::store::{
    CaptureDataAdmissionRecord, CaptureDataAdvanceError, CaptureDataPhase, CaptureDataProgress,
    CaptureDataSigner, DataReadError, TransferDataError,
};
use ic_platform::management_canister_types::ReadCanisterSnapshotDataResult;

struct DataSigner {
    base: PreparedStatus,
    calls: usize,
    admits: Cell<usize>,
    allow: bool,
    fail: bool,
    hang: bool,
    stop_prepared: bool,
}
impl CaptureDataSigner for DataSigner {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [93; 32]
    }
    fn admit_read(
        &self,
        _: &CapturePlan,
        _: &CaptureDataAdmissionRecord,
        _: u32,
        _: &Snapshot,
    ) -> Result<(), ()> {
        let n = self.admits.get();
        self.admits.set(n + 1);
        if self.allow && (!self.stop_prepared || n == 0) {
            Ok(())
        } else {
            Err(())
        }
    }
    async fn sign_data(
        &mut self,
        _: &CapturePlan,
        _: &CaptureDataAdmissionRecord,
        _: u32,
        args: &[u8],
    ) -> Result<Vec<u8>, ()> {
        self.calls += 1;
        if self.hang {
            pending::<()>().await;
        }
        if self.fail {
            return Err(());
        }
        Ok(prepared_data(&self.base, args).envelope)
    }
}
fn prepared_data(base: &PreparedStatus, args: &[u8]) -> PreparedStatus {
    let mut r = PreparedStatus::decode(&base.encode(4096).unwrap()).unwrap();
    change(&mut r, |e| {
        if let EnvelopeContent::Call {
            method_name, arg, ..
        } = e.content.to_mut()
        {
            *method_name = "read_canister_snapshot_data".into();
            *arg = args.to_vec();
        }
        e.sender_sig = Some(Scheme::Ed25519.sign(&e.content.to_request_id().signable()));
    });
    r
}
fn data_evidence(r: &PreparedStatus, payload: &[u8]) -> StatusEvidence {
    let response = candid_codec::encode_one(
        &ReadCanisterSnapshotDataResult {
            chunk: payload.to_vec(),
        },
        4096,
    )
    .unwrap();
    StatusEvidence {
        proof: signed(reply_tree(&r.request_id, &response, b"replied"), 1, None),
        response,
        rejected: false,
    }
}
struct Fixture {
    store: BackupStore,
    p: CapturePlan,
    op: CaptureOperation,
    t: StatusHttpTransport,
    provider: HttpProvider,
    signer: DataSigner,
    a: CaptureDataAdmissionRecord,
    requests: Vec<PreparedStatus>,
    results: Vec<StatusEvidence>,
    server: Server,
    root: PathBuf,
}
impl Fixture {
    fn new(disconnect: bool, bad_payload: bool) -> Self {
        Self::with_limits(disconnect, bad_payload, 20, 16 * 1024 * 1024)
    }
    fn with_limits(disconnect: bool, bad_payload: bool, network: u32, evidence: u64) -> Self {
        Self::with_hash(
            disconnect,
            bad_payload,
            network,
            evidence,
            ic_host_artifacts::artifact::Sha256Digest::compute(b"helloworld"),
        )
    }
    fn with_hash(
        disconnect: bool,
        bad_payload: bool,
        network: u32,
        evidence: u64,
        module_hash: ic_host_artifacts::artifact::Sha256Digest,
    ) -> Self {
        Self::with_parts(
            disconnect,
            bad_payload,
            network,
            evidence,
            module_hash,
            false,
        )
    }
    fn with_parts(
        disconnect: bool,
        bad_payload: bool,
        network: u32,
        evidence: u64,
        module_hash: ic_host_artifacts::artifact::Sha256Digest,
        mixed: bool,
    ) -> Self {
        let (p, s, r, mut e, a) = input();

        let mut fields = p.request().clone();
        fields.module_sha256 = module_hash;
        fields.limits.extent_bytes = 5;
        fields.limits.evidence_bytes = evidence;
        fields.limits.network_requests = network;
        fields.limits.signing_requests = 20;
        let p = CapturePlan::new(fields).unwrap();
        let mut m = p.decode_metadata(&s.id, &e.response).unwrap();
        if mixed {
            use ic_host_artifacts::artifact::Sha256Digest;
            use ic_platform::management_canister_types::ChunkHash;
            m.wasm_memory_size = 6;
            m.stable_memory_size = 3;
            m.wasm_chunk_store = [b"".as_slice(), b"chunk"]
                .into_iter()
                .map(|b| ChunkHash {
                    hash: Sha256Digest::compute(b).as_bytes().to_vec(),
                })
                .collect();
            e.response = candid_codec::encode_one(&m, 4096).unwrap();
            e.proof = signed(reply_tree(&r.request_id, &e.response, b"replied"), 1, None);
        }
        let transfer = p.plan_transfer(&s.id, &m).unwrap();
        assert_eq!(transfer.data_calls(), if mixed { 7 } else { 2 });
        let requests = (0..transfer.data_calls())
            .map(|n| prepared_data(&r, &transfer.data_arguments(n).unwrap()))
            .collect::<Vec<_>>();
        let results = requests
            .iter()
            .enumerate()
            .map(|(n, r)| {
                data_evidence(
                    r,
                    if bad_payload {
                        b"bad"
                    } else {
                        use ic_platform::management_canister_types::SnapshotDataKind;
                        match transfer.requests().nth(n).unwrap().kind {
                            SnapshotDataKind::WasmModule { offset: 0, .. } => b"hello",
                            SnapshotDataKind::WasmModule { .. } => b"world",
                            SnapshotDataKind::WasmMemory { offset: 0, .. } => b"abcde",
                            SnapshotDataKind::WasmMemory { .. } => b"f",
                            SnapshotDataKind::StableMemory { .. } => b"xyz",
                            SnapshotDataKind::WasmChunk { hash } => {
                                if hash
                                    == ic_host_artifacts::artifact::Sha256Digest::compute(b"")
                                        .as_bytes()
                                {
                                    b""
                                } else {
                                    b"chunk"
                                }
                            }
                        }
                    },
                )
            })
            .collect::<Vec<_>>();
        let replies = results
            .iter()
            .map(|e| {
                let Response::Bytes(b) = response(e) else {
                    unreachable!()
                };
                b
            })
            .collect();
        let server = Server::new(if disconnect {
            Response::Disconnect
        } else {
            Response::Sequence(replies)
        });
        let t = server.client(&p, Duration::from_secs(1));
        let ma = authority(&t, &a, &r);
        let mut provider = HttpProvider {
            transport: server.client(&p, Duration::from_secs(1)),
            prepared: None,
            replay: true,
        };
        let root = root("durable-data");
        let store = BackupStore::initialize(&root).unwrap();
        let mut op = captured(
            &store,
            CapturePlan::new(p.request().clone()).unwrap(),
            &s,
            &r,
            &a,
        );
        super::schedule_tests::settle(&mut op, &s, &r, &e, &ma);
        runtime()
            .block_on(op.bind_transfer(&t, &mut provider))
            .unwrap();
        let a = CaptureDataAdmissionRecord {
            call: StatusAdmissionRecord {
                result_bytes: 2 * 1024 * 1024,
                ..a
            },
            transport: t.lookup_commitment(),
            signer: [93; 32],
            read_authority: [94; 32],
            expires_ns: r.expires_ns,
            observation: r.observation.clone(),
        };
        let signer = DataSigner {
            base: r,
            calls: 0,
            admits: Cell::new(0),
            allow: true,
            fail: false,
            hang: false,
            stop_prepared: false,
        };
        Self {
            store,
            p,
            op,
            t,
            provider,
            signer,
            a,
            requests,
            results,
            server,
            root,
        }
    }
    fn advance(
        &mut self,
        n: u32,
    ) -> Result<CaptureDataProgress, CaptureDataAdvanceError<StatusHttpError, ()>> {
        runtime().block_on(self.op.advance_data(
            n,
            &self.a,
            &self.t,
            &mut self.signer,
            &mut self.provider,
        ))
    }
    fn ledger(&self) -> PathBuf {
        self.root
            .join("operations")
            .join(&self.p.request().operation_id)
            .join("ledger.bin")
    }
    fn reopen(self) -> Self {
        let Self {
            op,
            store,
            p,
            t,
            provider,
            signer,
            a,
            requests,
            results,
            server,
            root,
        } = self;
        drop(op);
        let op = reopen(&store, &p);
        Self {
            op,
            store,
            p,
            t,
            provider,
            signer,
            a,
            requests,
            results,
            server,
            root,
        }
    }
}
#[test]
fn sequential_data_retains_exact_bytes_and_replays_without_signer_or_http() {
    let mut f = Fixture::new(false, false);
    let metadata = f.op.inspect_metadata().unwrap();
    assert!(matches!(
        f.advance(1),
        Err(CaptureDataAdvanceError::Binding)
    ));
    for (n, expected) in [(0, b"hello"), (1, b"world")] {
        let CaptureDataProgress::Ready(bytes) = f.advance(n).unwrap() else {
            panic!("data")
        };
        assert_eq!(bytes, expected);
        assert_eq!(
            f.op.inspect_data(n).unwrap().unwrap().phase,
            CaptureDataPhase::Accepted
        );
    }
    assert_eq!(f.signer.calls, 2);
    assert_eq!(f.server.count(), 2);
    let journal = fs::read(f.ledger()).unwrap();
    f = f.reopen();
    f.signer.allow = false;
    let admits = f.signer.admits.get();
    for (n, expected) in [(0, b"hello"), (1, b"world")] {
        let CaptureDataProgress::Ready(bytes) = f.advance(n).unwrap() else {
            panic!("replay")
        };
        assert_eq!(bytes, expected);
    }
    assert_eq!(f.signer.admits.get(), admits);
    assert_eq!(f.signer.calls, 2);
    assert_eq!(f.server.count(), 2);
    assert_eq!(fs::read(f.ledger()).unwrap(), journal);
    assert_eq!(f.op.inspect_metadata().unwrap(), metadata);
}
#[test]
fn signing_failures_cancellation_and_lost_http_remain_spent() {
    for fault in 0..4 {
        let mut f = Fixture::new(true, false);
        if fault == 0 {
            f.signer.fail = true;
        }
        if fault == 1 {
            f.signer.hang = true;
        }
        if fault == 2 {
            change(&mut f.signer.base, |e| {
                if let EnvelopeContent::Call { sender, .. } = e.content.to_mut() {
                    *sender = ic_platform::Principal::anonymous()
                        .as_slice()
                        .try_into()
                        .unwrap();
                }
            });
        }
        if fault == 1 {
            assert!(
                runtime()
                    .block_on(async {
                        tokio::time::timeout(
                            Duration::from_millis(30),
                            f.op.advance_data(0, &f.a, &f.t, &mut f.signer, &mut f.provider),
                        )
                        .await
                    })
                    .is_err()
            );
        } else {
            assert!(f.advance(0).is_err());
        }
        f = f.reopen();
        let calls = f.signer.calls;
        f.signer.allow = false;
        if fault == 3 {
            assert!(matches!(
                f.advance(0).unwrap(),
                CaptureDataProgress::SubmissionPending
            ));
        } else {
            assert!(matches!(
                f.advance(0).unwrap(),
                CaptureDataProgress::SigningPending
            ));
        }
        assert_eq!(f.signer.calls, calls);
        assert_eq!(f.server.count(), usize::from(fault == 3));
        assert!(matches!(
            f.advance(1),
            Err(CaptureDataAdvanceError::Binding)
        ));
    }
}
#[test]
fn prepared_continuation_reuses_exact_request_and_invalid_payload_stays_settled() {
    let mut f = Fixture::new(false, false);
    f.signer.stop_prepared = true;
    assert!(matches!(
        f.advance(0),
        Err(CaptureDataAdvanceError::Signer(()))
    ));
    assert_eq!(
        f.op.inspect_data(0).unwrap().unwrap().phase,
        CaptureDataPhase::Prepared
    );
    f = f.reopen();
    f.signer.stop_prepared = false;
    assert!(matches!(
        f.advance(0).unwrap(),
        CaptureDataProgress::Ready(_)
    ));
    assert_eq!(f.signer.calls, 1);
    let mut f = Fixture::new(false, true);
    assert!(matches!(
        f.advance(0),
        Err(CaptureDataAdvanceError::Data(DataReadError::Data(
            TransferDataError::Length { .. }
        )))
    ));
    assert_eq!(
        f.op.inspect_data(0).unwrap().unwrap().phase,
        CaptureDataPhase::Settled
    );
    let journal = fs::read(f.ledger()).unwrap();
    f = f.reopen();
    f.signer.allow = false;
    assert!(f.advance(0).is_err());
    assert_eq!(fs::read(f.ledger()).unwrap(), journal);
    assert_eq!(f.server.count(), 1);
    assert_eq!(f.signer.calls, 1);
    assert!(matches!(
        f.advance(1),
        Err(CaptureDataAdvanceError::Binding)
    ));
}
#[test]
fn data_binding_permission_and_complete_proof_bounds_refuse_before_signing() {
    for fault in 0..5 {
        let mut f = Fixture::new(true, false);
        match fault {
            0 => f.a.call.result_bytes = 4096,
            1 => f.a.transport[0] ^= 1,
            2 => f.a.signer[0] ^= 1,
            3 => f.signer.allow = false,
            4 => f.a.call.application[0] ^= 1,
            _ => unreachable!(),
        }
        let state = f.op.check().unwrap();
        assert!(f.advance(0).is_err());
        assert_eq!(f.op.check().unwrap(), state);
        assert_eq!(f.signer.calls, 0);
        assert_eq!(f.server.count(), 0);
    }
}
#[test]
fn every_uncertain_data_frame_retains_spending_or_refuses_partial_recovery() {
    for stage in 0..5 {
        for partial in [false, true] {
            let mut f = Fixture::new(true, false);
            let request = f.requests[0].encode(f.a.call.request_bytes).unwrap();
            let result = f.results[0].encode(f.a.call.result_bytes).unwrap();
            let events = [
                EventRecord::DataStarted {
                    ordinal: 0,
                    admission: f.a.clone(),
                },
                EventRecord::DataPrepared {
                    ordinal: 0,
                    request: EvidenceRecord::new(&request),
                },
                EventRecord::DataSubmissionReserved { ordinal: 0 },
                EventRecord::DataSettled {
                    ordinal: 0,
                    result: EvidenceRecord::new(&result),
                },
                EventRecord::DataAccepted {
                    ordinal: 0,
                    payload: EvidenceRecord::new(b"hello"),
                },
            ];
            let evidence = [
                None,
                Some(("data-0000-request.bin", request.as_slice())),
                None,
                Some(("data-0000-result.bin", result.as_slice())),
                None,
            ];
            for n in 0..stage {
                f.op.commit(&events[n], evidence[n]).unwrap();
            }
            assert!(
                f.op.commit_with(&events[stage], evidence[stage], |file, bytes| {
                    file.write_all(if partial {
                        &bytes[..bytes.len() / 2]
                    } else {
                        bytes
                    })?;
                    file.sync_all()?;
                    Err(std::io::Error::other("uncertain data frame"))
                })
                .is_err()
            );
            assert!(matches!(f.op.inspect_data(0), Err(CaptureError::Unusable)));
            let Fixture {
                op,
                store,
                p,
                mut signer,
                mut provider,
                t,
                server,
                ..
            } = f;
            drop(op);
            let opened = store.open_capture(&p.request().operation_id, p.digest());
            if partial {
                assert!(opened.is_err());
            } else {
                let mut op = opened.unwrap();
                let phase = op.inspect_data(0).unwrap().unwrap().phase;
                assert_eq!(
                    phase,
                    [
                        CaptureDataPhase::SigningReserved,
                        CaptureDataPhase::Prepared,
                        CaptureDataPhase::SubmissionReserved,
                        CaptureDataPhase::Settled,
                        CaptureDataPhase::Accepted
                    ][stage]
                );
                if stage >= 3 {
                    signer.allow = false;
                    assert!(matches!(
                        runtime()
                            .block_on(op.advance_data(
                                0,
                                &events_admission(&events),
                                &t,
                                &mut signer,
                                &mut provider
                            ))
                            .unwrap(),
                        CaptureDataProgress::Ready(_)
                    ));
                }
            }
            assert_eq!(server.count(), 0);
        }
    }
}
fn events_admission(events: &[EventRecord]) -> CaptureDataAdmissionRecord {
    let EventRecord::DataStarted { admission, .. } = &events[0] else {
        unreachable!()
    };
    admission.clone()
}

#[test]
fn later_data_reads_cannot_reuse_network_or_evidence_allowance() {
    for evidence in [false, true] {
        let mut f = Fixture::with_limits(
            false,
            false,
            if evidence { 20 } else { 7 },
            if evidence {
                4 * 1024 * 1024
            } else {
                16 * 1024 * 1024
            },
        );
        if !evidence {
            f.a.call.prepare_requests = 2;
        }
        assert!(matches!(
            f.advance(0).unwrap(),
            CaptureDataProgress::Ready(_)
        ));
        let before = f.op.check().unwrap();
        assert!(matches!(
            f.advance(1),
            Err(CaptureDataAdvanceError::Capture(CaptureError::Ledger(_)))
        ));
        assert_eq!(f.op.check().unwrap(), before);
        assert_eq!(f.signer.calls, 1);
        assert_eq!(f.server.count(), 1);
    }
}

#[test]
fn changed_or_orphan_data_evidence_refuses_without_cleanup() {
    for orphan in [false, true] {
        let mut f = Fixture::new(false, false);
        assert!(matches!(
            f.advance(0).unwrap(),
            CaptureDataProgress::Ready(_)
        ));
        let path = f.ledger().parent().unwrap().join(if orphan {
            "data-0001-result.bin"
        } else {
            "data-0000-result.bin"
        });
        if orphan {
            fs::write(&path, b"orphan").unwrap();
        } else {
            let mut bytes = fs::read(&path).unwrap();
            bytes[0] ^= 1;
            fs::write(&path, &bytes).unwrap();
        }
        let bytes = fs::read(&path).unwrap();
        assert!(f.op.inspect_data(0).is_err());
        let Fixture { op, store, p, .. } = f;
        drop(op);
        assert!(
            store
                .open_capture(&p.request().operation_id, p.digest())
                .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn accepted_markers_cannot_replace_exact_authenticated_payload_identity() {
    let mut f = Fixture::new(true, false);
    let request = f.requests[0].encode(f.a.call.request_bytes).unwrap();
    let result = f.results[0].encode(f.a.call.result_bytes).unwrap();
    f.op.commit(
        &EventRecord::DataStarted {
            ordinal: 0,
            admission: f.a.clone(),
        },
        None,
    )
    .unwrap();
    f.op.commit(
        &EventRecord::DataPrepared {
            ordinal: 0,
            request: EvidenceRecord::new(&request),
        },
        Some(("data-0000-request.bin", &request)),
    )
    .unwrap();
    f.op.commit(&EventRecord::DataSubmissionReserved { ordinal: 0 }, None)
        .unwrap();
    f.op.commit(
        &EventRecord::DataSettled {
            ordinal: 0,
            result: EvidenceRecord::new(&result),
        },
        Some(("data-0000-result.bin", &result)),
    )
    .unwrap();
    f.op.commit(
        &EventRecord::DataAccepted {
            ordinal: 0,
            payload: EvidenceRecord::new(b"wrong"),
        },
        None,
    )
    .unwrap();
    f = f.reopen();
    let state = f.op.check().unwrap();
    for n in [0, 1] {
        assert!(matches!(
            f.advance(n),
            Err(CaptureDataAdvanceError::Data(DataReadError::Admission(
                CaptureError::Changed
            )))
        ));
    }
    assert_eq!(f.op.check().unwrap(), state);
    assert_eq!(f.signer.calls, 0);
    assert_eq!(f.server.count(), 0);
}

#[test]
fn complete_verification_requires_every_accepted_ordinal_and_is_offline() {
    let mut f = Fixture::new(false, false);
    for n in 0..2 {
        let before = fs::read(f.ledger()).unwrap();
        assert!(matches!(
            runtime().block_on(f.op.verify_data(&f.t, &mut f.provider)),
            Err(CaptureDataAdvanceError::Coverage)
        ));
        assert_eq!(fs::read(f.ledger()).unwrap(), before);
        assert!(matches!(
            f.advance(n).unwrap(),
            CaptureDataProgress::Ready(_)
        ));
    }
    let mut f = f.reopen();
    f.signer.allow = false;
    let before = fs::read(f.ledger()).unwrap();
    let calls = f.signer.calls;
    for _ in 0..2 {
        assert_eq!(
            runtime()
                .block_on(f.op.verify_data(&f.t, &mut f.provider))
                .unwrap(),
            10
        );
    }
    assert_eq!(f.signer.calls, calls);
    assert_eq!(fs::read(f.ledger()).unwrap(), before);
}

#[test]
fn complete_authenticated_extents_do_not_replace_original_module_hash() {
    let mut f = Fixture::with_hash(
        false,
        false,
        20,
        16 * 1024 * 1024,
        ic_host_artifacts::artifact::Sha256Digest::compute(b"otherwasm!"),
    );
    for n in 0..2 {
        f.advance(n).unwrap();
    }
    let before = fs::read(f.ledger()).unwrap();
    assert!(matches!(
        runtime().block_on(f.op.verify_data(&f.t, &mut f.provider)),
        Err(CaptureDataAdvanceError::ModuleHash)
    ));
    assert_eq!(fs::read(f.ledger()).unwrap(), before);
    assert_eq!(
        f.op.inspect_data(1).unwrap().unwrap().phase,
        CaptureDataPhase::Accepted
    );
}

mod assembly_tests;
mod final_status_tests;
