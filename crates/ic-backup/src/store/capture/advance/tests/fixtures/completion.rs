use super::super::*;
use crate::store::{
    BackupSummary, CaptureCompletionError, ScanLimits, StatusProgress,
    capture::advance::tests::fixtures::{
        data::Fixture,
        final_status::{advance, assembled, provider, request, result},
        http::{HttpProvider, Server, runtime},
    },
};
use std::{fs, path::PathBuf};

pub(in crate::store) fn settled() -> (Fixture, Server, HttpProvider) {
    let mut f = assembled(20);
    let r = request(&f, false);
    let e = result(&f, &r, false);
    let (server, mut p) = provider(&f, r, &e);
    let a = f.a.call.clone();
    assert_eq!(
        advance(&mut f, &a, &mut p).unwrap(),
        StatusProgress::Verified
    );
    p.replay = true;
    (f, server, p)
}

pub(in crate::store) fn complete(
    f: &mut Fixture,
    p: &mut HttpProvider,
) -> Result<BackupSummary, CaptureCompletionError<StatusHttpError>> {
    runtime().block_on(f.op.complete_capture(&f.t, p))
}

pub(in crate::store) fn manifest(f: &Fixture) -> PathBuf {
    f.root
        .join("bundles")
        .join(&f.p.request().operation_id)
        .join("manifest.json")
}

pub(in crate::store) fn limits() -> ScanLimits {
    ScanLimits {
        entries: 10.try_into().unwrap(),
        bytes: (1024 * 1024).try_into().unwrap(),
        results: 10.try_into().unwrap(),
    }
}

pub(in crate::store) fn replace_owner(f: &Fixture) -> (PathBuf, Vec<u8>) {
    let path = f
        .root
        .join("targets")
        .join(f.p.request().target.network.to_string())
        .join(f.p.request().target.canister.to_text())
        .join("owner.json");
    let mut request = f.p.request().clone();
    request.operation_id = "ef".repeat(16);
    let plan = CapturePlan::new(request).unwrap();
    let bytes = crate::store::capture::owner(&plan).unwrap();
    // Synthetic successor image: creation/atomic claim replacement is separate work.
    fs::write(&path, &bytes).unwrap();
    (path, bytes)
}

pub(in crate::store) fn rebound_transfer_event(
    f: &mut Fixture,
    p: &mut HttpProvider,
    next: &CapturePlan,
) -> Vec<u8> {
    let state = f.op.check().unwrap();
    let metadata_state = state.metadata.as_ref().unwrap();
    let snapshot = runtime()
        .block_on(f.op.metadata_snapshot::<_, std::convert::Infallible>(
            &metadata_state.admission,
            &f.t,
            p,
            &state,
        ))
        .unwrap();
    let metadata =
        f.op.retained_metadata::<std::convert::Infallible, std::convert::Infallible>(
            &state, &snapshot, &f.t,
        )
        .unwrap();
    let binding = next
        .plan_transfer(&snapshot.id, &metadata)
        .unwrap()
        .binding(
            metadata_state.request.as_ref().unwrap(),
            metadata_state.result.as_ref().unwrap(),
        );
    serde_json::to_vec(&EventRecord::TransferBound { binding }).unwrap()
}
