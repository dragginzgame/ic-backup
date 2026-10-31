//! Complete operator rejection cases through the independently built normal consumer.
use super::*;

pub(super) async fn inspect(fixture: &Fixture, root: &Path, capture: bool) -> serde_json::Value {
    let mode = if capture {
        "inspect-capture"
    } else {
        "inspect-restore"
    };
    let before = files(root);
    assert_eq!(child(fixture, root, mode).await, 0, "{}", root.display());
    assert_eq!(files(root), before);
    let outputs: Vec<_> = fs::read_dir(&fixture.root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(&format!("-{mode}.stdout"))
        })
        .collect();
    assert_eq!(outputs.len(), 1);
    serde_json::from_slice(&fs::read(&outputs[0]).unwrap()).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_operator_retention_and_partial_output_failures_preserve_originals() {
    for mode in [
        "capture-fail-ingress",
        "capture-fail-retention",
        "capture-fail-partial",
    ] {
        failure(mode).await;
    }
}

async fn snapshot_count(fixture: &mut Fixture) -> usize {
    let payload = support::payload(Method::ListCanisterSnapshots, &fixture.target, None);
    let (plan, layout, evidence) = fixture.original(&payload.digest());
    let _guard = support::retain(&layout, &plan, fixture.sequence);
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

async fn failure(mode: &str) {
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
    assert_eq!(child(&fixture, &root, mode).await, 1, "{}", root.display());
    let progress = inspect(&fixture, &root, true).await;
    let stage = if mode == "capture-fail-partial" { 2 } else { 0 };
    let row = &progress["stages"][stage]["state"]["progress"];
    assert_eq!(
        row["attempts"]["mutations_used"],
        if stage == 2 { 2 } else { 1 }
    );
    let pending = &row["operations"][usize::from(stage == 2)]["attempts"];
    assert_eq!(row["applied_operations"], usize::from(stage == 2));
    assert_eq!(pending["pending_mutation"], 1);
    assert_eq!(pending["mutations_remaining"], 0);
    let before = files(&root);
    assert_eq!(child(&fixture, &root, "capture-continue").await, 1);
    assert_eq!(child(&fixture, &root, "capture").await, 1);
    assert_eq!(files(&root), before);
    assert_eq!(
        snapshot_count(&mut fixture).await,
        usize::from(mode != "capture-fail-ingress")
    );
    if stage == 2 {
        partial(&root);
    }
    assert!(
        !root
            .join("capture/execution-stage-2/download-manifest.json")
            .exists()
    );
    assert!(!root.join("restore/execution-stage-3").exists());
    eprintln!(
        "Independent operator failure evidence retained: {}",
        fixture.root.display()
    );
}

fn partial(root: &Path) {
    let stage = root.join("capture/execution-stage-2");
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
