//! One model consumer with real signatures and synthetic certificates; no IC or custody claim.

use super::super::*;
use crate::store::capture::{
    SnapshotReplyError,
    advance::StatusEvidence,
    intent::{
        CapturePlan,
        tests::{profile, signed_reply},
    },
    procedure::{Error as ProcedureError, Next, Procedure},
    status::tests::status,
    tests::signatures::{Scheme, fixture},
};
use ic_platform_protocol::management_snapshots::{
    CanisterArgs, Extent, LowMemoryHook, ReadDataArgs, ReadKind, Reserved, Snapshot, SnapshotArgs,
    SnapshotChunk, SnapshotData, SnapshotMetadata, SnapshotSource, TakeSnapshotArgs, Timer,
};

fn encode(value: &impl crate::store::capture::tests::wire::EncodeFixture) -> Vec<u8> {
    crate::store::capture::tests::wire::encode(value, 4096)
}

pub(in crate::store) fn selected() -> CapturePlan {
    let mut request = fixture(Scheme::Ed25519).0.request().clone();
    request.module_sha256 = BackupDigest::compute(b"abcdefghij");
    request.limits.extent_bytes = 4;
    request.limits.payload_bytes = 64;
    request.limits.evidence_bytes = allowance().evidence;
    request.limits.signing_requests = allowance().signatures;
    request.limits.network_requests = allowance().network;
    request.limits.observations = allowance().observations;
    request.trust_root = BackupDigest::compute(&profile().root_der);

    CapturePlan::new(request, profile()).unwrap()
}

fn allowance() -> Allowance {
    Allowance {
        manager_reply_bytes: None,
        signatures: 20,
        network: 20,
        observations: 0,
        evidence: 8 * 1024 * 1024,
        signer_network: 0,
        request_bytes: 4096,
        result_bytes: 4096,
    }
}

fn snapshot(id: u8) -> Snapshot {
    Snapshot {
        id: vec![id],
        taken_at_timestamp: 100,
        total_size: u64::MAX,
    }
}

fn metadata() -> SnapshotMetadata {
    SnapshotMetadata {
        source: Some(SnapshotSource::TakenFromCanister(Reserved)),
        taken_at_timestamp: 100,
        wasm_module_size: 10,
        wasm_memory_size: 5,
        stable_memory_size: 5,
        globals: vec![],
        canister_version: 41,
        certified_data: vec![1, 2],
        wasm_chunk_store: vec![b"chk".as_slice(), b""]
            .into_iter()
            .map(|bytes| SnapshotChunk {
                hash: BackupDigest::compute(bytes).as_bytes().to_vec(),
            })
            .collect(),
        global_timer: Some(Timer::Inactive),
        on_low_wasm_memory_hook_status: Some(LowMemoryHook::ConditionNotSatisfied),
    }
}

fn data() -> Vec<(ReadKind, Vec<u8>)> {
    let mut result = Vec::new();
    for (region, bytes) in [b"abcdefghij".as_slice(), b"heap!", b"state"]
        .into_iter()
        .enumerate()
    {
        for (n, bytes) in bytes.chunks(4).enumerate() {
            let (offset, size) = ((n * 4) as u64, bytes.len() as u64);
            let kind = match region {
                0 => ReadKind::WasmModule(Extent { offset, size }),
                1 => ReadKind::WasmMemory(Extent { offset, size }),
                _ => ReadKind::StableMemory(Extent { offset, size }),
            };
            result.push((kind, bytes.to_vec()));
        }
    }
    let mut chunks = [b"chk".as_slice(), b""];
    chunks.sort_by_key(|bytes| *BackupDigest::compute(bytes).as_bytes());
    result.extend(chunks.map(|bytes| {
        (
            ReadKind::WasmChunk(SnapshotChunk {
                hash: BackupDigest::compute(bytes).as_bytes().to_vec(),
            }),
            bytes.to_vec(),
        )
    }));

    result
}

