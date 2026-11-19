//! Re-authored from frozen legacy 4ae087fad8c23de712aa60bf569bde4b2b932651,
//! Native custody and recovery regressions, not physical-power-loss or IC qualification.
//! ops/persistence/attempt_journal/tests and agent tests/http_boundary/recovery.

use super::{
    super::{
        procedure_tests::{expected_request, reply, selected},
        replay,
    },
    *,
};
use crate::store::{
    capture::{
        advance::{PreparedStatus, StatusEvidence},
        intent::tests::signed_reply,
        procedure::Next,
        tests::{certificates::signed, lookup::lookup, signatures::Scheme},
    },
    tests::{root, write_file},
};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, symlink},
    path::{Path, PathBuf},
};

pub(in crate::store::operation) fn store(path: &Path) -> Directory {
    write_file(&path.join("format"), b"ic-backup/host-store/1\n");
    for name in ["operations", "bundles", "targets"] {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path.join(name))
            .unwrap();
    }
    Directory::open(path, PRIVATE).unwrap().unwrap()
}

fn fixture() -> (PathBuf, Directory, CaptureOperation) {
    let path = root("capture-recorder");
    let parent = store(&path);
    let recorder = CaptureOperation::create(&parent, selected()).unwrap();
    (path, parent, recorder)
}

#[test]
fn renamed_store_keeps_original_custody_and_never_reopens_the_replacement_root() {
    let (path, _, operation) = fixture();
    let plan = operation.plan().clone();
    let moved = path.with_extension("retained");
    fs::rename(&path, &moved).unwrap();
    fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
    let replacement = crate::store::BackupStore::initialize(&path).unwrap();
    assert_eq!(operation.inspect().unwrap().signing_requests_reserved, 0);
    assert!(
        replacement
            .open_capture(&plan.request().operation_id, plan.digest())
            .is_err()
    );
    drop(operation);
    let original = crate::store::BackupStore::open(&moved).unwrap();
    let operation = original
        .open_capture(&plan.request().operation_id, plan.digest())
        .unwrap();
    assert_eq!(operation.plan().encoded(), plan.encoded());
    assert_eq!(operation.inspect().unwrap().network_requests_reserved, 0);
}

#[test]
fn capture_creation_refuses_a_different_held_target_lock_without_cleanup() {
    let (path, parent, operation) = fixture();
    let mut request = operation.plan().request().clone();
    request.operation_id = "ff".repeat(16);
    let intent = CapturePlan::new(request, operation.plan().profile().clone()).unwrap();
    let initial = Journal::create(&intent).unwrap();
    let target = target_for(&parent, intent.request().target, false).unwrap();
    fs::write(
        path.join("targets")
            .join(intent.request().target.network.to_string())
            .join(intent.request().target.canister.to_text())
            .join("owner.json"),
        owner_bytes(&intent.request().operation_id, intent.digest()).unwrap(),
    )
    .unwrap();
    exact(
        &target,
        "owner.json",
        &owner_bytes(&intent.request().operation_id, intent.digest()).unwrap(),
        1024,
    )
    .unwrap();
    let wrong_path = path.join("different-lock");
    crate::store::tests::write_file(&wrong_path, b"");
    let wrong = acquire(fs::File::open(wrong_path).unwrap()).unwrap();
    assert!(matches!(
        CaptureOperation::create_claimed(&parent, intent.clone(), wrong, &initial),
        Err(OperationStoreError::Changed)
    ));
    let retained = path.join("operations").join(&intent.request().operation_id);
    assert_eq!(
        fs::read(retained.join("intent.json")).unwrap(),
        intent.encoded()
    );
    assert_eq!(fs::read(retained.join("journal.bin")).unwrap(), initial);
}

fn directory(path: &Path, recorder: &CaptureOperation) -> PathBuf {
    path.join("operations")
        .join(&recorder.intent.plan().request().operation_id)
}

