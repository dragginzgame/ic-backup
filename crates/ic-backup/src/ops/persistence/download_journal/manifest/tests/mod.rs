//! Fresh local publication, immutable record replay and acknowledged crash recovery.

use super::*;
use crate::{
    model::{artifacts::ChecksumError, download_journal::DownloadArtifactRequest},
    ops::persistence::{create_operation_plan, json::create_json_durable_at_barriers},
    test_support::{
        hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier,
        membership::{hash, plan},
        temp_dir,
    },
};
use serde_json::json;
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    process::Command,
};

fn prepare() -> (PathBuf, BackupLayoutGuard, OperationPlanRecord) {
    let root = temp_dir("ic-backup-download-manifest");
    let mut value = serde_json::to_value(plan()).unwrap();
    value["selected_targets"] = json!(["aaaaa-aa", crate::test_support::membership::APP]);
    value["operations"][0]["target"] = json!("aaaaa-aa");
    let plan: OperationPlanRecord = serde_json::from_value(value).unwrap();
    for id in plan.selected_targets() {
        let staging = root.join(format!("artifacts/{id}.tmp"));
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("heap.bin"), id.as_bytes()).unwrap();
    }
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_operation_plan(&layout, &plan).unwrap();
    let mut journal = DownloadJournalGuard::create(
        &layout,
        plan.digest().hash(),
        plan.selected_targets()
            .iter()
            .map(|id| DownloadArtifactRequest {
                canister_id: id.clone(),
                snapshot_id: format!("Snap-{id}"),
                snapshot_taken_at_timestamp: u64::MAX,
                snapshot_total_size_bytes: u64::MAX,
            })
            .collect(),
    )
    .unwrap();
    for id in plan.selected_targets() {
        let snapshot = format!("Snap-{id}");
        journal.record_downloaded(id, &snapshot).unwrap();
        journal.verify_artifact(id, &snapshot).unwrap();
        journal.finalize_artifact(id, &snapshot).unwrap();
    }
    drop(journal);
    (root, layout, plan)
}

