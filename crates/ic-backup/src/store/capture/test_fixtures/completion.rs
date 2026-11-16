use super::data::{Fixture, FixtureConnection};
use super::http::{Server, runtime};
use crate::store::{BackupSummary, OperationAdvanceError, OperationProgress, ScanLimits};
use std::{convert::Infallible, fs, path::PathBuf};
pub(in crate::store) fn settled() -> (Fixture, Server, FixtureConnection) {
    super::data::settled(false)
}
pub(in crate::store) fn mixed() -> (Fixture, Server, FixtureConnection) {
    super::data::settled(true)
}
pub(in crate::store) fn complete(
    f: &mut Fixture,
    connection: &mut FixtureConnection,
) -> Result<BackupSummary, OperationAdvanceError<Infallible, Infallible>> {
    match runtime().block_on(f.op.advance(connection))? {
        OperationProgress::Complete(summary) => Ok(*summary),
        _ => panic!("retained complete capture"),
    }
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
    let plan = f.replan(request);
    let bytes = super::super::owner(&plan).unwrap();
    fs::write(&path, &bytes).unwrap();
    (path, bytes)
}
