use ic_agent::{Identity, identity::AnonymousIdentity};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_operation_plan},
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT: AtomicU64 = AtomicU64::new(0);
pub fn root(label: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ic-backup-agent-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
    }
    p
}

/// Retain exact original signed ingress privately and durably before test dispatch.
pub fn retain_signed(
    root: &std::path::Path,
    prepared: &ic_backup_agent::PreparedUpdate<'_>,
) -> std::io::Result<()> {
    use std::io::Write;
    for (name, bytes) in [
        ("signed-ingress.cbor", prepared.envelope().to_vec()),
        (
            "request-id.txt",
            prepared.request_id().to_string().into_bytes(),
        ),
    ] {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(root.join(name))?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }
    fs::File::open(root)?.sync_all()
}
pub fn identity() -> Arc<dyn Identity> {
    Arc::new(AnonymousIdentity)
}
pub fn payload(
    method: IcManagementMethodRecord,
    target: &str,
    snapshot_id: Option<Vec<u8>>,
) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: target.into(),
        snapshot_id,
    })
    .unwrap()
}
pub fn plan(target: &str, digest: &ArtifactChecksumRecord, sequence: u64) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":target,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[target],"graph":{"version":1,"nodes":[{"operation_sequence":sequence,"depends_on":[]}]},
        "operations":[{"operation_sequence":sequence,"target":target,"request":digest.hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
    })).unwrap()
}
pub fn retain<'a>(
    layout: &'a BackupLayoutGuard,
    plan: &OperationPlanRecord,
    sequence: u64,
) -> AttemptJournalGuard<'a> {
    create_operation_plan(layout, plan).unwrap();
    let mut guard =
        AttemptJournalGuard::create(layout, plan.attempt_authority(sequence).unwrap()).unwrap();
    guard.reserve_planned_mutation(&plan.digest()).unwrap();
    guard
}