#[test]
fn download_manifest_publishes_private_exact_records_and_replays_without_artifact_reads() {
    let (root, layout, plan) = prepare();
    let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    let record = journal.record().unwrap().clone();
    let original = fs::read(journal.path()).unwrap();
    let digest = journal.publish_download_manifest(&plan).unwrap();
    let path = root.join(MANIFEST_FILE);
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(
        matches!(journal.publish_download_manifest(&plan),Err(DownloadManifestError::Persistence(PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: error, .. }))) if error.kind()==io::ErrorKind::AlreadyExists)
    );
    assert!(matches!(
        read_download_manifest(&layout, &plan, &digest),
        Err(DownloadManifestError::Journal(DownloadJournalError::Lock(
            _
        )))
    ));
    drop(journal);
    fs::rename(root.join("artifacts"), root.join("retained-artifacts")).unwrap();
    drop(layout);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    for _ in 0..2 {
        assert_eq!(
            read_download_manifest(&layout, &plan, &digest).unwrap(),
            record
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(
            fs::read(root.join("download-journal.json")).unwrap(),
            original
        );
    }
    assert!(matches!(
        read_download_manifest(&layout, &plan, &hash("12")),
        Err(DownloadManifestError::DigestMismatch)
    ));
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn download_manifest_rejects_changed_last_tree_and_original_journal_without_publication() {
    let (root, layout, plan) = prepare();
    let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    let id = plan.selected_targets().last().unwrap();
    fs::write(root.join(format!("artifacts/{id}/heap.bin")), b"changed").unwrap();
    assert!(matches!(
        journal.publish_download_manifest(&plan),
        Err(DownloadManifestError::Integrity(
            DownloadIntegrityError::Checksum(ChecksumError::ChecksumMismatch { .. })
        ))
    ));
    assert!(!root.join(MANIFEST_FILE).exists());
    fs::write(root.join(format!("artifacts/{id}/heap.bin")), id.as_bytes()).unwrap();
    let mut changed = serde_json::to_value(journal.record().unwrap()).unwrap();
    changed["artifacts"][0]["snapshot_id"] = json!("another-token");
    fs::write(journal.path(), serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(matches!(
        journal.publish_download_manifest(&plan),
        Err(DownloadManifestError::Integrity(
            DownloadIntegrityError::JournalChanged
        ))
    ));
    assert!(!root.join(MANIFEST_FILE).exists());
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn download_manifest_lost_reply_preserves_originals_and_requires_exact_local_replay() {
    for published in [false, true] {
        let (root, layout, plan) = prepare();
        let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
        let original = fs::read(journal.path()).unwrap();
        let record = journal.record().unwrap().clone();
        assert!(
            journal
                .publish_manifest_with(&plan, |path, record| {
                    if published {
                        create_json_durable(path, record)?;
                    }
                    Err(io::Error::other("lost local publication response").into())
                })
                .is_err()
        );
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        assert_eq!(root.join(MANIFEST_FILE).exists(), published);
        if !published {
            journal.publish_download_manifest(&plan).unwrap();
        }
        drop(journal);
        assert_eq!(
            read_download_manifest(&layout, &plan, &record.digest()).unwrap(),
            record
        );
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn download_manifest_replay_rejects_missing_changed_unsafe_and_excessive_records_without_repair() {
    for kind in [
        "missing-journal",
        "changed-journal",
        "changed-plan",
        "unsafe",
        "oversized",
        "conflict",
    ] {
        let (root, layout, plan) = prepare();
        let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
        let record = journal.record().unwrap().clone();
        let digest = journal.publish_download_manifest(&plan).unwrap();
        drop(journal);
        let path = root.join(MANIFEST_FILE);
        let bytes = fs::read(&path).unwrap();
        match kind {
            "missing-journal" => fs::rename(
                root.join("download-journal.json"),
                root.join("retained-journal"),
            )
            .unwrap(),
            "changed-journal" => {
                let mut value = serde_json::to_value(&record).unwrap();
                value["artifacts"][0]["snapshot_taken_at_timestamp"] = json!(0);
                fs::write(
                    root.join("download-journal.json"),
                    serde_json::to_vec(&value).unwrap(),
                )
                .unwrap();
            }
            "changed-plan" => {
                fs::rename(root.join("operation-plan.json"), root.join("retained-plan")).unwrap();
            }
            "unsafe" => {
                fs::rename(&path, root.join("retained-manifest")).unwrap();
                symlink(root.join("retained-manifest"), &path).unwrap();
            }
            "oversized" => {
                fs::write(
                    &path,
                    vec![b' '; usize::try_from(MAX_DOWNLOAD_JOURNAL_BYTES).unwrap() + 1],
                )
                .unwrap();
            }
            "conflict" => {
                let mut value = serde_json::to_value(&record).unwrap();
                value["artifacts"][0]["snapshot_id"] = json!("other");
                fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        let result = read_download_manifest(&layout, &plan, &digest);
        match kind {
            "changed-journal" => {
                assert!(matches!(result, Err(DownloadManifestError::JournalChanged)));
            }
            "oversized" => assert!(matches!(
                result,
                Err(DownloadManifestError::Persistence(
                    PersistenceError::RecordTooLarge { .. }
                ))
            )),
            "conflict" => assert!(matches!(result, Err(DownloadManifestError::DigestMismatch))),
            _ => assert!(result.is_err()),
        }
        if !matches!(kind, "oversized" | "conflict") {
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn download_manifest_acknowledged_death_on_both_publication_sides_recovers_originals() {
    const ROOT: &str = "IC_BACKUP_MANIFEST_CHILD_ROOT";
    const PLAN: &str = "IC_BACKUP_MANIFEST_CHILD_PLAN";
    const PUBLISHED: &str = "IC_BACKUP_MANIFEST_CHILD_PUBLISHED";
    const HANDSHAKE: &str = "IC_BACKUP_MANIFEST_CHILD_HANDSHAKE";
    if let Some(root) = std::env::var_os(ROOT) {
        let root = PathBuf::from(root);
        let plan: OperationPlanRecord =
            serde_json::from_str(&std::env::var(PLAN).unwrap()).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
        let handshake = PathBuf::from(std::env::var_os(HANDSHAKE).unwrap());
        let published = std::env::var(PUBLISHED).unwrap() == "yes";
        journal
            .publish_manifest_with(&plan, |path, record| {
                create_json_durable_at_barriers(
                    path,
                    record,
                    || {
                        if !published {
                            hold_at_acknowledged_barrier(&handshake);
                        }
                    },
                    || {
                        if published {
                            hold_at_acknowledged_barrier(&handshake);
                        }
                    },
                )
            })
            .unwrap();
        panic!("manifest crash child passed acknowledged barrier");
    }
    for published in [false, true] {
        let (root, layout, plan) = prepare();
        let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
        let record = journal.record().unwrap().clone();
        let original = fs::read(journal.path()).unwrap();
        drop(journal);
        drop(layout);
        let handshake = temp_dir("ic-backup-download-manifest-handshake");
        fs::create_dir(&handshake).unwrap();
        let mut child=Command::new(std::env::current_exe().unwrap())
            .args(["--exact","ops::persistence::download_journal::manifest::tests::download_manifest_acknowledged_death_on_both_publication_sides_recovers_originals","--nocapture"])
            .env(ROOT,&root).env(PLAN,serde_json::to_string(&plan).unwrap())
            .env(PUBLISHED,if published {"yes"} else {"no"}).env(HANDSHAKE,&handshake).spawn().unwrap();
        kill_child_at_acknowledged_barrier(&mut child, &handshake);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        assert_eq!(root.join(MANIFEST_FILE).exists(), published);
        assert_eq!(
            fs::read(root.join("download-journal.json")).unwrap(),
            original
        );
        if !published {
            let journal = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
            journal.publish_download_manifest(&plan).unwrap();
        }
        assert_eq!(
            read_download_manifest(&layout, &plan, &record.digest()).unwrap(),
            record
        );
        drop(layout);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(handshake).unwrap();
    }
}