fn append_call(recorder: &mut CaptureOperation, step: Step, settle: bool) -> PreparedStatus {
    let (method, args) = expected_request(&recorder.intent, step);
    let (request, evidence) = signed_reply(
        &recorder.intent,
        step_number(step),
        method,
        args,
        reply(&recorder.intent, step),
    );
    recorder
        .append_fixture(
            FactRecord::CallReserved {
                step,
                reference_ns: 100,
            },
            None,
        )
        .unwrap();
    recorder
        .append_fixture(
            FactRecord::CallPrepared {
                step,
                request: ReferenceRecord::new(&request.envelope).unwrap(),
            },
            Some(&request.envelope),
        )
        .unwrap();
    recorder
        .append_fixture(FactRecord::CallSubmitted { step }, None)
        .unwrap();
    if settle {
        settle_call(recorder, step, &evidence);
    }
    request
}

fn step_number(step: Step) -> u8 {
    match step {
        Step::InitialStatus => 0,
        Step::ListSnapshots => 1,
        Step::TakeSnapshot => 2,
        Step::Metadata => 3,
        Step::Data(n) => n as u8 + 4,
        Step::FinalStatus => 99,
        _ => panic!("capture fixture received restore step"),
    }
}

fn settle_call(recorder: &mut CaptureOperation, step: Step, evidence: &StatusEvidence) {
    let bytes = evidence
        .encode(recorder.intent.profile().result_bytes)
        .unwrap();
    recorder
        .append_fixture(
            FactRecord::CallResult {
                step,
                result: ReferenceRecord::new(&bytes).unwrap(),
            },
            Some(&bytes),
        )
        .unwrap();
}

fn replay(recorder: &CaptureOperation, materialized: bool) -> Result<Next, replay::Error> {
    let journal = recorder.check()?;
    let flow = journal.replay(
        &recorder.intent,
        |call, role| recorder.read_role(call, role),
        |_| {
            if materialized {
                Ok(())
            } else {
                Err(OperationStoreError::Incomplete.into())
            }
        },
    )?;
    Ok(flow.next()?)
}

fn with_observations() -> CapturePlan {
    let intent = selected();
    let mut request = intent.plan().request().clone();
    request.limits.observations = 1;
    CapturePlan::new(request, intent.profile().clone()).unwrap()
}

fn observe(recorder: &mut CaptureOperation, original: &PreparedStatus, outcome: &[u8]) {
    let step = Step::InitialStatus;
    // Lookup time differs from the original call. Its proof is too late for the
    // original [100,120] window but valid under the separately retained [200,220].
    let bytes = lookup(recorder.intent.plan(), original, Scheme::Ed25519, 220);
    recorder
        .append_fixture(
            FactRecord::ObservationReserved {
                step,
                reference_ns: 200,
            },
            None,
        )
        .unwrap();
    recorder
        .append_fixture(
            FactRecord::ObservationPrepared {
                step,
                request: ReferenceRecord::new(&bytes).unwrap(),
            },
            Some(&bytes),
        )
        .unwrap();
    recorder
        .append_fixture(FactRecord::ObservationSubmitted { step }, None)
        .unwrap();
    // Keep canonical IC time LEB128 and real BLS signing; no protocol decoder is faked.
    use ic_platform::certificate::construction::{fork, labeled, leaf};
    let tree = fork(
        labeled(
            b"request_status",
            labeled(
                original.request_id.to_vec(),
                fork(
                    labeled(b"reply", leaf(reply(&recorder.intent, step))),
                    labeled(b"status", leaf(outcome.to_vec())),
                ),
            ),
        ),
        labeled(b"time", leaf(vec![200, 1])),
    );
    let certificate = signed(tree, 1, None);
    recorder
        .append_fixture(
            FactRecord::ObservationResult {
                step,
                certificate: ReferenceRecord::new(&certificate).unwrap(),
            },
            Some(&certificate),
        )
        .unwrap();
}

