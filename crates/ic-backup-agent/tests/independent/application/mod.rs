//! Actual opaque updates through an independently built normal-dependency consumer.
use super::*;
use std::os::unix::fs::DirBuilderExt;

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_update_and_completed_replay_are_effect_free() {
    exercise("application-init").await;
}
#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_process_death_retains_original_and_separately_observes() {
    exercise("application-die").await;
}
#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_reply_bound_preserves_pending_without_resend() {
    exercise("application-small").await;
}
#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_reply_retention_failure_preserves_pending() {
    exercise("application-refuse-reply").await;
}

async fn exercise(mode: &str) {
    build::binary();
    let identity = Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(&[7; 32]));
    let mut fixture = Fixture::with_wasm(identity, include_str!("state.wat")).await;
    let root = fixture.root.join("application-consumer");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let config = consumer::ApplicationConfig {
        connection: connection_config(&fixture),
        reply_limit: if mode == "application-small" { 8 } else { 16 },
    };
    write_config(&root, &config);
    // Independently account this fixture's fresh actual controller/release probe.
    let payload = support::payload(Method::CanisterStatus, &fixture.target, None);
    let (plan, layout, _) = fixture.original(&payload.digest());
    let _guard = support::retain(&layout, &plan, fixture.sequence);
    let status = fixture
        .pic
        .canister_status(fixture.principal(), Some(fixture.caller()))
        .await
        .unwrap();
    assert_eq!(status.settings.controllers, vec![fixture.caller()]);
    assert_eq!(
        ic_host_artifacts::artifact::Sha256Digest::from_bytes(
            status.module_hash.as_deref().unwrap().try_into().unwrap()
        )
        .to_string(),
        fixture.release.hash()
    );
    consumer::retain(&root.join("admission.json"), &serde_json::to_vec(&serde_json::json!({"context":config.connection.context,"target":fixture.target,"controllers":status.settings.controllers})).unwrap()).unwrap();
    assert_eq!(
        child(&fixture, &root, mode).await,
        if mode == "application-die" {
            75
        } else {
            i32::from(mode != "application-init")
        },
        "{}",
        root.display()
    );
    assert_eq!(
        state(&fixture).await,
        [1_u64.to_le_bytes(), 73_u64.to_le_bytes()].concat()
    );
    let journal = root.join("execution-stage-0/attempt-0.json");
    let original = fs::read(&journal).unwrap();
    if mode != "application-init" {
        let record: ic_backup::model::attempt_journal::AttemptJournalRecord =
            serde_json::from_slice(&original).unwrap();
        assert_eq!(record.view().pending_mutation, Some(1));
        assert_eq!(record.view().mutations_used, 1);
        assert_eq!(child(&fixture, &root, "application-resume").await, 1);
        assert_eq!(child(&fixture, &root, "application-init").await, 1);
        assert_eq!(fs::read(&journal).unwrap(), original);
        assert_eq!(
            state(&fixture).await,
            [1_u64.to_le_bytes(), 73_u64.to_le_bytes()].concat()
        );
        if mode != "application-die" {
            assert!(!root.join("execution-stage-0/mutation-reply.bin").exists());
            return;
        }
        assert_eq!(
            fs::read(root.join("execution-stage-0/mutation-reply.bin")).unwrap(),
            state(&fixture).await
        );
        assert_eq!(child(&fixture, &root, "application-observe").await, 0);
        let record: ic_backup::model::attempt_journal::AttemptJournalRecord =
            serde_json::from_slice(&fs::read(&journal).unwrap()).unwrap();
        assert!(record.view().applied);
        assert_eq!(
            (
                record.view().mutations_used,
                record.view().observations_used
            ),
            (1, 1)
        );
        assert!(
            root.join("execution-stage-0/observation-envelope.cbor")
                .is_file()
        );
    }
    let before = files(&root);
    assert_eq!(child(&fixture, &root, "application-replay").await, 0);
    assert_eq!(child(&fixture, &root, "application-replay").await, 0);
    assert_eq!(files(&root), before);
    assert_eq!(
        state(&fixture).await,
        [1_u64.to_le_bytes(), 73_u64.to_le_bytes()].concat()
    );
    eprintln!(
        "Independent opaque application evidence retained: {}",
        root.display()
    );
}
async fn state(fixture: &Fixture) -> Vec<u8> {
    fixture
        .pic
        .query_call(fixture.principal(), fixture.caller(), "read", vec![])
        .await
        .unwrap()
}
