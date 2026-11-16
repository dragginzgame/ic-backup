//! Exact restore intent/source custody is shared with capture's one-shot recorder.
use super::*;
use crate::store::capture::{
    intent::tests::profile,
    test_fixtures::{
        completion::{complete, settled},
        source::maximum,
    },
};
#[test]
fn restore_custody_preserves_source_and_refuses_changed_intent_and_unknown_tail() {
    let (mut f, _, mut connection) = settled();
    let receipt = complete(&mut f, &mut connection).unwrap();
    let original = std::fs::read(f.ledger()).unwrap();
    let source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    let mut direct = profile();
    direct.request_bytes = 8192;
    let plan = RestorePlan::new(&source, fixtures::declaration(&source), direct).unwrap();
    drop(source);
    let predecessor = f.p.clone();
    drop(f.op);
    assert!(
        f.store
            .create_restore(&plan, Sha256Digest::compute(b"wrong"), &predecessor)
            .is_err()
    );
    let mut operation = f
        .store
        .create_restore(&plan, plan.digest(), &predecessor)
        .unwrap();
    let view = operation.inspect().unwrap();
    assert_eq!(
        view.next_call,
        Some(crate::store::OperationStep::RestoreInitialStatus)
    );
    assert_eq!(view.signing_requests_reserved, 0);
    assert!(
        f.store
            .open_restore(&plan.request().operation_id, plan.digest())
            .is_err()
    );
    drop(operation);
    let mut reopened = f
        .store
        .open_restore(&plan.request().operation_id, plan.digest())
        .unwrap();
    assert_eq!(reopened.plan().encoded(), plan.encoded());
    assert_eq!(reopened.inspect().unwrap().network_requests_reserved, 0);
    drop(reopened);
    let path = f.root.join("operations").join(&plan.request().operation_id);
    let intent = std::fs::read(path.join("intent.json")).unwrap();
    std::fs::write(path.join("intent.json"), b"{}").unwrap();
    assert!(
        f.store
            .open_restore(&plan.request().operation_id, plan.digest())
            .is_err()
    );
    std::fs::write(path.join("intent.json"), intent).unwrap();
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(path.join("journal.bin"))
        .unwrap()
        .write_all(b"unknown")
        .unwrap();
    assert!(
        f.store
            .open_restore(&plan.request().operation_id, plan.digest())
            .is_err()
    );
    assert_eq!(
        std::fs::read(
            f.root
                .join("operations")
                .join(&predecessor.request().operation_id)
                .join("journal.bin")
        )
        .unwrap(),
        original
    );
}

fn retained_restore() -> (crate::store::BackupStore, RestorePlan, std::path::PathBuf) {
    let (mut f, _, mut connection) = settled();
    let receipt = complete(&mut f, &mut connection).unwrap();
    let source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    let mut direct = profile();
    direct.request_bytes = 8192;
    let plan = RestorePlan::new(&source, fixtures::declaration(&source), direct).unwrap();
    drop(source);
    let predecessor = f.p.clone();
    drop(f.op);
    drop(
        f.store
            .create_restore(&plan, plan.digest(), &predecessor)
            .unwrap(),
    );
    (f.store, plan, f.root)
}

#[test]
fn cold_restore_reopens_from_only_id_and_approval_without_reconstructing_authority() {
    let (store, plan, root) = retained_restore();
    let id = plan.request().operation_id.clone();
    let approval = plan.digest();
    let encoded = plan.encoded().to_vec();
    let journal = root.join("operations").join(&id).join("journal.bin");
    let before = std::fs::read(&journal).unwrap();
    drop(plan);
    drop(store);
    let store = crate::store::BackupStore::open(&root).unwrap();
    let mut operation = store.open_restore(&id, approval).unwrap();
    assert_eq!(operation.plan().encoded(), encoded);
    let view = operation.inspect().unwrap();
    assert_eq!(
        view.next_call,
        Some(crate::store::OperationStep::RestoreInitialStatus)
    );
    assert_eq!(
        (
            view.signing_requests_reserved,
            view.network_requests_reserved,
            view.observations_reserved
        ),
        (0, 0, 0)
    );
    assert_eq!(std::fs::read(journal).unwrap(), before);
}

