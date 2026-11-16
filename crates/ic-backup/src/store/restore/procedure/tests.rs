//! Re-authored fixed-policy cases from legacy 4ae087fad8c23de712aa60bf569bde4b2b932651,
//! workflow/ic_snapshot_restore/tests/mod.rs::failure_boundaries_retain_pending_or_applied_and_never_repeat_provider; actual effects require qualification.
use super::*;
use crate::store::{
    capture::{
        intent::tests::profile,
        test_fixtures::{
            completion::{complete, settled},
            source::maximum,
        },
    },
    restore::tests::fixtures::declaration,
};
use ic_platform::{
    candid_codec::{Reserved, arguments::encode_args},
    management_canister_types::{CanisterStatusType, SnapshotSource},
};
fn encoded(value: &impl Encode) -> Vec<u8> {
    candid_codec::encode_one(value, 2 * 1024 * 1024).unwrap()
}
fn response(flow: &Procedure<'_>, source: &mut Source<'_>, step: Step) -> Vec<u8> {
    match step {
        Step::RestoreInitialStatus | Step::BeforeLoadStatus | Step::AfterLoadStatus => {
            let mut status =
                crate::store::capture::status::tests::status(source.capture.plan().plan());
            status.settings.controllers = vec![flow.plan.request().sender];
            encoded(&status)
        }
        Step::RestoreListSnapshots => encoded(&vec![Snapshot {
            id: vec![9],
            taken_at_timestamp: 1,
            total_size: 0,
        }]),
        Step::AllocateSnapshot => encoded(&UploadCanisterSnapshotMetadataResult {
            snapshot_id: vec![10],
        }),
        Step::Upload(_) | Step::LoadSnapshot => encode_args(&(), 4096).unwrap(),
        Step::ReadbackMetadata => {
            let mut metadata = source.metadata.clone();
            metadata.source = Some(SnapshotSource::MetadataUpload(Reserved));
            metadata.taken_at_timestamp += 1;
            metadata.canister_version += 1;
            encoded(&metadata)
        }
        Step::Readback(n) => encoded(
            &ic_platform::management_canister_types::ReadCanisterSnapshotDataResult {
                chunk: source
                    .payload
                    .read_data(&flow.source_request(n).unwrap())
                    .unwrap(),
            },
        ),
        _ => panic!("unexpected capture step"),
    }
}
#[test]
fn exact_restore_policy_and_each_distinct_failure_boundary() {
    let (mut fixture, _, mut connection) = settled();
    let receipt = complete(&mut fixture, &mut connection).unwrap();
    let selected = fixture
        .store
        .open_capture_source(&mut fixture.op, &receipt, maximum())
        .unwrap();
    let mut direct = profile();
    direct.request_bytes = 8192;
    let plan = RestorePlan::new(&selected, declaration(&selected), direct).unwrap();
    let (payload, metadata) = selected.into_parts();
    let mut source = Source {
        capture: fixture.op,
        payload,
        metadata,
    };
    for fault in [
        "none",
        "rejected",
        "running",
        "wrong-controller",
        "capacity",
        "empty-allocation",
        "long-allocation",
        "old-allocation",
        "source-allocation",
        "unit-arity",
        "metadata-origin",
        "readback-bytes",
        "order",
        "early-load",
        "repeat-readback",
        "skip-readback",
        "short-readback",
        "long-readback",
        "reused-initial-status",
        "reused-preload-status",
    ] {
        let mut flow = Procedure::new(&plan, &source).unwrap();
        let mut failed = false;
        while let Some(step) = flow.next().unwrap() {
            let _request = flow.request(&mut source).unwrap();
            let mut bytes = response(&flow, &mut source, step);
            let mut rejected = false;
            let mut request_id =
                *ic_host_artifacts::artifact::Sha256Digest::compute(format!("{step:?}").as_bytes())
                    .as_bytes();
            let inject = match (fault, step) {
                ("reused-initial-status", Step::BeforeLoadStatus) => {
                    request_id = *ic_host_artifacts::artifact::Sha256Digest::compute(
                        b"RestoreInitialStatus",
                    )
                    .as_bytes();
                    true
                }
                ("reused-preload-status", Step::AfterLoadStatus) => {
                    request_id =
                        *ic_host_artifacts::artifact::Sha256Digest::compute(b"BeforeLoadStatus")
                            .as_bytes();
                    true
                }
                ("rejected", Step::LoadSnapshot) => {
                    rejected = true;
                    true
                }
                ("running" | "wrong-controller", Step::BeforeLoadStatus) => {
                    let mut status =
                        crate::store::capture::status::tests::status(source.capture.plan().plan());
                    status.settings.controllers = vec![plan.request().sender];
                    if fault == "running" {
                        status.status = CanisterStatusType::Running;
                    } else {
                        status.settings.controllers.clear();
                    }
                    bytes = encoded(&status);
                    true
                }
                ("capacity", Step::RestoreListSnapshots) => {
                    bytes = encoded(&vec![
                        Snapshot {
                            id: vec![1],
                            taken_at_timestamp: 1,
                            total_size: 0,
                        },
                        Snapshot {
                            id: vec![2],
                            taken_at_timestamp: 1,
                            total_size: 0,
                        },
                    ]);
                    true
                }
                (
                    "empty-allocation" | "long-allocation" | "old-allocation" | "source-allocation",
                    Step::AllocateSnapshot,
                ) => {
                    bytes = encoded(&UploadCanisterSnapshotMetadataResult {
                        snapshot_id: match fault {
                            "empty-allocation" => vec![],
                            "long-allocation" => vec![1; 257],
                            "old-allocation" => vec![9],
                            _ => vec![8],
                        },
                    });
                    true
                }
                ("unit-arity", Step::Upload(0)) => {
                    bytes = encoded(&0u8);
                    true
                }
                ("metadata-origin", Step::ReadbackMetadata) => {
                    bytes = encoded(&source.metadata);
                    true
                }
                ("readback-bytes", Step::Readback(0)) => {
                    bytes = encoded(
                        &ic_platform::management_canister_types::ReadCanisterSnapshotDataResult {
                            chunk: vec![255u8; 3],
                        },
                    );
                    true
                }
                ("order", Step::Upload(0))
                | ("early-load" | "skip-readback", Step::Readback(0))
                | ("repeat-readback", Step::Readback(1)) => true,
                ("short-readback" | "long-readback", Step::Readback(0)) => {
                    bytes = encoded(
                        &ic_platform::management_canister_types::ReadCanisterSnapshotDataResult {
                            chunk: vec![0; if fault == "short-readback" { 2 } else { 4 }],
                        },
                    );
                    true
                }
                _ => false,
            };
            let actual = match (fault, inject) {
                ("order", true) => Step::Upload(1),
                ("early-load", true) => Step::LoadSnapshot,
                ("repeat-readback", true) => Step::Readback(0),
                ("skip-readback", true) => Step::Readback(1),
                _ => step,
            };
            let result = flow.accept(actual, rejected, request_id, &bytes, &mut source);
            if inject {
                assert!(result.is_err(), "{fault}");
                assert!(flow.next().is_err(), "failed policy must remain blocked");
                failed = true;
                break;
            }
            result.unwrap();
        }
        assert_eq!(failed, fault != "none", "{fault}");
    }
    // Oversized and malformed allocation/unit inputs refuse before unbounded decoding.
    for bytes in [vec![], vec![0; 4097], b"DIDL\x00\x00trailing".to_vec()] {
        assert!(unit(&bytes).is_err());
        assert!(decode_one_with_skip_limit::<Allocation>(&bytes, limits(), 0).is_err());
    }
}

