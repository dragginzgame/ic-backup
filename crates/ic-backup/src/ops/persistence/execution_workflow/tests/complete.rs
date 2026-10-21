//! Complete local stage checkpoint replay for an original workflow allocation.

use super::*;

#[test]
fn every_original_stage_is_required_and_exact_checkpoint_rows_are_replayed() {
    let (root, layout, workflow) = prepare();
    let first = first_stage(&layout, &workflow);
    let first_row = settle_first(&layout, &workflow, &first);
    let first_bytes = fs::read(root.join("execution-stage-0/attempt-42.json")).unwrap();

    // A settled prefix is never a complete original workflow.
    assert!(read_complete_execution_workflow(&layout, &workflow.digest()).is_err());
    assert!(!root.join("execution-stage-7").exists());

    let plan = child(7);
    let binding =
        ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![first_row.clone()]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap();
    let mut journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(42).unwrap(),
    )
    .unwrap();
    let attempt = journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: stage.plan().operation(42).unwrap().request().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: "56".repeat(32),
        })
        .unwrap();
    drop(journal);
    let second_row = stage
        .reconcile_checkpoint(ArtifactChecksumRecord::from_bytes(
            b"second learned evidence",
        ))
        .unwrap();
    drop(stage);

    let view = read_complete_execution_workflow(&layout, &workflow.digest()).unwrap();
    {
        let lock = JournalLock::acquire(&root.join(WORKFLOW_FILE)).unwrap();
        assert!(matches!(
            read_complete_execution_workflow(&layout, &workflow.digest()),
            Err(ExecutionStageCheckpointError::Stage(
                ExecutionWorkflowPersistenceError::Lock(JournalLockError::Locked { .. })
            ))
        ));
        assert_eq!(
            read_complete_execution_workflow_locked(&layout, &lock, &workflow.digest()).unwrap(),
            view
        );
        assert!(JournalLock::acquire(&root.join(WORKFLOW_FILE)).is_err());
    }
    assert_eq!(view.workflow(), &workflow.digest());
    assert_eq!(view.stages().len(), 2);
    for (actual, expected) in view.stages().iter().zip([first_row, second_row]) {
        assert_eq!(actual.stage_sequence(), expected.stage_sequence());
        assert_eq!(actual.binding(), expected.binding());
        assert_eq!(actual.settlement(), expected.settlement());
    }
    assert_eq!(
        fs::read(root.join("execution-stage-0/attempt-42.json")).unwrap(),
        first_bytes
    );

    let binding_path = root.join("execution-stage-7/stage-binding.json");
    let original_binding = fs::read(&binding_path).unwrap();
    let mut changed_binding: serde_json::Value = serde_json::from_slice(&original_binding).unwrap();
    changed_binding["predecessors"][0]["settlement"] =
        serde_json::to_value(ArtifactChecksumRecord::from_bytes(b"other settlement")).unwrap();
    let changed_binding = serde_json::to_vec_pretty(&changed_binding).unwrap();
    fs::write(&binding_path, &changed_binding).unwrap();
    assert!(read_complete_execution_workflow(&layout, &workflow.digest()).is_err());
    {
        let lock = JournalLock::acquire(&root.join(WORKFLOW_FILE)).unwrap();
        assert!(
            read_complete_execution_workflow_locked(&layout, &lock, &workflow.digest()).is_err()
        );
    }
    assert_eq!(fs::read(&binding_path).unwrap(), changed_binding);
    fs::write(&binding_path, original_binding).unwrap();

    // Missing or changed retained checkpoint evidence never repairs or republishes.
    let path = root.join("execution-stage-7/execution-settlement.json");
    let original = fs::read(&path).unwrap();
    fs::rename(&path, root.join("retained-checkpoint")).unwrap();
    assert!(read_complete_execution_workflow(&layout, &workflow.digest()).is_err());
    assert!(!path.exists());
    fs::rename(root.join("retained-checkpoint"), &path).unwrap();
    let journal_path = root.join("execution-stage-7/attempt-42.json");
    let mut changed: serde_json::Value =
        serde_json::from_slice(&fs::read(&journal_path).unwrap()).unwrap();
    changed["events"][1]["evidence"] = serde_json::json!("ab".repeat(32));
    let changed = serde_json::to_vec_pretty(&changed).unwrap();
    fs::write(&journal_path, &changed).unwrap();
    assert!(read_complete_execution_workflow(&layout, &workflow.digest()).is_err());
    assert_eq!(fs::read(&journal_path).unwrap(), changed);
    assert_eq!(fs::read(&path).unwrap(), original);

    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn held_workflow_guard_requires_exact_original_and_bounded_regular_bytes() {
    let (root, layout, workflow) = prepare();
    let path = root.join(WORKFLOW_FILE);
    let original = fs::read(&path).unwrap();
    let lock = JournalLock::acquire(&path).unwrap();
    let wrong = JournalLock::acquire(&root.join("another-journal.json")).unwrap();
    for (guard, digest) in [
        (&wrong, workflow.digest()),
        (
            &lock,
            ArtifactChecksumRecord::from_bytes(b"another original"),
        ),
    ] {
        assert!(matches!(
            read_complete_execution_workflow_locked(&layout, guard, &digest),
            Err(ExecutionStageCheckpointError::Stage(
                ExecutionWorkflowPersistenceError::DigestMismatch
            ))
        ));
        assert_eq!(fs::read(&path).unwrap(), original);
    }
    let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
    changed["unknown"] = serde_json::json!(true);
    fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(matches!(
        read_complete_execution_workflow_locked(&layout, &lock, &workflow.digest()),
        Err(ExecutionStageCheckpointError::Stage(
            ExecutionWorkflowPersistenceError::Persistence(PersistenceError::Json(_))
        ))
    ));
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_EXECUTION_WORKFLOW_BYTES + 1)
        .unwrap();
    assert!(matches!(
        read_complete_execution_workflow_locked(&layout, &lock, &workflow.digest()),
        Err(ExecutionStageCheckpointError::Stage(
            ExecutionWorkflowPersistenceError::Persistence(PersistenceError::RecordTooLarge { .. })
        ))
    ));
    fs::remove_file(&path).unwrap();
    fs::write(root.join("retained-original"), &original).unwrap();
    symlink(root.join("retained-original"), &path).unwrap();
    assert!(read_complete_execution_workflow_locked(&layout, &lock, &workflow.digest()).is_err());
    assert_eq!(fs::read(root.join("retained-original")).unwrap(), original);
    drop(wrong);
    drop(lock);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
