// Supported single-canister direct-management workflow assembly. See README.md.

mod application;
mod capture;
mod connection;
mod metadata;
mod restore;
mod stage;
mod transfer;

pub use application::{Application, Reply, retain_applied_reply};
pub use connection::Connection;
pub use stage::{Phase, Stage};

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord as Digest,
        attempt_journal::{AttemptBudgetRecord, MutationOutcomeRecord, MutationReceiptRequest},
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        execution_workflow::{ExecutionStagePredecessorRecord, ExecutionWorkflowRecord},
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
        ic_snapshot_reply::IcSnapshotReply,
        operation_plan::{
            OperationPlanRecord, PlanBudgetRecord, PlannedOperationRecord, PlannedOperationRequest,
        },
    },
    ops::persistence::{
        BackupLayoutGuard, ExecutionStageGuard, create_execution_workflow,
        read_execution_workflow_progress,
    },
    workflow::execution_workflow::{
        ExecutionWorkflowAdvance, ExecutionWorkflowStageDriver, RetainedStageEvidence,
        advance_execution_workflow,
    },
};
use ic_backup_agent::{PreparedUpdate, ReservedUpdate};
use ic_host_fs::durable::{RetainedWriteError, write_retained_at_with};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    os::{fd::AsFd, unix::fs::DirBuilderExt},
    path::{Path, PathBuf},
};

const TOKEN: &str = "independent-original";
const CHUNK: u64 = 1024 * 1024;

/// Durably retain create-only private evidence under a caller-owned directory.
/// Failed writes leave the original name and partial bytes for reconciliation.
/// # Errors
/// Refuses existing entries and IO failures without overwriting evidence.
pub fn retain(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let name = path.file_name().ok_or(io::ErrorKind::InvalidInput)?;
    let directory = fs::File::open(parent)?;
    write_retained_at_with(directory.as_fd(), name, 0o600, |file| file.write_all(bytes))
        // This example's io::Result keeps the original native kind and OS error.
        .map_err(|error| match error {
            RetainedWriteError::Create(source)
            | RetainedWriteError::Produce(source)
            | RetainedWriteError::SyncFile(source)
            | RetainedWriteError::SyncDirectory(source) => source,
        })
}

/// Read bounded regular evidence without following a final symlink.
/// # Errors
/// Rejects unsafe, missing, excessive or unreadable files.
pub fn read(path: &Path) -> io::Result<Vec<u8>> {
    ic_host_fs::read::read_file_no_follow(path, 4 * 1024 * 1024).map_err(error)
}

/// Preserve a typed boundary error, including any returned raw reply, without journal changes.
pub fn error(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::other(error)
}

/// Retain one original operation's signed envelope and ingress ID before submission.
/// # Errors
/// Refuses unsupported requests, duplicate witnesses and IO failures.
pub fn retain_ingress(
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

/// One assembly over original capture/restore catalogs and a mandatory application adapter.
pub struct Driver<'application, A> {
    root: PathBuf,
    config: Connection,
    capture: ExecutionWorkflowRecord,
    restore: ExecutionWorkflowRecord,
    application: &'application A,
}

impl<'application, A: Application> Driver<'application, A> {
    /// Bind this assembly to explicit connection/application inputs without effects.
    /// The caller owns a stable private root and all application evidence lifetimes.
    pub fn new(root: PathBuf, config: Connection, application: &'application A) -> Self {
        let capture = workflow(&config, None);
        let restore = workflow(&config, Some(&capture.digest()));
        Self {
            root,
            config,
            capture,
            restore,
            application,
        }
    }

    /// Freeze both original catalogs once before any capture effect.
    /// # Errors
    /// Occupied/unsafe paths or persistence failures retain all partial evidence.
    pub fn create(&self) -> io::Result<()> {
        for phase in [Phase::Capture, Phase::Restore] {
            let (name, original) = self.original(phase);
            fs::DirBuilder::new()
                .mode(0o700)
                .create(self.root.join(name))?;
            create_execution_workflow(
                &BackupLayoutGuard::acquire(&self.root.join(name)).map_err(error)?,
                original,
            )
            .map_err(error)?;
        }
        Ok(())
    }

    /// Inspect original stage/attempt identities without dispatch or byte verification.
    /// # Errors
    /// Refuses incomplete, missing, changed or occupied original evidence.
    pub fn inspect(
        &self,
        phase: Phase,
    ) -> io::Result<ic_backup::view::execution_workflow::ExecutionWorkflowProgressView> {
        let (name, original) = self.original(phase);
        let layout = BackupLayoutGuard::acquire(&self.root.join(name)).map_err(error)?;
        read_execution_workflow_progress(&layout, &original.digest()).map_err(error)
    }