pub(in crate::store) fn reply(plan: &CapturePlan, step: Step) -> Vec<u8> {
    match step {
        Step::InitialStatus | Step::FinalStatus => encode(&status(plan.plan())),
        Step::ListSnapshots => encode(&vec![snapshot(9)]),
        Step::TakeSnapshot => encode(&snapshot(8)),
        Step::Metadata => encode(&metadata()),
        Step::Data(n) => encode(&SnapshotData {
            chunk: data()[n as usize].1.clone(),
        }),
        _ => panic!("capture fixture received restore step"),
    }
}

pub(in crate::store) fn expected_request(
    plan: &CapturePlan,
    step: Step,
) -> (&'static str, Vec<u8>) {
    let canister_id = plan.plan().request().target.canister;

    match step {
        Step::InitialStatus | Step::FinalStatus => {
            ("canister_status", encode(&CanisterArgs { canister_id }))
        }
        Step::ListSnapshots => (
            "list_canister_snapshots",
            encode(&CanisterArgs { canister_id }),
        ),
        Step::TakeSnapshot => (
            "take_canister_snapshot",
            encode(&TakeSnapshotArgs {
                canister_id,
                replace_snapshot: None,
                uninstall_code: Some(false),
                sender_canister_version: None,
            }),
        ),
        Step::Metadata => (
            "read_canister_snapshot_metadata",
            encode(&SnapshotArgs {
                canister_id,
                snapshot_id: vec![8],
            }),
        ),
        Step::Data(n) => (
            "read_canister_snapshot_data",
            encode(&ReadDataArgs {
                canister_id,
                snapshot_id: vec![8],
                kind: data()[n as usize].0.clone(),
            }),
        ),
        _ => panic!("capture fixture received restore step"),
    }
}

fn reference(bytes: &[u8]) -> ReferenceRecord {
    ReferenceRecord {
        bytes: bytes.len().try_into().unwrap(),
        sha256: *BackupDigest::compute(bytes).as_bytes(),
    }
}

fn append(bytes: &mut Vec<u8>, plan: &CapturePlan, fact: FactRecord) -> Journal {
    let journal = Journal::open(bytes.as_slice(), plan).unwrap();
    bytes.extend(journal.candidate(fact).unwrap());

    Journal::open(bytes.as_slice(), plan).unwrap()
}

fn through<'a>(plan: &'a CapturePlan, stop: Step) -> Procedure<'a> {
    let mut flow = Procedure::new(plan).unwrap();

    loop {
        match flow.next().unwrap() {
            Next::Call(step) if step == stop => return flow,
            Next::Call(step) => flow
                .accept(
                    false,
                    step,
                    if step == Step::InitialStatus {
                        [1; 32]
                    } else {
                        [2; 32]
                    },
                    &reply(plan, step),
                )
                .unwrap(),
            Next::Materialize => flow.after_materialization().unwrap(),
            Next::Publish => panic!("requested stop was not reached"),
        }
    }
}

