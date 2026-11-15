//! Real restore status ingress, then terminal replay with the IC server absent.
use super::*;
use ic_backup::store::{
    DirectRestoreStatus, PreparedStatus, RestoreOperation, RestorePlan, RestoreStatusSigner,
    RestoreStatusView, status_certificate_observation,
};
use ic_platform::certification::historical::{TimePolicy, TimeWindow};

struct Signer {
    calls: Arc<AtomicUsize>,
    allowed: Arc<AtomicBool>,
}
impl RestoreStatusSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [97; 32]
    }
    fn admit(&self, p: &RestorePlan, a: &StatusAdmissionRecord) -> Result<(), ()> {
        if self.allowed.load(Ordering::SeqCst)
            && a.application == [98; 32]
            && a.prepare_requests == 0
            && p.request().sender == Principal::self_authenticating(der())
        {
            Ok(())
        } else {
            Err(())
        }
    }
    async fn sign_status(
        &mut self,
        p: &RestorePlan,
        a: &StatusAdmissionRecord,
        args: &[u8],
    ) -> Result<PreparedStatus, ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let reference = now();
        let observation = status_certificate_observation(TimePolicy {
            certificate: TimeWindow {
                reference_ns: reference,
                max_age_ns: 60_000_000_000,
                max_future_skew_ns: a.submit_deadline_ns - reference + 1_000_000_000,
            },
            delegation: TimeWindow {
                reference_ns: reference,
                max_age_ns: 86_400_000_000_000,
                max_future_skew_ns: a.submit_deadline_ns - reference + 1_000_000_000,
            },
        })
        .unwrap();
        restore_preparation::signed(p, a, args, observation)
    }
}
pub(super) struct Replay {
    direct: DirectRestoreStatus<Signer>,
    calls: Arc<AtomicUsize>,
    allowed: Arc<AtomicBool>,
    path: PathBuf,
    ledger: Vec<u8>,
    request: Vec<u8>,
    result: Vec<u8>,
    view: RestoreStatusView,
}
pub(super) async fn check(
    op: &mut RestoreOperation<'_>,
    endpoint: &str,
    root: &[u8],
    path: &std::path::Path,
) -> Replay {
    let calls = Arc::new(AtomicUsize::new(0));
    let allowed = Arc::new(AtomicBool::new(true));
    let mut direct = DirectRestoreStatus::new(
        StatusHttpTransport::new(
            endpoint,
            op.inspect().unwrap().request().target.network,
            root.to_vec(),
            Duration::from_secs(30),
        )
        .unwrap(),
        Signer {
            calls: Arc::clone(&calls),
            allowed: Arc::clone(&allowed),
        },
        CaptureTiming {
            certificate_max_age_ns: 60_000_000_000,
            delegation_max_age_ns: 86_400_000_000_000,
            max_clock_skew_ns: 1_000_000_000,
        },
    );
    let reference = now();
    let a = StatusAdmissionRecord {
        provider: direct.commitment(),
        application: [98; 32],
        prepare_deadline_ns: reference + 30_000_000_000,
        submit_deadline_ns: reference + 60_000_000_000,
        prepare_requests: 0,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 256 * 1024,
    };
    op.prepare_status(&a, &mut direct).await.unwrap();
    let request = fs::read(path.with_file_name("status-request.bin")).unwrap();
    assert_eq!(
        op.submit_status(&direct).await.unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let view = op.inspect_status().unwrap();
    assert!(view.submission_reserved && view.result_retained);
    assert_eq!(view.network_requests_reserved, 1);
    assert_eq!(
        fs::read(path.with_file_name("status-request.bin")).unwrap(),
        request
    );
    Replay {
        direct,
        calls,
        allowed,
        path: path.to_path_buf(),
        ledger: fs::read(path).unwrap(),
        request,
        result: fs::read(path.with_file_name("status-result.bin")).unwrap(),
        view,
    }
}
impl Replay {
    pub(super) async fn check(self, op: &mut RestoreOperation<'_>) {
        self.allowed.store(false, Ordering::SeqCst);
        assert_eq!(
            op.submit_status(&self.direct).await.unwrap(),
            StatusProgress::Verified
        );
        assert_eq!(op.inspect_status().unwrap(), self.view);
        assert_eq!(self.calls.load(Ordering::SeqCst), 1);
        assert_eq!(fs::read(&self.path).unwrap(), self.ledger);
        assert_eq!(
            fs::read(self.path.with_file_name("status-request.bin")).unwrap(),
            self.request
        );
        assert_eq!(
            fs::read(self.path.with_file_name("status-result.bin")).unwrap(),
            self.result
        );
        fs::write(self.path.ancestors().nth(4).unwrap().join("restore-status-verified.txt"),b"real certified restore status ingress; original request/result preserved; server absent and guard revoked for terminal replay\n").unwrap();
    }
}