#[test]
fn exact_envelopes_and_spending_survive_cold_custody_replay() {
    let (path, parent, mut recorder) = fixture();
    let id = recorder.intent.plan().request().operation_id.clone();
    let request = append_call(&mut recorder, Step::InitialStatus, true);
    assert_eq!(
        fs::read(directory(&path, &recorder).join(Step::InitialStatus.name(Role::Request)))
            .unwrap(),
        request.envelope
    );
    assert_eq!(
        replay(&recorder, false).unwrap(),
        Next::Call(Step::ListSnapshots)
    );
    let spent = recorder.check().unwrap();
    assert_eq!((spent.signatures, spent.network), (1, 1));
    assert!(matches!(
        CaptureOperation::open(&parent, &id),
        Err(OperationStoreError::Locked)
    ));
    drop(recorder);
    let reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert_eq!(reopened.check().unwrap(), spent);
    assert_eq!(
        replay(&reopened, false).unwrap(),
        Next::Call(Step::ListSnapshots)
    );
}

#[test]
fn uncertain_full_reservations_stay_spent_and_partial_frames_refuse() {
    for partial in [false, true] {
        let (path, parent, mut recorder) = fixture();
        let id = recorder.intent.plan().request().operation_id.clone();
        let result = recorder.append_fixture_with(
            FactRecord::CallReserved {
                step: Step::InitialStatus,
                reference_ns: 100,
            },
            None,
            |file, frame| {
                file.write_all(if partial { &frame[..7] } else { frame })?;
                file.sync_all()?;
                Err(io::Error::other("interrupted acknowledgement"))
            },
        );
        assert!(matches!(result, Err(OperationStoreError::Io(_))));
        assert!(matches!(
            recorder.check(),
            Err(OperationStoreError::Unusable)
        ));
        let original = fs::read(directory(&path, &recorder).join("journal.bin")).unwrap();
        drop(recorder);
        if partial {
            assert!(CaptureOperation::open(&parent, &id).is_err());
        } else {
            let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
            let spent = reopened.check().unwrap();
            assert_eq!((spent.signatures, spent.network), (1, 1));
            assert!(spent.calls[0].update.request.is_none());
            assert!(
                reopened
                    .append_fixture(
                        FactRecord::CallReserved {
                            step: Step::InitialStatus,
                            reference_ns: 101
                        },
                        None
                    )
                    .is_err()
            );
        }
        assert_eq!(
            fs::read(path.join("operations").join(&id).join("journal.bin")).unwrap(),
            original
        );
    }
}

#[test]
fn append_reuses_only_its_exact_current_prefix_and_expected_written_frame() {
    let (path, parent, mut operation) = fixture();
    let initial = operation.check().unwrap();
    let reserved = operation
        .append(
            &initial,
            FactRecord::CallReserved {
                step: Step::InitialStatus,
                reference_ns: 100,
            },
            None,
        )
        .unwrap();
    let journal_path = directory(&path, &operation).join("journal.bin");
    let before = fs::read(&journal_path).unwrap();
    assert!(matches!(
        operation.append(
            &initial,
            FactRecord::CallReserved {
                step: Step::InitialStatus,
                reference_ns: 200
            },
            None
        ),
        Err(OperationStoreError::Changed)
    ));
    assert_eq!(operation.check().unwrap(), reserved);
    assert_eq!(fs::read(journal_path).unwrap(), before);
    let id = operation.plan().request().operation_id.clone();
    drop(operation);
    assert_eq!(
        CaptureOperation::open(&parent, &id)
            .unwrap()
            .check()
            .unwrap(),
        reserved
    );
    let (path, parent, mut operation) = fixture();
    let original = fs::read(directory(&path, &operation).join("journal.bin")).unwrap();
    assert!(
        operation
            .append_fixture_with(
                FactRecord::CallReserved {
                    step: Step::InitialStatus,
                    reference_ns: 200
                },
                None,
                |_, _| Ok(())
            )
            .is_err()
    );
    assert!(matches!(
        operation.check(),
        Err(OperationStoreError::Unusable)
    ));
    assert_eq!(
        fs::read(directory(&path, &operation).join("journal.bin")).unwrap(),
        original
    );
    drop(operation);
    assert_eq!(
        CaptureOperation::open(&parent, &id)
            .unwrap()
            .check()
            .unwrap()
            .signatures,
        0
    );
}