#[test]
fn one_consumer_derives_all_capture_requests_and_replays_the_same_policy() {
    let plan = selected();
    let mut bytes = Journal::create(&plan).unwrap();
    let mut flow = Procedure::new(&plan).unwrap();
    let mut replies = Vec::new();
    loop {
        match flow.next().unwrap() {
            Next::Call(step) => {
                let (method, args) = flow.request().unwrap();
                let expected = expected_request(&plan, step);
                assert_eq!(method, expected.0);
                assert_eq!(args, expected.1);
                let response = reply(&plan, step);
                let (prepared, evidence) = signed_reply(
                    &plan,
                    u8::try_from(replies.len()).unwrap(),
                    method,
                    args,
                    response,
                );
                let request_bytes = prepared.envelope.clone();
                let result_bytes = evidence.encode(plan.profile().result_bytes).unwrap();
                append(
                    &mut bytes,
                    &plan,
                    FactRecord::CallReserved {
                        step,
                        reference_ns: 100,
                    },
                );
                append(
                    &mut bytes,
                    &plan,
                    FactRecord::CallPrepared {
                        step,
                        request: reference(&request_bytes),
                    },
                );
                append(&mut bytes, &plan, FactRecord::CallSubmitted { step });
                append(
                    &mut bytes,
                    &plan,
                    FactRecord::CallResult {
                        step,
                        result: reference(&result_bytes),
                    },
                );
                flow.accept_authenticated(step, 100, &prepared, &evidence)
                    .unwrap();
                replies.push((step, request_bytes, result_bytes));
            }
            Next::Materialize => {
                assert!(flow.request().is_err());
                append(&mut bytes, &plan, FactRecord::MaterializationStarted);
                // No files are written by this policy test. The real completion
                // owner must verify materialization before this transition.
                flow.after_materialization().unwrap();
            }
            Next::Publish => {
                assert!(flow.request().is_err());
                append(
                    &mut bytes,
                    &plan,
                    FactRecord::PublishReady {
                        completed_at_ns: 200,
                    },
                );
                break;
            }
        }
    }
    let original = Journal::open(bytes.as_slice(), &plan).unwrap();
    assert_eq!(
        (original.signatures, original.network, original.observations),
        (14, 14, 0)
    );
    assert_eq!(original.completed_at_ns, Some(200));
    assert_eq!(original.count, 59); // 14 four-fact calls, local intent, publication, creation.
    let reopened = CapturePlan::decode(plan.encoded()).unwrap();
    let mut replay = Procedure::new(&reopened).unwrap();
    for (step, request_bytes, result_bytes) in replies {
        let prepared = reopened.view().decode_request(100, request_bytes).unwrap();
        let evidence = StatusEvidence::decode(&result_bytes).unwrap();
        if replay.next().unwrap() == Next::Materialize {
            replay.after_materialization().unwrap();
        }
        assert_eq!(replay.next().unwrap(), Next::Call(step));
        assert_eq!(replay.request().unwrap(), expected_request(&plan, step));
        replay
            .accept_authenticated(step, 100, &prepared, &evidence)
            .unwrap();
    }
    assert_eq!(replay.next().unwrap(), Next::Publish);
    assert_eq!(Journal::open(bytes.as_slice(), &plan).unwrap(), original);
}

#[test]
fn invalid_settled_payload_stays_spent_and_never_unlocks_another_call() {
    let plan = selected();
    let mut flow = Procedure::new(&plan).unwrap();
    let mut bytes = Journal::create(&plan).unwrap();
    let mut running = status(plan.plan());
    running.status = ic_platform_protocol::management_snapshots::Status::Running;
    let response = encode(&running);
    let step = Step::InitialStatus;
    append(
        &mut bytes,
        &plan,
        FactRecord::CallReserved {
            step,
            reference_ns: 100,
        },
    );
    append(
        &mut bytes,
        &plan,
        FactRecord::CallPrepared {
            step,
            request: reference(&flow.request().unwrap().1),
        },
    );
    append(&mut bytes, &plan, FactRecord::CallSubmitted { step });
    let spent = append(
        &mut bytes,
        &plan,
        FactRecord::CallResult {
            step,
            result: reference(&response),
        },
    );
    assert!(matches!(
        flow.accept(false, step, [1; 32], &response),
        Err(ProcedureError::Status(_))
    ));
    assert!(matches!(flow.next(), Err(ProcedureError::Order)));
    assert!(matches!(
        flow.accept(false, step, [2; 32], &reply(&plan, step)),
        Err(ProcedureError::Order)
    ));
    assert_eq!(Journal::open(bytes.as_slice(), &plan).unwrap(), spent);
    assert_eq!((spent.signatures, spent.network), (1, 1));
    assert!(spent.calls[0].update.result.is_some());
    let mut reopened = Procedure::new(&plan).unwrap();
    assert!(matches!(
        reopened.accept(false, step, [1; 32], &response),
        Err(ProcedureError::Status(_))
    ));
}

