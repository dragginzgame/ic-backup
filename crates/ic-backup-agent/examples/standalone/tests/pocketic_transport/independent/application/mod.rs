//! Actual opaque updates through an independently built normal-dependency consumer.
use super::*;
use std::os::unix::fs::DirBuilderExt;

#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_update_and_completed_replay_are_effect_free() {
    exercise(Mode::ApplicationInit).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_process_death_retains_original_and_separately_observes() {
    exercise(Mode::ApplicationDie).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_reply_bound_preserves_pending_without_resend() {
    exercise(Mode::ApplicationSmall).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn normal_consumer_application_reply_retention_failure_preserves_pending() {
    exercise(Mode::ApplicationRefuseReply).await;
}

async fn exercise(mode: Mode) {
    build::binary();
    let identity = Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(&[7; 32]));
    let mut fixture = Fixture::with_wasm(identity, include_str!("state.wat")).await;
    let root = fixture.root.join("application-consumer");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let config = consumer::ApplicationConfig {
        connection: connection_config(&fixture),
        reply_limit: if mode == Mode::ApplicationSmall {
            8
        } else {
            16
        },
    };
    write_config(&root, &config);
    // Independently account this fixture's fresh actual controller/release probe.
    let payload = support::payload(Method::CanisterStatus, &fixture.target, None);
    let OriginalCall {
        plan,
        layout,
        sequence,
    } = fixture.original(&payload.digest());
    let _guard = support::retain(&layout, &plan, sequence);
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
    child(&fixture, &root, mode)
        .await
        .assert_status(if mode == Mode::ApplicationDie {
            75
        } else {
            i32::from(mode != Mode::ApplicationInit)
        });
    assert_once_applied(&fixture).await;
    let journal = root.join("execution-stage-0/attempt-0.json");
    let original = fs::read(&journal).unwrap();
    if mode != Mode::ApplicationInit {
        let record: ic_backup::model::attempt_journal::AttemptJournalRecord =
            serde_json::from_slice(&original).unwrap();
        assert_eq!(record.view().pending_mutation, Some(1));
        assert_eq!(record.view().mutations_used, 1);
        child(&fixture, &root, Mode::ApplicationResume)
            .await
            .assert_status(1);
        child(&fixture, &root, Mode::ApplicationInit)
            .await
            .assert_status(1);
        assert_eq!(fs::read(&journal).unwrap(), original);
        assert_once_applied(&fixture).await;
        if mode != Mode::ApplicationDie {
            assert!(!root.join("execution-stage-0/mutation-reply.bin").exists());
            return;
        }
        assert_eq!(
            fs::read(root.join("execution-stage-0/mutation-reply.bin")).unwrap(),
            state(&fixture).await
        );
        child(&fixture, &root, Mode::ApplicationObserve)
            .await
            .assert_status(0);
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
    child(&fixture, &root, Mode::ApplicationReplay)
        .await
        .assert_status(0);
    child(&fixture, &root, Mode::ApplicationReplay)
        .await
        .assert_status(0);
    assert_eq!(files(&root), before);
    assert_once_applied(&fixture).await;
    eprintln!(
        "Independent opaque application evidence retained: {}",
        root.display()
    );
}
async fn assert_once_applied(fixture: &Fixture) {
    assert_eq!(
        state(fixture).await,
        [1_u64.to_le_bytes(), 73_u64.to_le_bytes()].concat()
    );
}
async fn state(fixture: &Fixture) -> Vec<u8> {
    fixture
        .pic
        .query_call(fixture.principal(), fixture.caller(), "read", vec![])
        .await
        .unwrap()
}
