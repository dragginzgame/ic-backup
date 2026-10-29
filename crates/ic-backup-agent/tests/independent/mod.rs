//! An independent normal consumer; Testkit and application authority stay in this parent.

mod application;
mod build;
mod consumer;

use super::*;
use std::{
    collections::BTreeMap,
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_cold_capture_restore_acceptance_and_terminal_replay() {
    exercise(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_death_after_authenticated_load_preserves_pending_without_redispatch() {
    exercise(true).await;
}

async fn child(fixture: &Fixture, root: &Path, mode: &str) -> i32 {
    static COMMAND: AtomicU64 = AtomicU64::new(0);
    let binary = build::binary().clone();
    let sequence = COMMAND.fetch_add(1, Ordering::Relaxed);
    let output = |extension| {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(
                fixture
                    .root
                    .join(format!("consumer-{sequence}-{mode}.{extension}")),
            )
            .unwrap()
    };
    let stdout = output("stdout");
    let stderr = output("stderr");
    let root = root.to_owned();
    let mode = mode.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut process = ic_host_process::child::OwnedChild::spawn(
            Command::new(binary)
                .arg(mode)
                .arg(root)
                .stdout(Stdio::from(stdout))
                .stderr(Stdio::from(stderr)),
            ic_host_process::child::CleanupPolicy::KillAndWait,
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            if let Some(status) = process.try_wait().unwrap() {
                return status.code().unwrap_or(-1);
            }
            if Instant::now() >= deadline {
                process.terminate().unwrap();
                panic!("bounded consumer deadline; original evidence retained");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })
    .await
    .unwrap()
}

async fn grant(fixture: &mut Fixture, root: &Path, mode: &str) {
    // A fresh actual control/lifecycle observation is separately reserved before
    // its call. It grants no mutation or retry spending to the consumer.
    let payload = support::payload(Method::CanisterStatus, &fixture.target, None);
    let (plan, layout, evidence) = fixture.original(&payload.digest());
    let journal = support::retain(&layout, &plan, fixture.sequence);
    let before = fs::read(journal.path()).unwrap();
    let status = fixture
        .pic
        .canister_status(fixture.principal(), Some(fixture.caller()))
        .await
        .unwrap();
    assert_eq!(status.settings.controllers, vec![fixture.caller()]);
    assert_eq!(
        serde_json::to_value(status.status).unwrap(),
        serde_json::to_value(CanisterStatusType::Stopped).unwrap()
    );
    let module_hash = ic_host_artifacts::artifact::Sha256Digest::from_bytes(
        status.module_hash.as_deref().unwrap().try_into().unwrap(),
    )
    .to_string();
    assert_eq!(module_hash, fixture.release.hash());
    let config: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("config.json")).unwrap()).unwrap();
    let raw = serde_json::to_vec(&serde_json::json!({"context":config["context"],"target":fixture.target,"module_hash":module_hash,"controllers":status.settings.controllers.iter().map(ic_platform::principal::ExternalPrincipal::to_text).collect::<Vec<_>>(),"status":status.status})).unwrap();
    consumer::retain(&evidence.join("actual-status.json"), &raw).unwrap();
    consumer::retain(&root.join(format!("admission-{mode}.json")), &raw).unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), before);
}

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            result.extend(files(&entry.path()));
        } else {
            assert!(entry.file_type().unwrap().is_file());
            result.insert(entry.path(), fs::read(entry.path()).unwrap());
        }
    }
    result
}