#[test]
fn snapshot_data_and_final_status_prerequisites_are_not_optional_phases() {
    let plan = selected();
    let mut invalid = profile();
    invalid.snapshot_capacity = 0;
    assert!(CapturePlan::new(plan.plan().request().clone(), invalid).is_err());
    let mut flow = through(&plan, Step::ListSnapshots);
    assert!(matches!(
        flow.accept(
            false,
            Step::ListSnapshots,
            [2; 32],
            &encode(&vec![snapshot(7), snapshot(9)])
        ),
        Err(ProcedureError::Snapshot(SnapshotReplyError::NoCapacity))
    ));
    let mut flow = through(&plan, Step::TakeSnapshot);
    assert!(matches!(
        flow.accept(false, Step::TakeSnapshot, [2; 32], &encode(&snapshot(9))),
        Err(ProcedureError::Snapshot(SnapshotReplyError::AlreadyExisted))
    ));
    let mut flow = through(&plan, Step::Metadata);
    let mut m = metadata();
    m.taken_at_timestamp += 1;
    assert!(matches!(
        flow.accept(false, Step::Metadata, [2; 32], &encode(&m)),
        Err(ProcedureError::SnapshotTimestamp)
    ));
    let mut flow = through(&plan, Step::Data(0));
    assert!(flow.after_materialization().is_err());
    assert!(matches!(
        flow.accept(
            false,
            Step::Data(0),
            [2; 32],
            &encode(&SnapshotData { chunk: vec![0] })
        ),
        Err(ProcedureError::Data(_))
    ));
    let mut flow = through(&plan, Step::Data(1));
    assert!(matches!(
        flow.accept(
            false,
            Step::FinalStatus,
            [2; 32],
            &reply(&plan, Step::FinalStatus)
        ),
        Err(ProcedureError::Order)
    ));
    let mut flow = through(&plan, Step::Data(1));
    assert!(matches!(
        flow.accept(false, Step::Data(0), [2; 32], &reply(&plan, Step::Data(1))),
        Err(ProcedureError::Order)
    ));
    let mut flow = through(&plan, Step::FinalStatus);
    assert!(matches!(
        flow.accept(
            false,
            Step::FinalStatus,
            [1; 32],
            &reply(&plan, Step::FinalStatus)
        ),
        Err(ProcedureError::ReusedStatus)
    ));
    assert!(flow.request().is_err());
    let mut flow = through(&plan, Step::FinalStatus);
    let mut running = status(plan.plan());
    running.status = ic_platform_protocol::management_snapshots::Status::Running;
    assert!(matches!(
        flow.accept(false, Step::FinalStatus, [3; 32], &encode(&running)),
        Err(ProcedureError::Status(_))
    ));
    assert!(flow.next().is_err());
}

#[test]
fn whole_module_hash_is_required_after_all_valid_length_replies() {
    let plan = selected();
    let mut flow = through(&plan, Step::Data(0));

    loop {
        let Next::Call(Step::Data(n)) = flow.next().unwrap() else {
            panic!("must not materialize corrupted module")
        };
        let mut bytes = data()[n as usize].1.clone();
        if n == 0 {
            bytes[0] ^= 1;
        }
        let result = flow.accept(
            false,
            Step::Data(n),
            [2; 32],
            &encode(&SnapshotData { chunk: bytes }),
        );
        if n == 8 {
            assert!(matches!(result, Err(ProcedureError::ModuleHash)));
            assert!(flow.next().is_err());
            break;
        }
        result.unwrap();
    }
}

#[test]
fn journal_reopen_cannot_replace_original_profile_or_allowance() {
    let intent = selected();
    let bytes = Journal::create(&intent).unwrap();
    let journal = Journal::open(bytes.as_slice(), &intent).unwrap();
    assert_eq!(journal.allowance, allowance());
    let mut request = intent.plan().request().clone();
    request.limits.signing_requests += 1;
    let renewed = CapturePlan::new(request, intent.profile().clone()).unwrap();
    assert!(Journal::open(bytes.as_slice(), &renewed).is_err());
    let mut changed = intent.profile().clone();
    changed.certificate_max_age_ns += 1;
    let changed = CapturePlan::new(intent.plan().request().clone(), changed).unwrap();
    assert!(Journal::open(bytes.as_slice(), &changed).is_err());
}
