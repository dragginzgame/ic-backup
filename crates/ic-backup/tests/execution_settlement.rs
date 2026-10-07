//! Public local settlement replay retains fence/source obligations; no provider or IC effects.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        download_journal::DownloadArtifactRequest,
        execution_settlement::{ExecutionSettlementJournalRecord, ExecutionSettlementRecord},
        fence_obligation::FenceObligationRecord,
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreFenceBindingRecord, RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest,
        },
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, FenceObligationRequirement,
        create_execution_settlement, create_fence_obligation, create_operation_plan,
        create_restore_safety_requirement, read_download_manifest, read_execution_settlement,
    },
};
use serde_json::json;
use std::fs;

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({"version":1,
        "context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":"renrk-eyaaa-aaaaa-aaada-cai","parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":["renrk-eyaaa-aaaaa-aaada-cai"],
        "graph":{"version":1,"nodes":[{"operation_sequence":0,"depends_on":[]},{"operation_sequence":7,"depends_on":[0]}]},
        "operations":[{"operation_sequence":0,"target":"renrk-eyaaa-aaaaa-aaada-cai","request":"12".repeat(32),"budget":{"mutations":1,"observations":1}},
            {"operation_sequence":7,"target":"renrk-eyaaa-aaaaa-aaada-cai","request":"34".repeat(32),"budget":{"mutations":2,"observations":1}}],
        "budget":{"mutations":3,"observations":2}
    })).unwrap()
}
#[test]
fn all_applied_local_replay_retains_exact_spending_fence_and_source_without_fresh_calls() {
    let root = support::temp_root("ic-backup-public-settlement");
    fs::create_dir_all(root.join("source")).unwrap();
    fs::create_dir(root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let plan = plan();
    retain_originals(&layout, &source_layout, &plan);
    source_layout
        .retain_restore(&root.join("restore/attempt-0.json"), plan.digest().hash())
        .unwrap();
    let references = source_layout.restore_references().unwrap();
    let obligation_bytes = fs::read(root.join("restore/fence-obligation.json")).unwrap();
    // Passive native operation receipts exercise local owners, not actual application effects.
    let mut rows = Vec::new();
    let mut journal_bytes = Vec::new();
    for authority in plan.attempt_authorities().unwrap() {
        let mut journal = AttemptJournalGuard::create(&layout, authority).unwrap();
        let attempt = journal.reserve_mutation().unwrap();
        let request = journal
            .record()
            .unwrap()
            .authority()
            .binding()
            .request()
            .into();
        journal
            .record_mutation(MutationReceiptRequest {
                attempt,
                request,
                outcome: MutationOutcomeRecord::Applied,
                evidence: hash("ef").hash().into(),
            })
            .unwrap();
        rows.push(ExecutionSettlementJournalRecord::from_journal(
            journal.record().unwrap(),
        ));
        journal_bytes.push((journal.path(), fs::read(journal.path()).unwrap()));
    }
    let record = ExecutionSettlementRecord::new(plan.digest(), rows).unwrap();
    create_execution_settlement(&layout, &record).unwrap();
    let bytes = fs::read(root.join("restore/execution-settlement.json")).unwrap();
    assert_eq!(source_layout.restore_references().unwrap(), references);
    assert_eq!(
        fs::read(root.join("restore/fence-obligation.json")).unwrap(),
        obligation_bytes
    );
    drop(layout);
    drop(source_layout);
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    for _ in 0..2 {
        assert_eq!(
            read_execution_settlement(&layout, &plan.digest(), &record.digest()).unwrap(),
            record
        );
        assert_eq!(
            fs::read(root.join("restore/execution-settlement.json")).unwrap(),
            bytes
        );
        assert_eq!(source_layout.restore_references().unwrap(), references);
        assert_eq!(
            fs::read(root.join("restore/fence-obligation.json")).unwrap(),
            obligation_bytes
        );
        for (path, original) in &journal_bytes {
            assert_eq!(&fs::read(path).unwrap(), original);
        }
    }
    let journal = AttemptJournalGuard::open(&layout, &plan.attempt_authority(7).unwrap()).unwrap();
    let view = journal.record().unwrap().view();
    assert!(view.applied);
    assert_eq!(view.mutations_used, 1);
    assert_eq!(view.mutations_remaining, 1);
    assert_eq!(view.pending_mutation, None);
    assert_eq!(view.pending_observation, None);
    drop(journal);
    drop(source_layout);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

fn retain_originals(
    layout: &BackupLayoutGuard,
    source_layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
) -> OperationPlanRecord {
    let mut source = serde_json::to_value(plan).unwrap();
    source["operations"][0]["request"] = json!("56".repeat(32));
    let source: OperationPlanRecord = serde_json::from_value(source).unwrap();
    create_operation_plan(source_layout, &source).unwrap();
    create_operation_plan(layout, plan).unwrap();
    let requirement = RestoreSafetyRequirementRecord::new(
        plan,
        &source,
        RestoreSafetyRequirementRequest {
            source_artifacts: hash("78"),
            safety: RestoreSafetyLaneRecord::ApplicationFenced,
            expected_fence: Some(RestoreFenceBindingRecord {
                identity: hash("90"),
                membership_revision: hash("ab"),
                external_obligations_revision: hash("cd"),
            }),
        },
    )
    .unwrap();
    create_restore_safety_requirement(layout, source_layout, plan, &source, &requirement).unwrap();
    let obligation = FenceObligationRecord::for_restore(plan, &source, &requirement, 0).unwrap();
    create_fence_obligation(
        layout,
        plan,
        FenceObligationRequirement::Restore {
            source_layout,
            source: &source,
            requirement: &requirement,
        },
        &obligation,
    )
    .unwrap();
    source
}

#[test]
fn immutable_download_manifest_replay_retains_unfinished_restore_references() {
    let root = support::temp_root("ic-backup-public-settlement");
    fs::create_dir_all(root.join("source")).unwrap();
    fs::create_dir(root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let restore = plan();
    let source = retain_originals(&layout, &source_layout, &restore);
    source_layout
        .retain_restore(
            &root.join("restore/attempt-0.json"),
            restore.digest().hash(),
        )
        .unwrap();
    let references = source_layout.restore_references().unwrap();
    let obligation = fs::read(root.join("restore/fence-obligation.json")).unwrap();
    let target = &source.selected_targets()[0];
    let snapshot = "Original-Snapshot";
    let staging = root.join(format!("source/artifacts/{target}.tmp"));
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("heap.bin"), b"retained native fixture bytes").unwrap();
    let mut journal = DownloadJournalGuard::create(
        &source_layout,
        source.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: target.clone(),
            snapshot_id: snapshot.into(),
            snapshot_taken_at_timestamp: u64::MAX,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap();
    journal.record_downloaded(target, snapshot).unwrap();
    journal.verify_artifact(target, snapshot).unwrap();
    journal.finalize_artifact(target, snapshot).unwrap();
    let record = journal.record().unwrap().clone();
    let digest = journal.publish_download_manifest(&source).unwrap();
    let bytes = fs::read(root.join("source/download-manifest.json")).unwrap();
    let original = fs::read(journal.path()).unwrap();
    drop(journal);
    drop(source_layout);
    drop(layout);
    // Retain the fixture trees elsewhere to prove replay never reopens artifacts.
    fs::rename(
        root.join("source/artifacts"),
        root.join("retained-artifacts"),
    )
    .unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    for _ in 0..2 {
        assert_eq!(
            read_download_manifest(&source_layout, &source, &digest).unwrap(),
            record
        );
        assert_eq!(
            fs::read(root.join("source/download-manifest.json")).unwrap(),
            bytes
        );
        assert_eq!(
            fs::read(root.join("source/download-journal.json")).unwrap(),
            original
        );
        assert_eq!(source_layout.restore_references().unwrap(), references);
        assert_eq!(
            fs::read(root.join("restore/fence-obligation.json")).unwrap(),
            obligation
        );
    }
    drop(source_layout);
    fs::remove_dir_all(root).unwrap();
}
