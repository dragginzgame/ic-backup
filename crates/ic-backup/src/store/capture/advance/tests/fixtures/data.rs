use super::super::*;
use crate::store::{
    BackupStore, CaptureDataAdmissionRecord, CaptureDataAdvanceError, CaptureDataProgress,
    CaptureDataSigner,
    capture::advance::tests::{
        certificate_tests::{reply_tree, signed},
        fixtures::{
            durable_lookup::reopen,
            http::{HttpProvider, Response, Server, runtime},
            metadata::{authority, captured, input, response},
            signatures::{Scheme, change},
        },
    },
    tests::root,
};
use ic_platform::management_canister_types::{ReadCanisterSnapshotDataResult, Snapshot};
use std::{cell::Cell, future::pending, path::PathBuf, time::Duration};

pub(in crate::store) struct DataSigner {
    pub(in crate::store) base: PreparedStatus,
    pub(in crate::store) calls: usize,
    pub(in crate::store) admits: Cell<usize>,
    pub(in crate::store) allow: bool,
    pub(in crate::store) fail: bool,
    pub(in crate::store) hang: bool,
    pub(in crate::store) stop_prepared: bool,
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

pub(in crate::store) fn prepared_data(base: &PreparedStatus, args: &[u8]) -> PreparedStatus {
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

pub(in crate::store) fn data_evidence(r: &PreparedStatus, payload: &[u8]) -> StatusEvidence {
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

pub(in crate::store) struct Fixture {
    pub(in crate::store) store: BackupStore,
    pub(in crate::store) p: CapturePlan,
    pub(in crate::store) op: CaptureOperation,
    pub(in crate::store) t: StatusHttpTransport,
    pub(in crate::store) provider: HttpProvider,
    pub(in crate::store) signer: DataSigner,
    pub(in crate::store) a: CaptureDataAdmissionRecord,
    pub(in crate::store) requests: Vec<PreparedStatus>,
    pub(in crate::store) results: Vec<StatusEvidence>,
    pub(in crate::store) server: Server,
    pub(in crate::store) root: PathBuf,
}

impl Fixture {
    pub(in crate::store) fn new(disconnect: bool, bad_payload: bool) -> Self {
        Self::with_limits(disconnect, bad_payload, 20, 16 * 1024 * 1024)
    }
    pub(in crate::store) fn with_limits(
        disconnect: bool,
        bad_payload: bool,
        network: u32,
        evidence: u64,
    ) -> Self {
        Self::with_hash(
            disconnect,
            bad_payload,
            network,
            evidence,
            ic_host_artifacts::artifact::Sha256Digest::compute(b"helloworld"),
        )
    }
    pub(in crate::store) fn with_hash(
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
    pub(in crate::store) fn with_parts(
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
        crate::store::capture::advance::tests::fixtures::schedule::settle(&mut op, &s, &r, &e, &ma);
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
    pub(in crate::store) fn advance(
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
    pub(in crate::store) fn ledger(&self) -> PathBuf {
        self.root
            .join("operations")
            .join(&self.p.request().operation_id)
            .join("ledger.bin")
    }
    pub(in crate::store) fn reopen(self) -> Self {
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