#[test]
fn actual_restore_cursor_owns_all_regions_chunks_routing_and_incomplete_coverage() {
    // Replaces the retired ephemeral UploadedSnapshotReadback fixture. Geometry,
    // maximum reply bounds and source block/hash/length faults retain their owning tests.
    let (mut fixture, _, mut connection) =
        crate::store::capture::test_fixtures::completion::mixed();
    let receipt = complete(&mut fixture, &mut connection).unwrap();
    let selected = fixture
        .store
        .open_capture_source(
            &mut fixture.op,
            &receipt,
            receipt.payload_bytes.try_into().unwrap(),
        )
        .unwrap();
    let mut request = declaration(&selected);
    request.limits.payload_bytes = receipt.payload_bytes;
    request.limits.extent_bytes = 5;
    request.limits.signing_requests = 64;
    request.limits.network_requests = 64;
    let mut direct = profile();
    direct.request_bytes = 8192;
    let plan = RestorePlan::new(&selected, request, direct).unwrap();
    let (payload, metadata) = selected.into_parts();
    let mut source = Source {
        capture: fixture.op,
        payload,
        metadata,
    };
    let calls = source.transfer(&plan).unwrap().minimum_management_calls();
    for prefix in 0..=calls {
        let mut flow = Procedure::new(&plan, &source).unwrap();
        for _ in 0..prefix {
            let step = flow.next().unwrap().unwrap();
            let bytes = response(&flow, &mut source, step);
            flow.accept(
                step,
                false,
                *ic_host_artifacts::artifact::Sha256Digest::compute(format!("{step:?}").as_bytes())
                    .as_bytes(),
                &bytes,
                &mut source,
            )
            .unwrap();
        }
        assert_eq!(
            flow.next().unwrap().is_none(),
            prefix == calls,
            "every incomplete prefix, including the empty chunk, stays incomplete"
        );
    }
    let n = source.transfer(&plan).unwrap().data_calls();
    for corrupt in 0..n {
        let mut flow = Procedure::new(&plan, &source).unwrap();
        while let Some(step) = flow.next().unwrap() {
            let (method, args) = flow.request(&mut source).unwrap();
            if let Step::Readback(ordinal) = step {
                assert_eq!(method, "read_canister_snapshot_data");
                let request: ReadCanisterSnapshotDataArgs =
                    decode_one_with_skip_limit(&args, limits(), 0).unwrap();
                assert_eq!(request.snapshot_id, [10]);
                assert_eq!(request.canister_id, plan.request().target.canister);
                assert_eq!(
                    encoded(&request.kind),
                    encoded(&flow.source_request(ordinal).unwrap().kind)
                );
            }
            let mut bytes = response(&flow, &mut source, step);
            if step == Step::Readback(corrupt) {
                let mut chunk = source
                    .payload
                    .read_data(&flow.source_request(corrupt).unwrap())
                    .unwrap();
                if let Some(byte) = chunk.first_mut() {
                    *byte ^= 1;
                } else {
                    chunk.push(1);
                }
                bytes = encoded(
                    &ic_platform::management_canister_types::ReadCanisterSnapshotDataResult {
                        chunk,
                    },
                );
            }
            let result = flow.accept(
                step,
                false,
                *ic_host_artifacts::artifact::Sha256Digest::compute(format!("{step:?}").as_bytes())
                    .as_bytes(),
                &bytes,
                &mut source,
            );
            if step == Step::Readback(corrupt) {
                assert!(matches!(result, Err(Error::Readback)));
                assert!(flow.next().is_err());
                break;
            }
            result.unwrap();
        }
    }
}
