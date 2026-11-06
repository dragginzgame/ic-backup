//! Lost replies in the canonical cold-process journey; oracle bytes grant no recovery.

use super::*;

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_lost_allocation_retains_pending_without_upload_or_redispatch() {
    lost_reply(Mode::LoseAllocation).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_lost_first_upload_retains_pending_without_applied_prefix() {
    lost_reply(Mode::LoseUploadFirst).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_lost_second_upload_retains_applied_prefix_and_source() {
    lost_reply(Mode::LoseUploadSecond).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_lost_start_retains_pending_without_restart_or_release() {
    lost_reply(Mode::LoseStart).await;
}

async fn lost_reply(mode: Mode) {
    let (mut fixture, root) = captured([7; 32]).await;
    let (stage, sequence) = mode.lost_reply().unwrap();
    if stage == Stage::Start {
        grant(&mut fixture, &root, Mode::Restore).await;
        child(&fixture, &root, Mode::Restore).await.assert_status(0);
        let acceptance = verify(&mut fixture, &root).await;
        consumer::retain(
            &root.join("acceptance.json"),
            &serde_json::to_vec(&acceptance).unwrap(),
        )
        .unwrap();
    }
    grant(&mut fixture, &root, mode).await;
    child(&fixture, &root, mode).await.assert_status(75);
    let progress = operator::inspect(&fixture, &root, false).await;
    let row = &operator::stage_row(&progress, stage)["state"]["progress"];
    let attempts = &operator::operation_row(row, sequence)["attempts"];
    assert_eq!(attempts["pending_mutation"], 1);
    assert_eq!(attempts["mutations_used"], 1);
    assert_eq!(attempts["mutations_remaining"], 0);
    assert_eq!(
        row["applied_operations"],
        usize::from(stage == Stage::Upload && sequence != 0)
    );
    assert!(stage.path(&root).join("discarded-oracle.candid").is_file());
    for successor in Stage::RESTORE
        .into_iter()
        .skip_while(|candidate| *candidate != stage)
        .skip(1)
    {
        assert_eq!(
            operator::stage_row(&progress, successor)["state"]["state"],
            "unprepared"
        );
        assert!(!successor.path(&root).exists());
    }
    let original: ic_backup::model::execution_workflow::ExecutionWorkflowRecord =
        serde_json::from_slice(&fs::read(root.join("restore/execution-workflow.json")).unwrap())
            .unwrap();
    let layout =
        ic_backup::ops::persistence::BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    assert!(
        ic_backup::ops::persistence::read_complete_execution_workflow(&layout, &original.digest())
            .is_err()
    );
    drop(layout);
    let snapshots = operator::snapshot_count(&mut fixture).await;
    let before = files(&root);
    for retry in [Mode::Restore, Mode::Start, Mode::Replay] {
        child(&fixture, &root, retry).await.assert_status(1);
        assert_eq!(files(&root), before);
    }
    assert_eq!(operator::snapshot_count(&mut fixture).await, snapshots);
    let references: ic_backup::model::restore_references::RestoreReferencesRecord =
        serde_json::from_slice(
            &fs::read(Stage::Download.path(&root).join("restore-references.json")).unwrap(),
        )
        .unwrap();
    assert!(
        references
            .entries()
            .iter()
            .any(
                |reference| reference.journal() == root.join("restore/execution-workflow.json")
                    && reference.authority() == original.digest().hash()
            )
    );
    eprintln!(
        "Independent lost-reply evidence retained: {}",
        fixture.root.display()
    );

    fixture.close().await;
}
