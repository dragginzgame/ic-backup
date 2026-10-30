use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptBudgetRecord,
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest},
        operation_plan::{
            OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlanContextRecord,
            PlanContextRequest, PlannedOperationRecord, PlannedOperationRequest,
        },
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_operation_plan},
};
use ic_platform::agent::{AnonymousIdentity, Identity};
use std::{fs, path::PathBuf, sync::Arc};

pub fn root(label: &str) -> PathBuf {
    let selected_temp = std::env::temp_dir()
        .canonicalize()
        .expect("resolve selected fixture parent");
    ic_host_fs::testing::create_fixture_directory(
        &selected_temp,
        &format!("ic-backup-agent-{label}"),
    )
    .unwrap()
}

/// Retain exact original signed ingress privately and durably before test dispatch.
pub fn retain_signed(
    root: &std::path::Path,
    prepared: &ic_backup_agent::PreparedUpdate<'_>,
) -> std::io::Result<()> {
    for (name, bytes) in [
        ("signed-ingress.cbor", prepared.envelope().to_vec()),
        (
            "request-id.txt",
            prepared.request_id().to_string().into_bytes(),
        ),
    ] {
        retain_file(&root.join(name), &bytes)?;
    }
    fs::File::open(root)?.sync_all()
}

/// Create and sync one private evidence file, preserving partial bytes on failure.
fn retain_file(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Retain a single evidence file and synchronize its existing parent.
#[allow(
    dead_code,
    reason = "shared support also compiles in binaries retaining only signed ingress"
)]
pub fn retain_bytes(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    retain_file(path, bytes)?;
    fs::File::open(path.parent().unwrap())?.sync_all()
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
#[allow(
    dead_code,
    reason = "HTTP fixtures bind the default declaration; PocketIC supplies its actual context before admission"
)]
pub fn plan(target: &str, digest: &ArtifactChecksumRecord, sequence: u64) -> OperationPlanRecord {
    OperationPlanRecord::new(plan_request(target, digest, sequence)).unwrap()
}

pub fn plan_request(
    target: &str,
    digest: &ArtifactChecksumRecord,
    sequence: u64,
) -> OperationPlanRequest {
    OperationPlanRequest {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: "ab".repeat(32),
            caller: "2vxsx-fae".into(),
            release: "cd".repeat(32),
        })
        .unwrap(),
        inventory: InventoryRecord::new(vec![
            InventoryTargetRecord::new(&InventoryTargetRequest {
                canister_id: target.into(),
                parent_canister_id: None,
                role: None,
                module_hash: None,
            })
            .unwrap(),
        ])
        .unwrap(),
        selected_targets: vec![target.into()],
        graph: EffectGraphRecord::new(vec![
            EffectNodeRecord::new(EffectNodeRequest {
                operation_sequence: sequence,
                depends_on: vec![],
            })
            .unwrap(),
        ])
        .unwrap(),
        operations: vec![
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: sequence,
                target: target.into(),
                request: digest.hash().into(),
                budget: AttemptBudgetRecord::new(1, 1).unwrap(),
            })
            .unwrap(),
        ],
        budget: PlanBudgetRecord::new(1, 1).unwrap(),
    }
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
