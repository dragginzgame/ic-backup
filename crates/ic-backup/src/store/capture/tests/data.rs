//! A completed native retained-evidence capture; no simulated IC effects.
use super::{
    http::{Response, Server},
    *,
};
use crate::store::{BackupHttpTransport, BackupStore, tests::root};
use std::{convert::Infallible, path::PathBuf, time::Duration};

pub(in crate::store) struct UnusedSigner;
impl BackupSigner for UnusedSigner {
    type Error = Infallible;
    fn protocol(&self) -> [u8; 32] {
        [2; 32]
    }

    async fn sign(&mut self, _: OperationSigningRequest<'_>) -> Result<Vec<u8>, Infallible> {
        panic!("native retained source never signs")
    }
}
pub(in crate::store) struct UnusedGuard;
impl BackupApplicationGuard for UnusedGuard {
    type Error = Infallible;
    fn protocol(&self) -> [u8; 32] {
        [3; 32]
    }

    fn admit(
        &self,
        _: crate::store::OperationPlan<'_>,
        _: OperationAction,
        _: &[u8; 32],
    ) -> Result<(), Infallible> {
        panic!("native retained source never reads live authority")
    }
}
pub(in crate::store) type FixtureConnection = BackupConnection<UnusedSigner, UnusedGuard>;
pub(in crate::store) struct Fixture {
    pub store: BackupStore,
    pub p: CapturePlan,
    pub op: CaptureOperation,
    pub root: PathBuf,
}
impl Fixture {
    pub fn replan(&self, request: CaptureRequest) -> CapturePlan {
        CapturePlan::new(request, self.p.profile().clone()).unwrap()
    }

    pub fn ledger(&self) -> PathBuf {
        self.root
            .join("operations")
            .join(&self.p.request().operation_id)
            .join("journal.bin")
    }
}

pub(in crate::store) fn settled(mixed: bool) -> (Fixture, Server, FixtureConnection) {
    let root = root("capture-source");
    let store = BackupStore::initialize(&root).unwrap();
    let server = Server::new(Response::Disconnect);
    let mut request = signatures::fixture(signatures::Scheme::Ed25519)
        .0
        .request()
        .clone();
    request.module_sha256 = BackupDigest::compute(b"helloworld");
    request.trust_root = BackupDigest::compute(&certificates::der(1));
    request.limits.extent_bytes = 5;
    request.limits.payload_bytes = 64;
    request.limits.evidence_bytes = 16 * 1024 * 1024;
    request.limits.network_requests = 20;
    request.limits.signing_requests = 20;
    let transport = BackupHttpTransport::new(
        &server.endpoint,
        request.target.network,
        certificates::der(1),
        Duration::from_secs(1),
    )
    .unwrap();
    let connection = BackupConnection::new(transport, UnusedSigner, UnusedGuard);
    let p = connection
        .plan(
            request,
            OperationOptions {
                application: [4; 32],
                snapshot_capacity: 2,
                prepare_duration_ns: 10,
                submit_duration_ns: 20,
                timing: CaptureTiming {
                    certificate_max_age_ns: 0,
                    delegation_max_age_ns: 90,
                    max_clock_skew_ns: 0,
                },
                signer_network: 0,
                request_bytes: 4096,
                result_bytes: 4096,
            },
        )
        .unwrap();
    let mut op = store
        .create_capture(CapturePlan::decode(p.encoded()).unwrap(), p.digest())
        .unwrap();
    crate::store::operation::tests::fixtures::settle(&mut op, mixed);
    (Fixture { store, p, op, root }, server, connection)
}