#[test]
fn unreferenced_surviving_evidence_is_retained_and_never_adopted() {
    let (path, parent, mut recorder) = fixture();
    let id = recorder.intent.plan().request().operation_id.clone();
    recorder
        .append_fixture(
            FactRecord::CallReserved {
                step: Step::InitialStatus,
                reference_ns: 100,
            },
            None,
        )
        .unwrap();
    let bytes = b"surviving output";
    assert!(
        recorder
            .append_fixture_with(
                FactRecord::CallPrepared {
                    step: Step::InitialStatus,
                    request: ReferenceRecord::new(bytes).unwrap()
                },
                Some(bytes),
                |_, _| Err(io::Error::other("journal unavailable"))
            )
            .is_err()
    );
    let name = directory(&path, &recorder).join(Step::InitialStatus.name(Role::Request));
    assert_eq!(fs::read(&name).unwrap(), bytes);
    assert!(matches!(
        recorder.check(),
        Err(OperationStoreError::Unusable)
    ));
    drop(recorder);
    assert!(CaptureOperation::open(&parent, &id).is_err());
    assert_eq!(fs::read(name).unwrap(), bytes);
}

#[test]
fn changed_replaced_aliased_and_orphan_files_refuse_without_repair() {
    for mutation in 0..8 {
        let (path, parent, mut recorder) = fixture();
        append_call(&mut recorder, Step::InitialStatus, true);
        let dir = directory(&path, &recorder);
        let evidence = dir.join(Step::InitialStatus.name(Role::Result));
        let bytes = fs::read(&evidence).unwrap();
        match mutation {
            0 => {
                let mut changed = bytes;
                changed[5] ^= 1;
                fs::write(&evidence, changed).unwrap();
            }
            1 => {
                fs::rename(&evidence, path.join("original")).unwrap();
                write_file(&evidence, &bytes);
            }
            2 => {
                fs::rename(&evidence, path.join("original")).unwrap();
                symlink(path.join("original"), &evidence).unwrap();
            }
            3 => {
                fs::hard_link(&evidence, path.join("alias")).unwrap();
            }
            4 => {
                write_file(&dir.join("orphan.bin"), b"retained");
            }
            5 => {
                fs::rename(&dir, path.join("held-original")).unwrap();
                fs::create_dir(&dir).unwrap();
            }
            6 => fs::write(&evidence, &bytes[..bytes.len() - 1]).unwrap(),
            7 => fs::write(&evidence, [bytes.as_slice(), b"extra"].concat()).unwrap(),
            _ => unreachable!(),
        }
        assert!(recorder.check().is_err(), "{mutation}");
        let id = recorder.intent.plan().request().operation_id.clone();
        drop(recorder);
        // An identical replacement before reopening has no persisted inode identity;
        // held custody must detect it during this session, not invent historical proof.
        if mutation != 1 {
            assert!(CaptureOperation::open(&parent, &id).is_err(), "{mutation}");
        }
    }
}