    /// Advance a finite number of new stages after replaying every occupied original.
    /// Pending/partially settled stages refuse; no allowance or learned digest is invented.
    /// # Errors
    /// Retains original spending, signed evidence, partial bytes and references on failure.
    pub async fn advance(
        &mut self,
        phase: Phase,
        limit: std::num::NonZeroUsize,
    ) -> io::Result<ExecutionWorkflowAdvance> {
        let (name, original) = self.original(phase);
        let original = original.clone();
        let layout = BackupLayoutGuard::acquire(&self.root.join(name)).map_err(error)?;
        let evidence = retained(&layout, &original)?;
        advance_execution_workflow(&layout, &original.digest(), &evidence, limit, self)
            .await
            .map_err(error)
    }

    /// Release only this restore's source reference after original terminal admission.
    /// Also freshly qualifies already-released replay; no IC call or artifact deletion.
    /// # Errors
    /// Refuses unfinished originals, absent acceptance or unqualified terminal/command evidence.
    pub fn release(&self) -> io::Result<()> {
        let layout =
            BackupLayoutGuard::acquire(&self.root.join(Phase::Restore.name())).map_err(error)?;
        self.terminal(&layout)
    }

    fn original(&self, phase: Phase) -> (&'static str, &ExecutionWorkflowRecord) {
        let original = match phase {
            Phase::Capture => &self.capture,
            Phase::Restore => &self.restore,
        };
        (phase.name(), original)
    }

    fn payload(&self, method: Method, snapshot_id: Option<Vec<u8>>) -> IcManagementRequestRecord {
        IcManagementRequestRecord::new(IcManagementRequest {
            method,
            target: self.config.target.clone(),
            snapshot_id,
        })
        .unwrap()
    }

    fn restore_payload(
        &self,
        stage: Stage,
        destination: impl FnOnce() -> io::Result<Vec<u8>>,
    ) -> io::Result<IcManagementRequestRecord> {
        match stage {
            Stage::Load => Ok(self.payload(Method::LoadCanisterSnapshot, Some(destination()?))),
            Stage::Start => Ok(self.payload(Method::StartCanister, None)),
            _ => Err(io::ErrorKind::InvalidInput.into()),
        }
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
    fn singleton(
        &self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        request: &Digest,
    ) -> OperationPlanRecord {
        connection::singleton(
            workflow.allocation(),
            sequence,
            &self.config.target,
            request,
            AttemptBudgetRecord::new(1, 0).unwrap(),
        )
    }
}

impl<A: Application> ExecutionWorkflowStageDriver for Driver<'_, A> {
    type Error = io::Error;
    fn bind_stage(
        &mut self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> io::Result<OperationPlanRecord> {
        let stage = Stage::from_sequence(sequence)?;
        if stage == Stage::Start {
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
                let original = self.metadata_evidence()?;
                let metadata = original.decode()?;
                IcSnapshotDownloadPlan::new(workflow, Stage::Download.sequence(), &metadata, CHUNK)
                    .map_err(error)?
                    .plan()
                    .ok_or(io::ErrorKind::InvalidData)?
                    .clone()
            }
            Stage::Allocation | Stage::Upload => {
                self.bind_upload(workflow, sequence, predecessors)?
            }
            Stage::Load | Stage::Start => self.singleton(
                workflow,
                sequence,
                &self.restore_payload(stage, || self.destination())?.digest(),
            ),
        })
    }
    async fn execute_stage(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<Digest> {
        let sequence = stage.binding().stage_sequence();
        let learned = match Stage::from_sequence(sequence)? {
            Stage::Capture => self.capture_stage(stage).await?,
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

fn workflow(config: &Connection, restore: Option<&Digest>) -> ExecutionWorkflowRecord {
    let phase = if restore.is_some() {
        Phase::Restore
    } else {
        Phase::Capture
    };
    let stages = phase.stages();
    let mut previous = None;
    let mut graph = vec![];
    let mut operations = vec![];
    let purpose = restore
        .cloned()
        .unwrap_or_else(|| Digest::from_bytes(b"independent capture original"));
    for stage in stages {
        let sequence = stage.sequence();
        let mutations = stage.mutations();
        graph.push(
            EffectNodeRecord::new(EffectNodeRequest {
                operation_sequence: sequence,
                depends_on: previous.into_iter().collect(),
            })
            .unwrap(),
        );
        operations.push(
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: sequence,
                target: config.target.clone(),
                request: purpose.hash().into(),
                budget: AttemptBudgetRecord::new(mutations, 0).unwrap(),
            })
            .unwrap(),
        );
        previous = Some(sequence);
    }
    ExecutionWorkflowRecord::new(config.plan(
        EffectGraphRecord::new(graph).unwrap(),
        operations,
        PlanBudgetRecord::new(stages.iter().map(|stage| stage.mutations()).sum(), 0).unwrap(),
    ))
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
        let binding = stage::read_binding(&root, sequence)?;
        // Missing learned evidence is an explicit refusal, never a placeholder
        // digest, reconstructed oracle or permission to repeat a pending effect.
        let bytes = read(&root.join("learned.sha256"))?;
        let learned =
            Digest::from_hash(std::str::from_utf8(&bytes).map_err(error)?).map_err(error)?;
        result.insert(
            sequence,
            RetainedStageEvidence::new(binding.digest(), learned),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
