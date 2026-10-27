//! Public host-ingress accounting for application-owned dispatchers.
//! Native fixture evidence only: no Wire codec, Canic authority or IC effects are qualified.

mod ready;
mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord as Digest,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, ExecutionStageGuard},
    workflow::execution_workflow::{
        ExecutionWorkflowRunError, ExecutionWorkflowStageDriver, RetainedStageEvidence,
        run_new_execution_workflow, run_retained_execution_workflow,
    },
};
use serde_json::json;
use std::{collections::BTreeMap, fs, io, os::unix::fs::DirBuilderExt, path::PathBuf};

const ROOT: &str = "renrk-eyaaa-aaaaa-aaada-cai";
const STAGE: u64 = 7;
const OPERATION: u64 = 42;

fn original_plan(request: &Digest) -> OperationPlanRecord {
    support::single_operation_plan(ROOT, OPERATION, request.hash())
}

// The application owns the exact envelope encoding/digest and current admission.
// This JSON is fixture data, not Canic's Wire representation or an authority source.
fn original_request() -> serde_json::Value {
    json!({
        "network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32),
        "receiver": ROOT, "method": "fixture_checkpoint_advance", "mode": "update",
        "arguments": [0, 255, 17], "unit": [ROOT, "rno2w-sqaaa-aaaaa-aaacq-cai"]
    })
}

struct ApplicationDriver {
    root: PathBuf,
    plan: OperationPlanRecord,
    request: Vec<u8>,
    lose_reply: bool,
    callbacks: usize,
    binding: Option<ExecutionStageBindingRecord>,
}

impl ApplicationDriver {
    fn new(root: PathBuf, plan: OperationPlanRecord, request: Vec<u8>) -> Self {
        Self {
            root,
            plan,
            request,
            lose_reply: false,
            callbacks: 0,
            binding: None,
        }
    }
}

impl ExecutionWorkflowStageDriver for ApplicationDriver {
    type Error = io::Error;