async fn verify(fixture: &mut Fixture, root: &Path) -> serde_json::Value {
    let payload = support::payload(Method::TakeCanisterSnapshot, &fixture.target, None);
    let original = fs::read(root.join("capture/execution-stage-0/reply-0.candid")).unwrap();
    let original = IcSnapshotReply::decode(&payload, &original).unwrap();
    let original_request =
        IcSnapshotMetadataRequest::new(&fixture.target, original.snapshots()[0].id()).unwrap();
    let raw = fs::read(root.join("capture/execution-stage-1/reply-1.candid")).unwrap();
    let expected = IcSnapshotMetadataReply::decode(&original_request, &raw).unwrap();
    let (_, raw) = fixture
        .mutation(Method::TakeCanisterSnapshot, None, false)
        .await;
    let snapshot = IcSnapshotReply::decode(&payload, &raw).unwrap();
    let request =
        IcSnapshotMetadataRequest::new(&fixture.target, snapshot.snapshots()[0].id()).unwrap();
    let raw = fixture
        .read(IcSnapshotTransferReadPayload::Metadata(&request))
        .await;
    let actual = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let [actual_state, expected_state] = [actual.metadata(), expected.metadata()].map(|m| {
        ic_platform::candid::contract::encode_one((
            &m.certified_data,
            &m.globals,
            m.wasm_module_size,
            m.wasm_memory_size,
            m.stable_memory_size,
            &m.wasm_chunk_store,
            &m.global_timer,
            &m.on_low_wasm_memory_hook_status,
        ))
        .unwrap()
    });
    assert_eq!(actual_state, expected_state);
    let chunks = read_complete_regions(fixture, &actual).await;
    let capture: ic_backup::model::execution_workflow::ExecutionWorkflowRecord =
        serde_json::from_slice(&fs::read(root.join("capture/execution-workflow.json")).unwrap())
            .unwrap();
    let download = ic_backup::model::ic_snapshot_download::IcSnapshotDownloadPlan::new(
        &capture,
        2,
        &expected,
        1024 * 1024,
    )
    .unwrap();
    assert_eq!(chunks.len(), download.requests().len());
    let region_hashes: Vec<_> = chunks
        .iter()
        .map(|(_, bytes)| ArtifactChecksumRecord::from_bytes(bytes))
        .collect();
    for (index, (request, (kind, bytes))) in download.requests().iter().zip(chunks).enumerate() {
        let raw =
            fs::read(root.join(format!("capture/execution-stage-2/reply-{index}.candid"))).unwrap();
        let reply = IcSnapshotDataReply::decode(request, &raw).unwrap();
        assert_eq!(
            ic_platform::candid::contract::encode_one(request.kind()).unwrap(),
            ic_platform::candid::contract::encode_one(&kind).unwrap()
        );
        assert_eq!(reply.chunk(), bytes);
    }
    let restore: ic_backup::model::execution_workflow::ExecutionWorkflowRecord =
        serde_json::from_slice(&fs::read(root.join("restore/execution-workflow.json")).unwrap())
            .unwrap();
    let arguments = fs::read(root.join("restore/execution-stage-5/intent.candid")).unwrap();
    let args: ic_platform::management_canister_types::LoadCanisterSnapshotArgs =
        ic_platform::candid::decode_one(
            &arguments,
            ic_platform::candid::DecodeLimits {
                reply_bytes: 4096,
                decoding_quota: 100_000,
                skipping_quota: 100_000,
                type_entries: 128,
                header_bytes: 4096,
            },
        )
        .unwrap();
    serde_json::json!({"restore":restore.digest(),"source_metadata":expected.digest(),"destination":args.snapshot_id,"verification_metadata":actual.digest(),"region_hashes":region_hashes,"complete_state":true})
}

