//! An independent normal consumer; Testkit and application authority stay in this parent.

mod application;
mod build;
mod consumer;
mod operator;
mod restore_failures;
mod terminal;

use super::*;
use consumer::{Mode, Stage};
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
    exercise(Mode::Restore).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_death_after_authenticated_load_preserves_pending_without_redispatch() {
    exercise(Mode::LoseLoad).await;
}

struct CommandResult {
    status: i32,
    stdout: PathBuf,
    stderr: PathBuf,
}

impl CommandResult {
    fn assert_status(&self, expected: i32) {
        assert_eq!(
            self.status,
            expected,
            "stdout: {}; stderr: {}",
            self.stdout.display(),
            self.stderr.display()
        );
    }
}

async fn child(fixture: &Fixture, root: &Path, mode: Mode) -> CommandResult {
    static COMMAND: AtomicU64 = AtomicU64::new(0);
    let binary = build::binary().clone();
    let sequence = COMMAND.fetch_add(1, Ordering::Relaxed);
    let path = |extension| {
        fixture
            .root
            .join(format!("consumer-{sequence}-{mode}.{extension}"))
    };
    let stdout_path = path("stdout");
    let stderr_path = path("stderr");
    let output = |path: &Path| {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .unwrap()
    };
    let stdout = output(&stdout_path);
    let stderr = output(&stderr_path);
    let root = root.to_owned();
    let status = tokio::task::spawn_blocking(move || {
        let mut process = ic_host_process::child::OwnedChild::spawn(
            Command::new(binary)
                .arg(mode.to_string())
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
    .unwrap();
    CommandResult {
        status,
        stdout: stdout_path,
        stderr: stderr_path,
    }
}

async fn grant(fixture: &mut Fixture, root: &Path, mode: Mode) {
    // A fresh actual control/lifecycle observation is separately reserved before
    // its call. It grants no mutation or retry spending to the consumer.
    let payload = support::payload(Method::CanisterStatus, &fixture.target, None);
    let OriginalCall {
        plan,
        layout,
        sequence,
    } = fixture.original(&payload.digest());
    let evidence = layout.root().to_path_buf();
    let journal = support::retain(&layout, &plan, sequence);
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
    let config = consumer::Config::read(root).unwrap();
    let raw = serde_json::to_vec(&serde_json::json!({"context":config.context,"target":fixture.target,"module_hash":module_hash,"controllers":status.settings.controllers.iter().map(ic_platform::principal::ExternalPrincipal::to_text).collect::<Vec<_>>(),"status":status.status})).unwrap();
    consumer::retain(&evidence.join("actual-status.json"), &raw).unwrap();
    consumer::retain(&root.join(format!("admission-{mode}.json")), &raw).unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), before);
}

fn connection_config(fixture: &Fixture) -> consumer::Config {
    use ic_backup::model::operation_plan::{PlanContextRecord, PlanContextRequest};
    consumer::Config {
        context: PlanContextRecord::new(&PlanContextRequest {
            network: ArtifactChecksumRecord::from_bytes(&fixture.key)
                .hash()
                .into(),
            caller: fixture.caller().to_text(),
            release: fixture.release.hash().into(),
        })
        .unwrap(),
        target: fixture.target.clone(),
        endpoint: fixture.endpoint.clone(),
        key: fixture.key.clone(),
        identity_seed: [7; 32],
    }
}

fn write_config(root: &Path, config: &impl serde::Serialize) {
    // Retain the original JSON object's canonical key ordering and create-only custody.
    let value = serde_json::to_value(config).unwrap();
    consumer::retain(
        &root.join("config.json"),
        &serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
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
    let original = fs::read(
        Stage::Capture
            .path(root)
            .join(format!("reply-{}.candid", Stage::Capture.sequence())),
    )
    .unwrap();
    let original = IcSnapshotReply::decode(&payload, &original).unwrap();
    let original_request =
        IcSnapshotMetadataRequest::new(&fixture.target, original.snapshots()[0].id()).unwrap();
    let raw = fs::read(
        Stage::Metadata
            .path(root)
            .join(format!("reply-{}.candid", Stage::Metadata.sequence())),
    )
    .unwrap();
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
        Stage::Download.sequence(),
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
        let raw = fs::read(
            Stage::Download
                .path(root)
                .join(format!("reply-{index}.candid")),
        )
        .unwrap();
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
    let arguments = fs::read(Stage::Load.path(root).join("intent.candid")).unwrap();
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

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_cold_reentry_after_each_restore_stage_preserves_originals() {
    exercise(Mode::RestoreStep).await;
}

async fn captured(identity_seed: [u8; 32]) -> (Fixture, PathBuf) {
    let _: fn() = consumer::main;
    build::binary();
    let mut fixture = Fixture::new(Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(
        &identity_seed,
    )))
    .await;
    fixture.mutation(Method::StopCanister, None, false).await;
    let root = fixture.root.join("independent-consumer");
    std::os::unix::fs::DirBuilderExt::mode(&mut fs::DirBuilder::new(), 0o700)
        .create(&root)
        .unwrap();
    let mut config = connection_config(&fixture);
    config.identity_seed = identity_seed;
    write_config(&root, &config);
    grant(&mut fixture, &root, Mode::Capture).await;
    child(&fixture, &root, Mode::Capture).await.assert_status(0);
    operator::inspect(&fixture, &root, true).await;
    let before = files(&root);
    child(&fixture, &root, Mode::CaptureContinue)
        .await
        .assert_status(0);
    assert_eq!(files(&root), before);
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
    (fixture, root)
}

async fn exercise(mode: Mode) {
    let lose_load = mode == Mode::LoseLoad;
    let (mut fixture, root) = captured(if mode == Mode::Restore {
        [9; 32]
    } else {
        [7; 32]
    })
    .await;
    grant(&mut fixture, &root, mode).await;
    let steps = if mode == Mode::RestoreStep { 3 } else { 1 };
    for step in 0..steps {
        let original = files(&root);
        child(&fixture, &root, mode)
            .await
            .assert_status(if lose_load { 75 } else { 0 });
        if mode == Mode::RestoreStep {
            for (path, bytes) in original {
                assert_eq!(fs::read(path).unwrap(), bytes);
            }
            let progress = operator::inspect(&fixture, &root, false).await;
            for (index, stage) in Stage::RESTORE.into_iter().enumerate() {
                assert_eq!(
                    operator::stage_row(&progress, stage)["state"]["state"],
                    if index <= step {
                        "prepared"
                    } else {
                        "unprepared"
                    }
                );
            }
        }
    }
    let progress = operator::inspect(&fixture, &root, false).await;
    assert_eq!(
        operator::stage_row(&progress, Stage::Start)["state"]["state"],
        "unprepared"
    );
    let load_progress = &operator::stage_row(&progress, Stage::Load)["state"]["progress"];
    let load = operator::operation_row(load_progress, Stage::Load.sequence());
    assert_eq!(
        load["attempts"]["pending_mutation"],
        if lose_load {
            serde_json::json!(1)
        } else {
            serde_json::Value::Null
        }
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
        child(&fixture, &root, Mode::Restore).await.assert_status(1);
        assert_eq!(files(&root), before);
        assert!(!Stage::Start.path(&root).exists());
        let references: ic_backup::model::restore_references::RestoreReferencesRecord =
            serde_json::from_slice(
                &fs::read(Stage::Download.path(&root).join("restore-references.json")).unwrap(),
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
                &fs::read(
                    Stage::Load
                        .path(&root)
                        .join(format!("attempt-{}.json", Stage::Load.sequence())),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert_eq!(journal.view().mutations_remaining, 0);
    } else {
        terminal::complete(&mut fixture, &root).await;
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

    fixture.close().await;
}
