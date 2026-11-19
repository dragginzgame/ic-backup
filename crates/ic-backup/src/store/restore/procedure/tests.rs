//! Re-authored fixed-policy cases from legacy 4ae087fad8c23de712aa60bf569bde4b2b932651,
//! workflow/ic_snapshot_restore/tests/mod.rs::failure_boundaries_retain_pending_or_applied_and_never_repeat_provider; actual effects require qualification.

use super::*;
use crate::store::{
    capture::{
        intent::tests::profile,
        tests::{
            completion::{complete, settled},
            source::maximum,
            wire::frozen,
        },
    },
    restore::tests::fixtures::declaration,
};
use ic_platform_protocol::management_snapshots::{
    Reserved, SnapshotAllocation, SnapshotSource, Status, fixtures,
};

fn allocation(value: &SnapshotAllocation) -> Vec<u8> {
    fixtures::snapshot_allocation(value, 4096).unwrap()
}

fn encoded(value: &impl crate::store::capture::tests::wire::EncodeFixture) -> Vec<u8> {
    crate::store::capture::tests::wire::encode(value, 2 * 1024 * 1024)
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
        Step::AllocateSnapshot => allocation(&SnapshotAllocation {
            snapshot_id: vec![10],
        }),
        Step::Upload(_) | Step::LoadSnapshot => fixtures::empty_response(4096).unwrap(),
        Step::ReadbackMetadata => {
            let mut metadata = source.metadata.clone();
            metadata.source = Some(SnapshotSource::MetadataUpload(Reserved));
            metadata.taken_at_timestamp += 1;
            metadata.canister_version += 1;
            encoded(&metadata)
        }
        Step::Readback(n) => encoded(&ic_platform_protocol::management_snapshots::SnapshotData {
            chunk: source
                .payload
                .read_data(&flow.source_request(n).unwrap())
                .unwrap(),
        }),
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
                *crate::store::BackupDigest::compute(format!("{step:?}").as_bytes()).as_bytes();
            let inject = match (fault, step) {
                ("reused-initial-status", Step::BeforeLoadStatus) => {
                    request_id =
                        *crate::store::BackupDigest::compute(b"RestoreInitialStatus").as_bytes();
                    true
                }
                ("reused-preload-status", Step::AfterLoadStatus) => {
                    request_id =
                        *crate::store::BackupDigest::compute(b"BeforeLoadStatus").as_bytes();
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
                        status.status = Status::Running;
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
                    bytes = allocation(&SnapshotAllocation {
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
                    bytes = frozen("unit-u8-argument");
                    true
                }
                ("metadata-origin", Step::ReadbackMetadata) => {
                    bytes = encoded(&source.metadata);
                    true
                }
                ("readback-bytes", Step::Readback(0)) => {
                    bytes = encoded(&ic_platform_protocol::management_snapshots::SnapshotData {
                        chunk: vec![255u8; 3],
                    });
                    true
                }
                ("order", Step::Upload(0))
                | ("early-load" | "skip-readback", Step::Readback(0))
                | ("repeat-readback", Step::Readback(1)) => true,
                ("short-readback" | "long-readback", Step::Readback(0)) => {
                    bytes = encoded(&ic_platform_protocol::management_snapshots::SnapshotData {
                        chunk: vec![0; if fault == "short-readback" { 2 } else { 4 }],
                    });
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
        assert!(decode_snapshot_allocation(&bytes, limits()).is_err());
    }
}

#[test]
fn actual_restore_cursor_owns_all_regions_chunks_routing_and_incomplete_coverage() {
    // Replaces the retired ephemeral UploadedSnapshotReadback fixture. Geometry,
    // maximum reply bounds and source block/hash/length faults retain their owning tests.
    let (mut fixture, _, mut connection) = crate::store::capture::tests::completion::mixed();
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
                *crate::store::BackupDigest::compute(format!("{step:?}").as_bytes()).as_bytes(),
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
                let expected = [
                    frozen("restore-readback-0"),
                    frozen("restore-readback-1"),
                    frozen("restore-readback-2"),
                    frozen("restore-readback-3"),
                    frozen("restore-readback-4"),
                    frozen("restore-readback-5"),
                    frozen("restore-readback-6"),
                ];
                assert_eq!(
                    args, expected[ordinal as usize],
                    "original readback ordinal {ordinal}"
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
                bytes =
                    encoded(&ic_platform_protocol::management_snapshots::SnapshotData { chunk });
            }
            let result = flow.accept(
                step,
                false,
                *crate::store::BackupDigest::compute(format!("{step:?}").as_bytes()).as_bytes(),
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

#[test]
fn protocol_allocation_bounds_and_exact_empty_results_preserve_legacy_refusals() {
    // Re-authored from legacy 4ae087fad8c23de712aa60bf569bde4b2b932651,
    // model/ic_snapshot_upload/tests/mod.rs::bounded_reply_identity_exact_empty_tuple_and_wire_extensions_reject.
    // Empty/reused/source IDs remain policy checks in the fixed-procedure matrix.
    for length in [1, 256] {
        let id = vec![21; length];
        let raw = allocation(&SnapshotAllocation {
            snapshot_id: id.clone(),
        });
        assert_eq!(
            decode_snapshot_allocation(&raw, limits())
                .unwrap()
                .snapshot_id,
            id
        );
        for end in 0..raw.len() {
            assert!(decode_snapshot_allocation(&raw[..end], limits()).is_err());
        }
        assert_eq!(
            decode_snapshot_allocation(&[raw.as_slice(), &[0]].concat(), limits()),
            Err(DecodeError::Trailing)
        );
    }
    let excessive = allocation(&SnapshotAllocation {
        snapshot_id: vec![1; 257],
    });
    assert_eq!(
        decode_snapshot_allocation(&excessive, limits()),
        Err(DecodeError::Limit)
    );
    assert!(unit(b"DIDL\0\0").is_ok());
    assert!(matches!(
        unit(b"DIDL\0\0\0"),
        Err(Error::Reply(DecodeError::Trailing))
    ));
    assert!(matches!(
        unit(&frozen("unit-null-argument")),
        Err(Error::Reply(DecodeError::Arity))
    ));
}