#[test]
fn evidence_read_requires_both_declared_length_and_digest() {
    let (_, _, mut recorder) = fixture();
    append_call(&mut recorder, Step::InitialStatus, true);
    let journal = recorder.check().unwrap();
    let name = Step::InitialStatus.name(Role::Result);
    let original = journal
        .evidence()
        .find(|(n, _)| *n == name)
        .unwrap()
        .1
        .clone();
    let bytes = recorder.read(&name, &original).unwrap();
    assert_eq!(bytes.len(), original.bytes as usize);

    // A matching digest does not excuse a longer declaration ending before its bound.
    let mut longer = original.clone();
    longer.bytes += 1;
    assert!(matches!(
        recorder.read(&name, &longer),
        Err(OperationStoreError::Changed)
    ));
    let mut shorter = original.clone();
    shorter.bytes -= 1;
    assert!(matches!(
        recorder.read(&name, &shorter),
        Err(OperationStoreError::Store(StoreError::Read(
            ic_host_artifacts::Error::Limit { .. }
        )))
    ));
    let mut wrong_hash = original;
    wrong_hash.sha256[0] ^= 1;
    assert!(matches!(
        recorder.read(&name, &wrong_hash),
        Err(OperationStoreError::Changed)
    ));
    assert_eq!(recorder.check().unwrap(), journal);
}

#[test]
fn replay_requires_original_signature_for_pending_reply_and_observed_calls() {
    for resolution in 0..4 {
        let path = root("capture-replay-signature");
        let parent = store(&path);
        let mut recorder = CaptureOperation::create(&parent, with_observations()).unwrap();
        let mut original = append_call(&mut recorder, Step::InitialStatus, resolution == 1);
        if resolution >= 2 {
            observe(
                &mut recorder,
                &original,
                if resolution == 2 {
                    b"replied"
                } else {
                    b"processing"
                },
            );
        }
        if resolution == 3 {
            let (method, args) = expected_request(&recorder.intent, Step::InitialStatus);
            let (_, evidence) = signed_reply(
                &recorder.intent,
                0,
                method,
                args,
                reply(&recorder.intent, Step::InitialStatus),
            );
            settle_call(&mut recorder, Step::InitialStatus, &evidence);
        }
        let journal = recorder.check().unwrap();
        assert!(replay(&recorder, false).is_ok());
        // The content/request ID and valid result proof stay unchanged. A result
        // certificate alone cannot excuse an invalid original request signature.
        crate::store::capture::tests::signatures::change(&mut original, |envelope| {
            envelope.sender_sig.as_mut().unwrap()[0] ^= 1;
        });
        let result = journal.replay(
            &recorder.intent,
            |call, role| {
                if matches!(role, Role::Request) {
                    Ok(original.envelope.clone())
                } else {
                    recorder.read_role(call, role)
                }
            },
            |_| unreachable!(),
        );
        assert!(matches!(
            result,
            Err(replay::Error::Authentication(
                crate::store::operation::intent::Error::Request(
                    crate::store::StatusRequestError::Signature
                )
            ))
        ));
        assert_eq!(recorder.check().unwrap(), journal);
    }
}

#[test]
fn terminal_observation_replays_its_separate_time_without_a_copied_result() {
    let path = root("capture-recorder-observation");
    let parent = store(&path);
    let mut recorder = CaptureOperation::create(&parent, with_observations()).unwrap();
    let original = append_call(&mut recorder, Step::InitialStatus, false);
    observe(&mut recorder, &original, b"replied");
    assert_eq!(
        replay(&recorder, false).unwrap(),
        Next::Call(Step::ListSnapshots)
    );
    let journal = recorder.check().unwrap();
    assert!(journal.calls[0].update.result.is_none());
    assert_eq!(
        (journal.signatures, journal.network, journal.observations),
        (2, 2, 1)
    );
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    let reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert_eq!(
        replay(&reopened, false).unwrap(),
        Next::Call(Step::ListSnapshots)
    );
}