    fn bind_stage(
        &mut self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> Result<OperationPlanRecord, Self::Error> {
        self.callbacks += 1;
        assert_eq!(sequence, STAGE);
        assert_eq!(predecessors, []);
        let raw = fs::read(self.root.join("original-request.json"))?;
        if raw != self.request
            || self.plan.operation(OPERATION).unwrap().request() != Digest::from_bytes(&raw).hash()
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        self.binding =
            Some(ExecutionStageBindingRecord::new(workflow, sequence, &self.plan, vec![]).unwrap());
        Ok(self.plan.clone())
    }

    fn execute_stage(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> impl std::future::Future<Output = io::Result<Digest>> {
        let result = (|| {
            self.callbacks += 1;
            let layout = stage.layout().unwrap();
            let authority = stage.plan().attempt_authority(OPERATION).unwrap();
            let mut journal = AttemptJournalGuard::open(layout, &authority).unwrap();
            let attempt = journal
                .reserve_planned_mutation(&stage.plan().digest())
                .unwrap();
            // The durable reservation exists before any integration-owned dispatch.
            let persisted: serde_json::Value =
                support::read_json(&journal.path(), 1024 * 1024).unwrap();
            assert_eq!(persisted["events"][0]["attempt"], attempt);
            // Native dispatch witness only; this test makes no remote call. Production
            // admission authenticates current context and retains signed bytes first.
            support::create_json_durable(
                &self.root.join("host-dispatch.json"),
                &json!({
                    "authority": authority.digest(), "attempt": attempt,
                    "request": Digest::from_bytes(&self.request)
                }),
            )
            .unwrap();
            if self.lose_reply {
                return Err(io::Error::other(
                    "fixture reply unavailable after host dispatch",
                ));
            }
            // Explicit fixture qualification of this host ingress only. Root's separate
            // paid-effect evidence is never turned into, or consumed as, a host receipt.
            let evidence =
                Digest::from_bytes(b"fixture-qualified host reply; not paid-effect settlement");
            journal
                .record_mutation(MutationReceiptRequest {
                    attempt,
                    request: Digest::from_bytes(&self.request).hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: evidence.hash().into(),
                })
                .unwrap();
            Ok(evidence)
        })();
        std::future::ready(result)
    }
}

fn setup() -> (PathBuf, ExecutionWorkflowRecord, ApplicationDriver) {
    let root = support::temp_root("ic-backup-application-ingress");
    fs::DirBuilder::new()
        .mode(0o700)
        .create(root.join("workflow"))
        .unwrap();
    support::create_json_durable(&root.join("original-request.json"), &original_request()).unwrap();
    // Root owns this evidence outside loaded participant snapshots. This test only
    // verifies that host accounting leaves the independently retained bytes alone.
    support::create_json_durable(
        &root.join("root-paid-effect.json"),
        &json!({"state": "pending", "original_limit": 1}),
    )
    .unwrap();
    let request = fs::read(root.join("original-request.json")).unwrap();
    let plan = original_plan(&Digest::from_bytes(&request));
    let mut allocation = serde_json::to_value(&plan).unwrap();
    allocation["graph"]["nodes"][0]["operation_sequence"] = json!(STAGE);
    allocation["operations"][0]["operation_sequence"] = json!(STAGE);
    allocation["operations"][0]["request"] =
        json!(Digest::from_bytes(b"original application ingress stage purpose").hash());
    let workflow = ExecutionWorkflowRecord::new(serde_json::from_value(allocation).unwrap());
    let driver = ApplicationDriver::new(root.clone(), plan, request);
    (root, workflow, driver)
}

#[test]
fn exact_application_ingress_settles_only_host_accounting_and_replays_without_callbacks() {
    let (root, workflow, mut driver) = setup();
    let paid = fs::read(root.join("root-paid-effect.json")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("workflow")).unwrap();
    let settled =
        ready::ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).unwrap();
    let binding = driver.binding.as_ref().unwrap().digest();
    let learned = Digest::from_bytes(b"fixture-qualified host reply; not paid-effect settlement");
    let retained = BTreeMap::from([(STAGE, RetainedStageEvidence::new(binding.clone(), learned))]);
    let stage = ExecutionStageGuard::open(&layout, &workflow.digest(), STAGE, &binding).unwrap();
    let journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &driver.plan.attempt_authority(OPERATION).unwrap(),
    )
    .unwrap();
    let journal_path = journal.path();
    let journal_bytes = fs::read(&journal_path).unwrap();
    let witness = fs::read(root.join("host-dispatch.json")).unwrap();
    assert_eq!(journal.record().unwrap().view().mutations_used, 1);
    drop(journal);
    drop(stage);
    drop(layout);
    // Reconstruct the adapter after releasing all original guards.
    let mut cold = ApplicationDriver::new(root.clone(), driver.plan, driver.request);
    let layout = BackupLayoutGuard::acquire(&root.join("workflow")).unwrap();
    assert_eq!(
        ready::ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            &mut cold
        ))
        .unwrap(),
        settled
    );
    assert_eq!(cold.callbacks, 0);
    assert_eq!(fs::read(journal_path).unwrap(), journal_bytes);
    assert_eq!(fs::read(root.join("host-dispatch.json")).unwrap(), witness);
    assert_eq!(fs::read(root.join("root-paid-effect.json")).unwrap(), paid);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lost_root_ingress_reply_preserves_original_pending_attempt_and_refuses_continuation() {
    let (root, workflow, mut driver) = setup();
    driver.lose_reply = true;
    let paid = fs::read(root.join("root-paid-effect.json")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("workflow")).unwrap();
    assert!(matches!(
        ready::ready(run_new_execution_workflow(&layout, &workflow, &mut driver)),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    let binding = driver.binding.as_ref().unwrap().digest();
    let (stage, progress) =
        ExecutionStageGuard::resume(&layout, &workflow.digest(), STAGE, &binding).unwrap();
    assert_eq!(progress.attempts.mutations_used, 1);
    assert_eq!(progress.attempts.mutations_remaining, 0);
    let authority = driver.plan.attempt_authority(OPERATION).unwrap();
    let journal = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
    assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
    let path = journal.path();
    let bytes = fs::read(&path).unwrap();
    let witness = fs::read(root.join("host-dispatch.json")).unwrap();
    drop(journal);
    drop(stage);
    drop(layout);
    let mut cold = ApplicationDriver::new(root.clone(), driver.plan, driver.request);
    let layout = BackupLayoutGuard::acquire(&root.join("workflow")).unwrap();
    // Neither an invented learned digest nor an empty retained set settles an
    // occupied pending stage. Recovery must use its original observation allowance.
    assert!(matches!(
        ready::ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            &mut cold
        )),
        Err(ExecutionWorkflowRunError::RetainedStageSetMismatch)
    ));
    let retained = BTreeMap::from([(
        STAGE,
        RetainedStageEvidence::new(binding, Digest::from_bytes(b"unqualified")),
    )]);
    assert!(matches!(
        ready::ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            &mut cold
        )),
        Err(ExecutionWorkflowRunError::Checkpoint(_))
    ));
    assert!(matches!(
        ready::ready(run_new_execution_workflow(&layout, &workflow, &mut cold)),
        Err(ExecutionWorkflowRunError::Workflow(_))
    ));
    assert_eq!(cold.callbacks, 0);
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert_eq!(fs::read(root.join("host-dispatch.json")).unwrap(), witness);
    assert_eq!(fs::read(root.join("root-paid-effect.json")).unwrap(), paid);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn substituted_original_envelope_refuses_before_stage_allocation_or_dispatch() {
    for field in [
        "network",
        "caller",
        "release",
        "receiver",
        "method",
        "mode",
        "arguments",
        "unit",
    ] {
        let (root, workflow, mut driver) = setup();
        let mut substituted = original_request();
        substituted[field] = json!("changed");
        fs::write(
            root.join("original-request.json"),
            serde_json::to_vec(&substituted).unwrap(),
        )
        .unwrap();
        let layout = BackupLayoutGuard::acquire(&root.join("workflow")).unwrap();
        assert!(matches!(
            ready::ready(run_new_execution_workflow(&layout, &workflow, &mut driver)),
            Err(ExecutionWorkflowRunError::Driver(error)) if error.kind() == io::ErrorKind::PermissionDenied
        ));
        assert_eq!(driver.callbacks, 1);
        assert!(!root.join("host-dispatch.json").exists());
        assert!(driver.binding.is_none());
        assert!(!root.join("workflow/execution-stage-7").exists());
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