#[test]
fn cold_restore_selection_refuses_wrong_approval_identity_and_bounded_original_bytes() {
    let (store, plan, root) = retained_restore();
    let id = &plan.request().operation_id;
    let directory = root.join("operations").join(id);
    let original = std::fs::read(directory.join("intent.json")).unwrap();
    let journal = std::fs::read(directory.join("journal.bin")).unwrap();
    for wrong in [
        "../outside".to_owned(),
        id.to_uppercase(),
        "ef".repeat(16),
        plan.source_operation_id().to_owned(),
    ] {
        assert!(store.open_restore(&wrong, plan.digest()).is_err());
    }
    assert!(matches!(
        store.open_restore(id, Sha256Digest::compute(b"wrong approval")),
        Err(crate::store::RestorePersistenceError::Capture(
            crate::store::OperationStoreError::ApprovalMismatch
        ))
    ));
    for bytes in [
        b"{}".to_vec(),
        original[..original.len() - 1].to_vec(),
        vec![b' '; 1024 * 1024 + 1],
    ] {
        std::fs::write(directory.join("intent.json"), &bytes).unwrap();
        assert!(store.open_restore(id, plan.digest()).is_err());
        assert_eq!(std::fs::read(directory.join("intent.json")).unwrap(), bytes);
        assert_eq!(
            std::fs::read(directory.join("journal.bin")).unwrap(),
            journal
        );
    }
    std::fs::write(directory.join("intent.json"), original).unwrap();
    assert!(store.open_restore(id, plan.digest()).is_ok());
}

#[test]
fn independently_valid_replacement_cannot_rebind_a_selected_restore_approval() {
    // Re-authored from frozen legacy 4ae087fad8c23de712aa60bf569bde4b2b932651,
    // workflow/ic_snapshot_restore/tests/mod.rs::independently_valid_load_plan_cannot_rebind_the_admitted_snapshot_source.
    use crate::store::{
        capture::{acquire, target_for},
        open_optional,
        operation::{Journal, custody::Recorder},
    };
    let (store, plan, root) = retained_restore();
    let id = &plan.request().operation_id;
    let selected = Recorder::<RestorePlan>::select_intent(&store.root, id).unwrap();
    let target = target_for(&store.root, plan.request().target, false).unwrap();
    let lock = acquire(open_optional(&target, "lock").unwrap().unwrap()).unwrap();
    let mut record: super::super::PlanRecord = serde_json::from_slice(plan.encoded()).unwrap();
    record.profile.application = [99; 32];
    let replacement = RestorePlan::decode(&serde_json::to_vec(&record).unwrap()).unwrap();
    let directory = root.join("operations").join(id);
    let journal = Journal::create(&replacement).unwrap();
    std::fs::write(directory.join("intent.json"), replacement.encoded()).unwrap();
    std::fs::write(directory.join("journal.bin"), &journal).unwrap();
    let path = root
        .join("targets")
        .join(plan.request().target.network.to_string())
        .join(plan.request().target.canister.to_text());
    let owner = crate::store::capture::owner_bytes(id, replacement.digest()).unwrap();
    std::fs::write(path.join("owner.json"), &owner).unwrap();
    assert!(
        store
            .open_restore_locked(&selected, selected.digest(), &target, &lock)
            .is_err()
    );
    assert_eq!(
        std::fs::read(directory.join("intent.json")).unwrap(),
        replacement.encoded()
    );
    assert_eq!(
        std::fs::read(directory.join("journal.bin")).unwrap(),
        journal
    );
    assert_eq!(std::fs::read(path.join("owner.json")).unwrap(), owner);
    drop(lock);
    assert!(store.open_restore(id, plan.digest()).is_err());
    assert!(store.open_restore(id, replacement.digest()).is_ok());
}

#[test]
fn cold_restore_refuses_replaced_original_target_lock_without_adopting_it() {
    use crate::store::{
        capture::{acquire, target_for},
        open_optional,
    };
    let (store, plan, root) = retained_restore();
    let id = &plan.request().operation_id;
    let target = target_for(&store.root, plan.request().target, false).unwrap();
    let lock = acquire(open_optional(&target, "lock").unwrap().unwrap()).unwrap();
    let path = root
        .join("targets")
        .join(plan.request().target.network.to_string())
        .join(plan.request().target.canister.to_text());
    let journal = root.join("operations").join(id).join("journal.bin");
    let before = std::fs::read(&journal).unwrap();
    std::fs::rename(path.join("lock"), root.join("retained-target-lock")).unwrap();
    crate::store::tests::write_file(&path.join("lock"), b"");
    assert!(matches!(
        store.open_restore_locked(&plan, plan.digest(), &target, &lock),
        Err(crate::store::RestorePersistenceError::Capture(
            crate::store::OperationStoreError::Changed
        ))
    ));
    assert_eq!(std::fs::read(journal).unwrap(), before);
    assert!(root.join("retained-target-lock").exists());
    assert!(path.join("lock").exists());
}
