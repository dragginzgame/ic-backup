//! Complete operator rejection cases through the independently built normal consumer.
use super::*;

pub(super) async fn inspect(fixture: &Fixture, root: &Path, capture: bool) -> serde_json::Value {
    let mode = if capture {
        Mode::InspectCapture
    } else {
        Mode::InspectRestore
    };
    let before = files(root);
    let result = child(fixture, root, mode).await;
    result.assert_status(0);
    assert_eq!(files(root), before);
    serde_json::from_slice(&fs::read(result.stdout).unwrap()).unwrap()
}

// Match opaque identities rather than treating allocation order as an index.
pub(super) fn stage_row(progress: &serde_json::Value, stage: Stage) -> &serde_json::Value {
    progress["stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["stage_sequence"].as_u64() == Some(stage.sequence()))
        .expect("declared stage is present in progress")
}

pub(super) fn operation_row(progress: &serde_json::Value, sequence: u64) -> &serde_json::Value {
    progress["operations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["operation_sequence"].as_u64() == Some(sequence))
        .expect("declared operation is present in progress")
}

#[test]
fn inspection_matches_original_sequences_in_reordered_progress() {
    let progress = serde_json::json!({"stages": [
        {"stage_sequence": 6, "state": {"state": "unprepared"}},
        {"stage_sequence": 5, "state": {"state": "prepared", "progress": {"operations": [
            {"operation_sequence": 21, "attempts": {"pending_mutation": null}},
            {"operation_sequence": 8, "attempts": {"pending_mutation": 1}}
        ]}}}
    ]});
    assert_eq!(
        stage_row(&progress, Stage::Start)["state"]["state"],
        "unprepared"
    );
    let load = &stage_row(&progress, Stage::Load)["state"]["progress"];
    assert_eq!(operation_row(load, 8)["attempts"]["pending_mutation"], 1);
    assert!(operation_row(load, 21)["attempts"]["pending_mutation"].is_null());
}

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_operator_retention_and_partial_output_failures_preserve_originals() {
    for mode in [
        Mode::CaptureFailIngress,
        Mode::CaptureFailRetention,
        Mode::CaptureFailPartial,
    ] {
        failure(mode).await;
    }
}

pub(super) async fn snapshot_count(fixture: &mut Fixture) -> usize {
    let payload = support::payload(Method::ListCanisterSnapshots, &fixture.target, None);
    let OriginalCall {
        plan,
        layout,
        sequence,
    } = fixture.original(&payload.digest());
    let evidence = layout.root().to_path_buf();
    let _guard = support::retain(&layout, &plan, sequence);
    let snapshots = fixture
        .pic
        .list_canister_snapshots(fixture.principal(), Some(fixture.caller()))
        .await
        .unwrap();
    consumer::retain(
        &evidence.join("fixture-inventory-count.json"),
        &serde_json::to_vec(&snapshots.len()).unwrap(),
    )
    .unwrap();
    snapshots.len()
}

async fn failure(mode: Mode) {
    build::binary();
    let mut fixture = Fixture::new(Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(
        &[7; 32],
    )))
    .await;
    fixture.mutation(Method::StopCanister, None, false).await;
    let root = fixture.root.join("operator-failure");
    std::os::unix::fs::DirBuilderExt::mode(&mut fs::DirBuilder::new(), 0o700)
        .create(&root)
        .unwrap();
    let config = connection_config(&fixture);
    write_config(&root, &config);
    grant(&mut fixture, &root, mode).await;
    assert_eq!(snapshot_count(&mut fixture).await, 0);
    child(&fixture, &root, mode).await.assert_status(1);
    let progress = inspect(&fixture, &root, true).await;
    let stage = if mode == Mode::CaptureFailPartial {
        Stage::Download
    } else {
        Stage::Capture
    };
    let row = &stage_row(&progress, stage)["state"]["progress"];
    assert_eq!(
        row["attempts"]["mutations_used"],
        if stage == Stage::Download { 2 } else { 1 }
    );
    let pending = &operation_row(row, u64::from(stage == Stage::Download))["attempts"];
    assert_eq!(
        row["applied_operations"],
        usize::from(stage == Stage::Download)
    );
    assert_eq!(pending["pending_mutation"], 1);
    assert_eq!(pending["mutations_remaining"], 0);
    let before = files(&root);
    child(&fixture, &root, Mode::CaptureContinue)
        .await
        .assert_status(1);
    child(&fixture, &root, Mode::Capture).await.assert_status(1);
    assert_eq!(files(&root), before);
    assert_eq!(
        snapshot_count(&mut fixture).await,
        usize::from(mode != Mode::CaptureFailIngress)
    );
    if stage == Stage::Download {
        partial(&root);
    }
    assert!(
        !Stage::Download
            .path(&root)
            .join("download-manifest.json")
            .exists()
    );
    assert!(!Stage::Allocation.path(&root).exists());
    eprintln!(
        "Independent operator failure evidence retained: {}",
        fixture.root.display()
    );
}

fn partial(root: &Path) {
    let stage = Stage::Download.path(root);
    let record: ic_backup::model::download_journal::DownloadJournalRecord =
        serde_json::from_slice(&fs::read(stage.join("download-journal.json")).unwrap()).unwrap();
    let artifact = &record.artifacts()[0];
    assert_eq!(
        artifact.state(),
        ic_backup::model::download_journal::ArtifactStateRecord::Created
    );
    assert!(stage.join(artifact.staging_path()).is_dir());
    assert!(!stage.join(artifact.artifact_path()).exists());
    assert!(
        files(&stage.join(artifact.staging_path()))
            .values()
            .any(|bytes| !bytes.is_empty())
    );
}
