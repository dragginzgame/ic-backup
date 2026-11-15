//! Fixture-only loss after a real certified reply, before workflow settlement.
use ic_backup::store::{
    CaptureCall, CapturePlan, CaptureProvider, PreparedStatus, StatusAdmissionRecord,
    StatusEvidence, StatusProvider,
};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

pub(super) struct LostReply<P> {
    inner: P,
    directory: PathBuf,
    interrupt: bool,
    take: Option<PreparedStatus>,
}
impl<P> LostReply<P> {
    pub fn take_request(&self) -> &PreparedStatus {
        self.take.as_ref().expect("prepared take")
    }
    pub fn new(inner: P, directory: PathBuf, interrupt: bool) -> Self {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        Self {
            inner,
            directory,
            interrupt,
            take: None,
        }
    }
}
fn retain(directory: &Path, evidence: &StatusEvidence) {
    for (name, bytes) in [
        ("response.bin", evidence.response.as_slice()),
        ("proof.bin", evidence.proof.as_slice()),
        ("rejected.bin", &[u8::from(evidence.rejected)]),
    ] {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(directory.join(name))
            .unwrap();
        file.write_all(bytes).unwrap();
        file.sync_all().unwrap();
    }
    fs::File::open(directory).unwrap().sync_all().unwrap();
}
pub(super) fn load(directory: &Path, limit: u32) -> StatusEvidence {
    let read = |name: &str| {
        let mut bytes = Vec::new();
        fs::File::open(directory.join(name))
            .unwrap()
            .take(u64::from(limit) + 1)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(bytes.len() <= limit as usize);
        bytes
    };
    let rejected = read("rejected.bin");
    assert!(matches!(rejected.as_slice(), [0] | [1]));
    let response = read("response.bin");
    let proof = read("proof.bin");
    assert!(response.len() + proof.len() + 5 <= limit as usize);
    StatusEvidence {
        rejected: rejected == [1],
        response,
        proof,
    }
}
impl<P: CaptureProvider + Send> StatusProvider for LostReply<P> {
    type Error = ();
    fn admit(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        args: &[u8],
        r: Option<&PreparedStatus>,
    ) -> Result<(), ()> {
        self.inner.admit(p, a, args, r).map_err(|_| ())
    }
    async fn prepare(
        &mut self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        args: &[u8],
    ) -> Result<PreparedStatus, ()> {
        self.inner.prepare(p, a, args).await.map_err(|_| ())
    }
    async fn submit_once(
        &mut self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        r: &PreparedStatus,
    ) -> Result<StatusEvidence, ()> {
        self.inner.submit_once(p, a, r).await.map_err(|_| ())
    }
    fn authenticate(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        r: &PreparedStatus,
        e: &StatusEvidence,
    ) -> Result<(), ()> {
        self.inner.authenticate(p, a, r, e).map_err(|_| ())
    }
}
impl<P: CaptureProvider + Send> CaptureProvider for LostReply<P> {
    fn admit_call(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        call: CaptureCall,
        r: Option<&PreparedStatus>,
    ) -> Result<(), ()> {
        self.inner.admit_call(p, a, call, r).map_err(|_| ())
    }
    async fn prepare_call(
        &mut self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        call: CaptureCall,
    ) -> Result<PreparedStatus, ()> {
        self.inner.prepare_call(p, a, call).await.map_err(|_| ())
    }
    async fn submit_call(
        &mut self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        call: CaptureCall,
        r: &PreparedStatus,
    ) -> Result<StatusEvidence, ()> {
        if call == CaptureCall::TakeSnapshot {
            self.take = Some(PreparedStatus {
                request_id: r.request_id,
                expires_ns: r.expires_ns,
                envelope: r.envelope.clone(),
                observation: r.observation.clone(),
            });
        }
        let evidence = self
            .inner
            .submit_call(p, a, call, r)
            .await
            .map_err(|_| ())?;
        if self.interrupt && call == CaptureCall::TakeSnapshot {
            retain(&self.directory, &evidence);
            return Err(());
        }
        Ok(evidence)
    }
    fn authenticate_call(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        call: CaptureCall,
        r: &PreparedStatus,
        e: &StatusEvidence,
    ) -> Result<(), ()> {
        self.inner
            .authenticate_call(p, a, call, r, e)
            .map_err(|_| ())
    }
}
