//! Canonical normal-consumer program; the parent owns the isolated application.

mod application;
mod restore;
mod transfer;

mod config;
mod stage;
use config::Config;
use stage::Stage;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord as Digest,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
        },
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_reply::IcSnapshotReply,
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, ExecutionStageGuard, create_execution_workflow,
        read_execution_workflow_progress,
    },
    workflow::{
        execution_workflow::{
            ExecutionWorkflowAdvance, ExecutionWorkflowStageDriver, RetainedStageEvidence,
            advance_execution_workflow,
        },
        ic_snapshot_capture::capture_snapshot,
    },
};
use ic_backup_agent::{AgentTransport, PreparedUpdate, ReservedUpdate};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

const TOKEN: &str = "independent-original";
const CHUNK: u64 = 1024 * 1024;

pub(super) fn retain(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::File::open(path.parent().unwrap())?.sync_all()
}

fn read(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(bytes)
}

fn error(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

fn signed(
    root: &Path,
    request: &ReservedUpdate<'_>,
    prepared: &PreparedUpdate<'_>,
) -> io::Result<()> {
    let sequence = match request {
        ReservedUpdate::Mutation(r) => r.authority().binding().operation_sequence(),
        ReservedUpdate::TransferRead(r) => r.authority().binding().operation_sequence(),
        ReservedUpdate::Upload(r) => r.authority().binding().operation_sequence(),
        _ => return Err(io::ErrorKind::InvalidInput.into()),
    };
    // Create-only original-operation names refuse another signature/dispatch even
    // if an expiry change would produce a different request ID.
    retain(
        &root.join(format!("signed-{sequence}.cbor")),
        prepared.envelope(),
    )?;
    retain(
        &root.join(format!("request-{sequence}.txt")),
        prepared.request_id().to_string().as_bytes(),
    )
}

fn receipt(
    root: &Path,
    sequence: u64,
    attempt: u32,
    request: &Digest,
    arguments: &[u8],
    reply: &[u8],
) -> io::Result<MutationReceiptRequest> {
    retain(
        &root.join(format!("arguments-{sequence}.candid")),
        arguments,
    )?;
    retain(&root.join(format!("reply-{sequence}.candid")), reply)?;
    // The Agent authenticates this reply to the configured root/effective target.
    // Exclusive fixture/controller custody independently attributes its one ingress.
    Ok(MutationReceiptRequest {
        attempt,
        request: request.hash().into(),
        outcome: MutationOutcomeRecord::Applied,
        evidence: Digest::from_bytes(reply).hash().into(),
    })
}

struct Driver {
    root: PathBuf,
    config: Config,
    capture: ExecutionWorkflowRecord,
    restore: ExecutionWorkflowRecord,
    mode: String,
}

impl Driver {
    fn payload(&self, method: Method, snapshot_id: Option<Vec<u8>>) -> IcManagementRequestRecord {
        IcManagementRequestRecord::new(IcManagementRequest {
            method,
            target: self.config.target.clone(),
            snapshot_id,
        })
        .unwrap()
    }
    fn source(&self) -> PathBuf {
        Stage::Download.path(&self.root)
    }
    fn raw(&self, stage: Stage) -> io::Result<Vec<u8>> {
        read(
            &stage
                .path(&self.root)
                .join(format!("reply-{}.candid", stage.sequence())),
        )
    }
    fn metadata_request(&self) -> io::Result<IcSnapshotMetadataRequest> {
        let payload = self.payload(Method::TakeCanisterSnapshot, None);
        let raw = self.raw(Stage::Capture)?;
        let reply = IcSnapshotReply::decode(&payload, &raw).map_err(error)?;
        IcSnapshotMetadataRequest::new(&self.config.target, reply.snapshots()[0].id())
            .map_err(error)
    }
    fn transport(&self, plan: &OperationPlanRecord) -> AgentTransport {
        AgentTransport::new(
            plan.context().clone(),
            &self.config.endpoint,
            Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(&[7; 32])),
            self.config.key.clone(),
            Duration::from_secs(30),
        )
        .unwrap()
    }
    fn singleton(
        &self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        request: &Digest,
    ) -> OperationPlanRecord {
        let mut value = serde_json::to_value(workflow).unwrap()["allocation"].clone();
        value["graph"]["nodes"] = json!([{"operation_sequence":sequence,"depends_on":[]}]);
        value["operations"] = json!([{"operation_sequence":sequence,"target":self.config.target,"request":request.hash(),"budget":{"mutations":1,"observations":0}}]);
        value["budget"] = json!({"mutations":1,"observations":0});
        serde_json::from_value(value).unwrap()
    }
}