#[test]
fn nonterminal_observation_cannot_unlock_a_forged_later_history() {
    for outcome in [b"received".as_slice(), b"processing", b"done"] {
        let path = root("capture-recorder-pending");
        let parent = store(&path);
        let mut recorder = CaptureOperation::create(&parent, with_observations()).unwrap();
        let original = append_call(&mut recorder, Step::InitialStatus, false);
        observe(&mut recorder, &original, outcome);
        assert_eq!(
            replay(&recorder, false).unwrap(),
            Next::Call(Step::InitialStatus)
        );
        recorder
            .append_fixture(
                FactRecord::CallReserved {
                    step: Step::ListSnapshots,
                    reference_ns: 100,
                },
                None,
            )
            .unwrap();
        assert!(matches!(
            replay(&recorder, false),
            Err(replay::Error::History)
        ));
    }
}

#[test]
fn interrupted_observation_remains_spent_after_surviving_original_reply() {
    let path = root("capture-recorder-surviving-reply");
    let parent = store(&path);
    let mut recorder = CaptureOperation::create(&parent, with_observations()).unwrap();
    append_call(&mut recorder, Step::InitialStatus, false);
    recorder
        .append_fixture(
            FactRecord::ObservationReserved {
                step: Step::InitialStatus,
                reference_ns: 200,
            },
            None,
        )
        .unwrap();
    let (method, args) = expected_request(&recorder.intent, Step::InitialStatus);
    let (_, evidence) = signed_reply(
        &recorder.intent,
        0,
        method,
        args,
        reply(&recorder.intent, Step::InitialStatus),
    );
    settle_call(&mut recorder, Step::InitialStatus, &evidence);
    assert_eq!(
        replay(&recorder, false).unwrap(),
        Next::Call(Step::ListSnapshots)
    );
    let journal = recorder.check().unwrap();
    assert_eq!(
        (journal.signatures, journal.network, journal.observations),
        (2, 2, 1)
    );
    assert!(
        journal.calls[0]
            .observation
            .as_ref()
            .unwrap()
            .request
            .is_none()
    );
}

#[test]
fn final_status_requires_verified_materialization_on_every_replay() {
    let (path, parent, mut recorder) = fixture();
    for step in [
        Step::InitialStatus,
        Step::ListSnapshots,
        Step::TakeSnapshot,
        Step::Metadata,
    ] {
        append_call(&mut recorder, step, true);
    }
    for n in 0..9 {
        append_call(&mut recorder, Step::Data(n), true);
    }
    assert_eq!(replay(&recorder, false).unwrap(), Next::Materialize);
    recorder
        .append_fixture(FactRecord::MaterializationStarted, None)
        .unwrap();
    append_call(&mut recorder, Step::FinalStatus, true);
    assert!(replay(&recorder, false).is_err());
    assert_eq!(replay(&recorder, true).unwrap(), Next::Publish);
    let id = recorder.intent.plan().request().operation_id.clone();
    let before = fs::read(directory(&path, &recorder).join("journal.bin")).unwrap();
    drop(recorder);
    let reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert!(replay(&reopened, false).is_err());
    assert_eq!(replay(&reopened, true).unwrap(), Next::Publish);
    assert_eq!(
        fs::read(path.join("operations").join(id).join("journal.bin")).unwrap(),
        before
    );
}

#[test]
fn old_store_marker_and_changed_target_custody_refuse_before_work() {
    let old = root("recorder-old-store");
    write_file(&old.join("format"), b"ic-backup/local-store/1\n");
    for name in ["bundles", "operations", "targets"] {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(old.join(name))
            .unwrap();
    }
    let directory = Directory::open(&old, PRIVATE).unwrap().unwrap();
    let before = fs::read(old.join("format")).unwrap();
    let intent = selected();
    let id = intent.plan().request().operation_id.clone();
    assert!(CaptureOperation::create(&directory, intent).is_err());
    assert!(CaptureOperation::open(&directory, &id).is_err());
    assert_eq!(fs::read(old.join("format")).unwrap(), before);
    assert!(matches!(
        crate::store::BackupStore::open(&old),
        Err(crate::store::StoreError::UnsupportedFormat)
    ));
    assert!(
        fs::read_dir(old.join("operations"))
            .unwrap()
            .next()
            .is_none()
    );
    for missing_lock in [false, true] {
        let (path, parent, recorder) = fixture();
        let id = recorder.intent.plan().request().operation_id.clone();
        let target = recorder.intent.plan().request().target;
        let original = path
            .join("targets")
            .join(target.network.to_string())
            .join(target.canister.to_text());
        if missing_lock {
            fs::rename(original.join("lock"), path.join("original-lock")).unwrap();
        } else {
            fs::write(original.join("owner.json"), b"changed-owner").unwrap();
        }
        assert!(recorder.check().is_err());
        drop(recorder);
        assert!(CaptureOperation::open(&parent, &id).is_err());
        assert!(
            fs::read_dir(path.join("operations").join(id))
                .unwrap()
                .count()
                == 3
        );
    }
}

