//! Exact bounded binding and ancestor admission, with one layout lock at a time.

use super::{
    ArtifactChecksumRecord, BINDING_FILE, BTreeMap, BackupLayoutGuard, Entry,
    ExecutionStageBindingRecord, ExecutionWorkflowPersistenceError, ExecutionWorkflowRecord,
    MAX_EXECUTION_STAGE_BYTES, OperationPlanRecord, PathBuf, PersistenceError, fs,
    read_execution_settlement, read_json, read_operation_plan,
};

pub(super) fn read_binding(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    sequence: u64,
    expected: &ArtifactChecksumRecord,
) -> Result<(ExecutionStageBindingRecord, OperationPlanRecord), ExecutionWorkflowPersistenceError> {
    layout.check_root()?;
    let binding: ExecutionStageBindingRecord =
        read_json(&layout.root().join(BINDING_FILE), MAX_EXECUTION_STAGE_BYTES)?;
    super::super::json::check_json_size(&binding, MAX_EXECUTION_STAGE_BYTES)?;
    if binding.stage_sequence() != sequence || &binding.digest() != expected {
        return Err(ExecutionWorkflowPersistenceError::DigestMismatch);
    }
    let plan = read_operation_plan(layout, binding.plan())?;
    binding.validate(workflow, &plan)?;
    Ok((binding, plan))
}
pub(super) fn validate_predecessors(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    binding: &ExecutionStageBindingRecord,
) -> Result<(), ExecutionWorkflowPersistenceError> {
    // Traverse the original acyclic catalog iteratively, admitting each ancestor
    // once. Keep at most one row per catalog node, not one per dependency path.
    // Learned evidence may differ between edges; binding/settlement identity cannot.
    let mut expected: BTreeMap<_, _> = binding
        .predecessors()
        .iter()
        .map(|row| {
            (
                row.stage_sequence(),
                (row.binding().clone(), row.settlement().clone()),
            )
        })
        .collect();
    let mut pending: Vec<_> = expected.keys().copied().collect();
    while let Some(sequence) = pending.pop() {
        let (expected_binding, expected_settlement) = &expected[&sequence];
        let original = {
            // Release this layout and every journal lock before admitting another
            // ancestor, including an ancestor shared by multiple branches.
            let predecessor = acquire_stage(layout, sequence)?;
            let (original, _) = read_binding(&predecessor, workflow, sequence, expected_binding)?;
            read_execution_settlement(&predecessor, original.plan(), expected_settlement)?;
            original
        };
        for ancestor in original.predecessors() {
            match expected.entry(ancestor.stage_sequence()) {
                Entry::Vacant(entry) => {
                    pending.push(ancestor.stage_sequence());
                    entry.insert((ancestor.binding().clone(), ancestor.settlement().clone()));
                }
                Entry::Occupied(entry) => {
                    let (expected_binding, expected_settlement) = entry.get();
                    if expected_binding != ancestor.binding()
                        || expected_settlement != ancestor.settlement()
                    {
                        return Err(ExecutionWorkflowPersistenceError::DigestMismatch);
                    }
                }
            }
        }
    }
    Ok(())
}
pub(super) fn stage_path(layout: &BackupLayoutGuard, sequence: u64) -> PathBuf {
    layout.root().join(format!("execution-stage-{sequence}"))
}
pub(super) fn acquire_stage(
    layout: &BackupLayoutGuard,
    sequence: u64,
) -> Result<BackupLayoutGuard, ExecutionWorkflowPersistenceError> {
    layout.check_root()?;
    let path = stage_path(layout, sequence);
    let metadata = fs::symlink_metadata(&path).map_err(PersistenceError::from)?;
    if !metadata.is_dir() {
        return Err(ExecutionWorkflowPersistenceError::UnsafeStage);
    }
    let stage = BackupLayoutGuard::acquire(&path)?;
    // Unlike an explicitly operator-selected root, a derived stage symlink is
    // never adopted. Noncooperating namespace stability remains caller-owned.
    if stage.root() != path {
        return Err(ExecutionWorkflowPersistenceError::UnsafeStage);
    }
    layout.check_root()?;
    Ok(stage)
}
// Both reopening and effect-boundary admission validate the same exact records
// and complete ancestors. This does not cache admission across an effect.
pub(super) fn read_admitted_binding(
    workflow_layout: &BackupLayoutGuard,
    stage_layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    sequence: u64,
    expected: &ArtifactChecksumRecord,
) -> Result<(ExecutionStageBindingRecord, OperationPlanRecord), ExecutionWorkflowPersistenceError> {
    let original = read_binding(stage_layout, workflow, sequence, expected)?;
    validate_predecessors(workflow_layout, workflow, &original.0)?;
    Ok(original)
}