impl ExecutionWorkflowStageDriver for Driver {
    type Error = io::Error;
    fn bind_stage(
        &mut self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> io::Result<OperationPlanRecord> {
        let stage = Stage::from_sequence(sequence)?;
        if stage == Stage::Start {
            self.admit()?;
            self.acceptance()?;
        }
        Ok(match stage {
            Stage::Capture => self.singleton(
                workflow,
                Stage::Capture.sequence(),
                &self.payload(Method::TakeCanisterSnapshot, None).digest(),
            ),
            Stage::Metadata => self.singleton(
                workflow,
                Stage::Metadata.sequence(),
                &self.metadata_request()?.digest(),
            ),
            Stage::Download => {
                let request = self.metadata_request()?;
                let raw = self.raw(Stage::Metadata)?;
                let metadata = IcSnapshotMetadataReply::decode(&request, &raw).map_err(error)?;
                IcSnapshotDownloadPlan::new(workflow, Stage::Download.sequence(), &metadata, CHUNK)
                    .map_err(error)?
                    .plan()
                    .unwrap()
                    .clone()
            }
            Stage::Allocation | Stage::Upload => {
                self.bind_upload(workflow, sequence, predecessors)?
            }
            Stage::Load | Stage::Start => self.singleton(
                workflow,
                sequence,
                &self
                    .payload(
                        if stage == Stage::Load {
                            Method::LoadCanisterSnapshot
                        } else {
                            Method::StartCanister
                        },
                        (stage == Stage::Load).then(|| self.destination().unwrap()),
                    )
                    .digest(),
            ),
        })
    }
    async fn execute_stage(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<Digest> {
        let sequence = stage.binding().stage_sequence();
        let learned = match Stage::from_sequence(sequence)? {
            Stage::Capture => {
                let payload = self.payload(Method::TakeCanisterSnapshot, None);
                let transport = self.transport(stage.plan());
                let root = stage.layout().map_err(error)?.root();
                let mut provider = ic_backup_agent::AgentUpdateProvider::new(
                    &transport,
                    |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                        if self.mode == "capture-fail-ingress" {
                            return Err(
                                ic_backup::ports::ic_mutation::IcMutationProviderError::Unavailable,
                            );
                        }
                        signed(root, request, prepared).map_err(|_| {
                            ic_backup::ports::ic_mutation::IcMutationProviderError::Unavailable
                        })
                    },
                );
                let reply = capture_snapshot(stage, 0, &payload, &mut provider, async |_| {
                    self.admit().map(|_| ())
                })
                .await
                .map_err(error)?;
                if self.mode == "capture-fail-retention" {
                    return Err(io::Error::other("fixture durable reply retention refused"));
                }
                let receipt = receipt(
                    root,
                    0,
                    1,
                    &payload.digest(),
                    payload.arguments(),
                    &reply.input().reply,
                )?;
                let mut journal = AttemptJournalGuard::open(
                    stage.layout().map_err(error)?,
                    &stage.plan().attempt_authority(0).map_err(error)?,
                )
                .map_err(error)?;
                journal.record_mutation(receipt).map_err(error)?;
                IcSnapshotReply::decode(&payload, &reply.input().reply)
                    .map_err(error)?
                    .digest()
            }
            Stage::Metadata | Stage::Download => self.transfer(stage).await?,
            Stage::Allocation | Stage::Upload => self.upload(stage).await?,
            Stage::Load | Stage::Start => self.restore_stage(stage).await?,
        };
        retain(
            &stage.layout().map_err(error)?.root().join("learned.sha256"),
            learned.hash().as_bytes(),
        )?;
        Ok(learned)
    }
}

fn workflow(config: &Config, restore: Option<&Digest>) -> ExecutionWorkflowRecord {
    let stages = if restore.is_some() {
        Stage::RESTORE.as_slice()
    } else {
        Stage::CAPTURE.as_slice()
    };
    let mut previous = None;
    let mut graph = vec![];
    let mut operations = vec![];
    let purpose = restore
        .cloned()
        .unwrap_or_else(|| Digest::from_bytes(b"independent capture original"));
    for stage in stages {
        let sequence = stage.sequence();
        let mutations = stage.mutations();
        graph.push(json!({"operation_sequence":sequence,"depends_on":previous.into_iter().collect::<Vec<u64>>() }));
        operations.push(json!({"operation_sequence":sequence,"target":config.target,"request":purpose.hash(),"budget":{"mutations":mutations,"observations":0}}));
        previous = Some(sequence);
    }
    ExecutionWorkflowRecord::new(serde_json::from_value(json!({"version":1,"context":config.context,
        "inventory":{"version":1,"targets":[{"canister_id":config.target,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[config.target],"graph":{"version":1,"nodes":graph},"operations":operations,
        "budget":{"mutations":stages.iter().map(|stage|stage.mutations()).sum::<u32>(),"observations":0}})).unwrap())
}

fn retained(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
) -> io::Result<BTreeMap<u64, RetainedStageEvidence>> {
    let mut result = BTreeMap::new();
    for operation in workflow.stages() {
        let sequence = operation.operation_sequence();
        let root = layout.root().join(format!("execution-stage-{sequence}"));
        if !root.exists() {
            continue;
        }
        let binding: ExecutionStageBindingRecord =
            serde_json::from_slice(&read(&root.join("stage-binding.json"))?)?;
        // Missing learned evidence is an explicit refusal, never a placeholder
        // digest, reconstructed oracle or permission to repeat a pending effect.
        let learned =
            Digest::from_hash(&fs::read_to_string(root.join("learned.sha256"))?).map_err(error)?;
        result.insert(
            sequence,
            RetainedStageEvidence::new(binding.digest(), learned),
        );
    }
    Ok(result)
}

#[tokio::main(flavor = "multi_thread")]
pub(super) async fn entry() {
    if let Err(error) = run().await {
        eprintln!("consumer refused: {error}");
        std::process::exit(1);
    }
}

async fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let root = PathBuf::from(&args[2]);
    let mode = args[1]
        .to_str()
        .ok_or(io::ErrorKind::InvalidInput)?
        .to_owned();
    if mode.starts_with("application-") {
        return application::run(&root, &mode).await;
    }
    if !matches!(
        mode.as_str(),
        "capture"
            | "capture-continue"
            | "restore"
            | "start"
            | "replay"
            | "lose-load"
            | "inspect-capture"
            | "inspect-restore"
            | "capture-fail-ingress"
            | "capture-fail-retention"
            | "capture-fail-partial"
    ) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let config = Config::read(&root)?;
    let capture = workflow(&config, None);
    let restore = workflow(&config, Some(&capture.digest()));
    let mut driver = Driver {
        root: root.clone(),
        config,
        capture: capture.clone(),
        restore: restore.clone(),
        mode: mode.clone(),
    };
    if mode == "capture" || mode.starts_with("capture-fail-") {
        for (name, original) in [("capture", &capture), ("restore", &restore)] {
            fs::DirBuilder::new().mode(0o700).create(root.join(name))?;
            create_execution_workflow(
                &BackupLayoutGuard::acquire(&root.join(name)).map_err(error)?,
                original,
            )
            .map_err(error)?;
        }
    }
    let is_capture = mode.starts_with("capture") || mode == "inspect-capture";
    let name = if is_capture { "capture" } else { "restore" };
    let original = if is_capture { &capture } else { &restore };
    let layout = BackupLayoutGuard::acquire(&root.join(name)).map_err(error)?;
    let progress = read_execution_workflow_progress(&layout, &original.digest()).map_err(error)?;
    assert_eq!(progress.workflow, original.digest());
    assert_eq!(progress.stages.len(), original.stages().len());
    if mode.starts_with("inspect-") {
        serde_json::to_writer(io::stdout(), &progress)?;
        println!();
        return Ok(());
    }
    if mode == "replay" {
        return driver.terminal(&layout);
    }
    let evidence = retained(&layout, original)?;
    let limit = std::num::NonZeroUsize::new(if matches!(mode.as_str(), "restore" | "lose-load") {
        3
    } else {
        usize::MAX
    })
    .unwrap();
    let result =
        advance_execution_workflow(&layout, &original.digest(), &evidence, limit, &mut driver)
            .await;
    match result {
        Ok(ExecutionWorkflowAdvance::Complete(_)) if mode == "start" => driver.terminal(&layout),
        Ok(ExecutionWorkflowAdvance::Complete(_)) => Ok(()),
        Ok(ExecutionWorkflowAdvance::Paused) if mode == "restore" => Ok(()),
        Ok(ExecutionWorkflowAdvance::Paused) => Err(io::ErrorKind::InvalidData.into()),
        Err(e) => Err(error(e)),
    }
}