#[expect(
    clippy::too_many_lines,
    reason = "keep cold process boundaries, real application acceptance and lost-reply custody in one qualification trace"
)]
async fn exercise(lose_load: bool) {
    let _: fn() = consumer::entry;
    build::binary();
    let mut fixture = Fixture::new(Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(
        &[7; 32],
    )))
    .await;
    fixture.mutation(Method::StopCanister, None, false).await;
    let root = fixture.root.join("independent-consumer");
    std::os::unix::fs::DirBuilderExt::mode(&mut fs::DirBuilder::new(), 0o700)
        .create(&root)
        .unwrap();
    let config = serde_json::json!({"target":fixture.target,"endpoint":fixture.endpoint,"key":fixture.key,"context":{"network":ArtifactChecksumRecord::from_bytes(&fixture.key).hash(),"caller":fixture.caller().to_text(),"release":fixture.release.hash()}});
    consumer::retain(
        &root.join("config.json"),
        &serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    grant(&mut fixture, &root, "capture").await;
    assert_eq!(
        child(&fixture, &root, "capture").await,
        0,
        "{}",
        root.display()
    );
    fixture.mutation(Method::StartCanister, None, false).await;
    fixture
        .pic
        .update_call(
            fixture.principal(),
            ic_platform::principal::ExternalPrincipal::anonymous(),
            "write",
            99_u64.to_le_bytes().to_vec(),
        )
        .await
        .unwrap();
    fixture.mutation(Method::StopCanister, None, false).await;
    let mode = if lose_load { "lose-load" } else { "restore" };
    grant(&mut fixture, &root, mode).await;
    assert_eq!(
        child(&fixture, &root, mode).await,
        if lose_load { 75 } else { 0 },
        "{}",
        root.display()
    );
    if lose_load {
        // Actual full-state verification is an oracle only in this branch. It must
        // neither settle the child's pending load nor enter its acceptance channel.
        let oracle = verify(&mut fixture, &root).await;
        consumer::retain(
            &fixture.root.join("discarded-load-oracle.json"),
            &serde_json::to_vec(&oracle).unwrap(),
        )
        .unwrap();
        let before = files(&root);
        assert_eq!(child(&fixture, &root, "restore").await, 1);
        let mut after = files(&root);
        let mutex = root.join("restore/execution-stage-5/execution-settlement.json.lock");
        if !before.contains_key(&mutex) {
            assert_eq!(after.remove(&mutex), Some(vec![]));
        }
        assert_eq!(after, before);
        assert!(!root.join("restore/execution-stage-6").exists());
        let references: ic_backup::model::restore_references::RestoreReferencesRecord =
            serde_json::from_slice(
                &fs::read(root.join("capture/execution-stage-2/restore-references.json")).unwrap(),
            )
            .unwrap();
        let original: ic_backup::model::execution_workflow::ExecutionWorkflowRecord =
            serde_json::from_slice(
                &fs::read(root.join("restore/execution-workflow.json")).unwrap(),
            )
            .unwrap();
        assert!(references.entries().iter().any(|entry| entry.journal()
            == root.join("restore/execution-workflow.json")
            && entry.authority() == original.digest().hash()));
        let journal: ic_backup::model::attempt_journal::AttemptJournalRecord =
            serde_json::from_slice(
                &fs::read(root.join("restore/execution-stage-5/attempt-5.json")).unwrap(),
            )
            .unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert_eq!(journal.view().mutations_remaining, 0);
    } else {
        grant(&mut fixture, &root, "start").await;
        let before = files(&root);
        assert_eq!(child(&fixture, &root, "start").await, 1);
        assert_eq!(files(&root), before);
        assert!(!root.join("restore/execution-stage-6").exists());
        let acceptance = verify(&mut fixture, &root).await;
        consumer::retain(
            &root.join("acceptance.json"),
            &serde_json::to_vec(&acceptance).unwrap(),
        )
        .unwrap();
        assert_eq!(
            child(&fixture, &root, "start").await,
            0,
            "{}",
            root.display()
        );
        let restored = fixture
            .pic
            .query_call(
                fixture.principal(),
                ic_platform::principal::ExternalPrincipal::anonymous(),
                "read",
                vec![],
            )
            .await
            .unwrap();
        assert_eq!(restored, [42_u64.to_le_bytes(); 3].concat());
        let before = files(&root);
        assert_eq!(child(&fixture, &root, "replay").await, 0);
        assert_eq!(files(&root), before);
    }
    let executable =
        ic_host_fs::read::hash_file_no_follow(build::binary(), 512 * 1024 * 1024).unwrap();
    consumer::retain(&fixture.root.join("independent-qualification.json"), &serde_json::to_vec_pretty(&serde_json::json!({
        "scope":"candidate normal-dependency consumer with parent-owned isolated application authority",
        "consumer":build::binary(),"consumer_sha256":executable.sha256.to_string(),"target":fixture.target,"caller":fixture.caller().to_text(),
        "source_release":fixture.release,"lost_load":lose_load,"cold_reentry_without_dispatch":true,"application_acceptance":!lose_load,"terminal_replay":!lose_load
    })).unwrap()).unwrap();
    eprintln!(
        "Independent authenticated consumer evidence retained: {}",
        fixture.root.display()
    );
}