#[test]
fn materialized_closed_tree_rejects_legacy_corruption_cases_on_new_replay() {
    // Frozen legacy ic_snapshot_artifact/tests::
    // changed_unsafe_extra_and_missing_files_reject_before_any_journal_transition.
    // Missing members may be materialized before final status in the new contract;
    // after that request is retained, replay must only verify the original tree.
    for mutation in 0..9 {
        let (path, parent, mut recorder) = fixture();
        for step in [
            Step::InitialStatus,
            Step::ListSnapshots,
            Step::TakeSnapshot,
            Step::Metadata,
        ] {
            append_call(&mut recorder, step, true);
        }
        for n in 0..9 {
            append_call(&mut recorder, Step::Data(n), true);
        }
        recorder
            .append_fixture(FactRecord::MaterializationStarted, None)
            .unwrap();
        let journal = recorder.check().unwrap();
        let intent = recorder.intent.clone();
        let flow = recorder.project(&journal, &intent).unwrap();
        recorder.materialize(&journal, &flow, true).unwrap();
        append_call(&mut recorder, Step::FinalStatus, true);
        let dir = path
            .join("bundles")
            .join(&recorder.intent.plan().request().operation_id)
            .join("payload");
        let module = dir.join("module.wasm");
        match mutation {
            0 => fs::write(&module, b"abcdefghik").unwrap(),
            1 => fs::write(dir.join("heap.bin"), []).unwrap(),
            2 => write_file(&dir.join("unexpected"), b"retained extra"),
            3 => fs::DirBuilder::new()
                .mode(0o700)
                .create(dir.join("unexpected"))
                .unwrap(),
            4 => fs::rename(&module, path.join("retained-module")).unwrap(),
            5 => {
                fs::rename(&module, path.join("retained-module")).unwrap();
                symlink(path.join("retained-module"), &module).unwrap();
            }
            6 => fs::hard_link(&module, path.join("retained-alias")).unwrap(),
            7 => {
                fs::rename(&dir, path.join("retained-payload")).unwrap();
                fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
            }
            8 => {
                fs::rename(&module, path.join("retained-module")).unwrap();
                fs::write(dir.join("heap.bin"), b"wrong").unwrap();
                // A later corrupt member must refuse before recreating an earlier missing one.
                assert!(recorder.materialize(&journal, &flow, true).is_err());
                assert!(!module.exists());
                assert_eq!(fs::read(dir.join("heap.bin")).unwrap(), b"wrong");
            }
            _ => unreachable!(),
        }
        let journal_path = directory(&path, &recorder).join("journal.bin");
        let original = fs::read(&journal_path).unwrap();
        assert!(
            recorder
                .project(&recorder.check().unwrap(), &recorder.intent)
                .is_err(),
            "{mutation}"
        );
        let id = recorder.intent.plan().request().operation_id.clone();
        drop(recorder);
        let reopened = CaptureOperation::open(&parent, &id).unwrap();
        assert!(
            reopened
                .project(&reopened.check().unwrap(), &reopened.intent)
                .is_err(),
            "{mutation}"
        );
        assert_eq!(fs::read(&journal_path).unwrap(), original);
    }
}
