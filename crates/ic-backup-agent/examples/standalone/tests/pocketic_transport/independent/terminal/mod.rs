//! Terminal acceptance and exact reference release through the normal consumer.

use super::*;
use ic_backup::ops::persistence::BackupLayoutGuard;

pub(super) async fn complete(fixture: &mut Fixture, root: &Path) {
    grant(fixture, root, Mode::Start).await;
    let before = files(root);
    child(fixture, root, Mode::Start).await.assert_status(1);
    assert_eq!(files(root), before);
    assert!(!Stage::Start.path(root).exists());
    let acceptance = verify(fixture, root).await;
    let accepted = serde_json::to_vec(&acceptance).unwrap();
    let path = root.join("acceptance.json");
    let mut substituted = acceptance;
    substituted["complete_state"] = serde_json::json!(false);
    let substituted = serde_json::to_vec(&substituted).unwrap();
    consumer::retain(&path, &substituted).unwrap();
    let before = files(root);
    child(fixture, root, Mode::Start).await.assert_status(1);
    assert_eq!(files(root), before);
    assert!(!Stage::Start.path(root).exists());
    fs::write(&path, &accepted).unwrap();

    let peer = root.join("unfinished-restore");
    fs::create_dir(&peer).unwrap();
    let source = BackupLayoutGuard::acquire(&Stage::Download.path(root)).unwrap();
    let peer = source
        .retain_restore(
            &peer.join("execution-workflow.json"),
            ArtifactChecksumRecord::from_bytes(b"another unfinished restore").hash(),
        )
        .unwrap();
    drop(source);
    child(fixture, root, Mode::Start).await.assert_status(0);
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
    let source = BackupLayoutGuard::acquire(&Stage::Download.path(root)).unwrap();
    assert_eq!(source.restore_references().unwrap().entries(), [peer]);
    assert!(source.has_restore_references().unwrap());
    drop(source);

    let before = files(root);
    child(fixture, root, Mode::Replay).await.assert_status(0);
    assert_eq!(files(root), before);
    // Even already-released replay must freshly admit the original acceptance.
    fs::write(&path, &substituted).unwrap();
    let before = files(root);
    child(fixture, root, Mode::Replay).await.assert_status(1);
    assert_eq!(files(root), before);
    fs::write(&path, &accepted).unwrap();
    let before = files(root);
    child(fixture, root, Mode::Replay).await.assert_status(0);
    assert_eq!(files(root), before);
}
