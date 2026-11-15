//! Local completion, retained crash images and inventory consumers.
use super::*;
use crate::store::{BackupSummary, CaptureCompletionError, ScanLimits};
use ic_host_artifacts::artifact::Sha256Digest;

fn settled() -> (Fixture, Server, HttpProvider) {
    let mut f = assembled(20);
    let r = request(&f, false);
    let e = result(&f, &r, false);
    let (server, mut p) = provider(&f, r, &e);
    let a = f.a.call.clone();
    assert_eq!(
        advance(&mut f, &a, &mut p).unwrap(),
        StatusProgress::Verified
    );
    p.replay = true;
    (f, server, p)
}
fn complete(
    f: &mut Fixture,
    p: &mut HttpProvider,
) -> Result<BackupSummary, CaptureCompletionError<StatusHttpError>> {
    runtime().block_on(f.op.complete_capture(&f.t, p))
}
fn manifest(f: &Fixture) -> PathBuf {
    f.root
        .join("bundles")
        .join(&f.p.request().operation_id)
        .join("manifest.json")
}
fn limits() -> ScanLimits {
    ScanLimits {
        entries: 10.try_into().unwrap(),
        bytes: (1024 * 1024).try_into().unwrap(),
        results: 10.try_into().unwrap(),
    }
}

#[test]
fn completion_binds_original_bytes_and_existing_inventory_and_payload_consumers() {
    let (mut f, server, mut p) = settled();
    let summary = complete(&mut f, &mut p).unwrap();
    let bytes = fs::read(manifest(&f)).unwrap();
    let ledger = fs::read(f.ledger()).unwrap();
    let record: crate::store::manifest::ManifestRecord = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(summary.payload_bytes, 10);
    assert_eq!(summary.captured_at_ns, 55);
    assert!(summary.completed_at_ns > 0);
    assert_eq!(summary.target, f.p.request().target);
    assert_eq!(summary.module_sha256, f.p.request().module_sha256);
    assert_eq!(summary.manifest_sha256, Sha256Digest::compute(&bytes));
    assert_eq!(record.ledger.bytes, ledger.len() as u64);
    assert_eq!(
        record.ledger.sha256,
        Sha256Digest::compute(&ledger).to_string()
    );
    assert_eq!(
        record.plan.sha256,
        Sha256Digest::compute(f.p.encoded()).to_string()
    );
    assert_eq!(record.plan.bytes, f.p.encoded().len() as u64);
    assert_eq!(
        f.store.list_backups(None, limits()).unwrap(),
        vec![summary.clone()]
    );
    assert_eq!(
        f.store
            .latest_completed_backup(&summary.target, limits())
            .unwrap(),
        Some(summary.clone())
    );
    assert_eq!(
        f.store
            .verify_payload(
                &summary.bundle_id,
                summary.manifest_sha256,
                10.try_into().unwrap()
            )
            .unwrap(),
        summary
    );
    assert!(
        f.op.check()
            .unwrap()
            .candidate(&EventRecord::AssemblyStarted, &f.p)
            .is_err()
    );
    let mut f = f.reopen();
    assert_eq!(complete(&mut f, &mut p).unwrap(), summary);
    assert_eq!(fs::read(manifest(&f)).unwrap(), bytes);
    assert_eq!(fs::read(f.ledger()).unwrap(), ledger);
    assert_eq!(server.count(), 1);
    assert!(!manifest(&f).with_file_name("manifest.pending").exists());
}

#[test]
fn completion_requires_successful_final_status_before_freezing_or_publication() {
    for running in [false, true] {
        let mut f = assembled(20);
        let r = request(&f, false);
        let e = result(&f, &r, running);
        let (server, mut p) = provider(&f, r, &e);
        let a = f.a.call.clone();
        if running {
            assert!(advance(&mut f, &a, &mut p).is_err());
        }
        let before = fs::read(f.ledger()).unwrap();
        let refusal = complete(&mut f, &mut p).unwrap_err();
        if running {
            assert!(matches!(refusal, CaptureCompletionError::Status(_)));
        } else {
            assert!(matches!(
                refusal,
                CaptureCompletionError::FinalStatusRequired
            ));
        }
        assert_eq!(fs::read(f.ledger()).unwrap(), before);
        assert!(!manifest(&f).exists());
        assert!(f.op.check().unwrap().completed_at_ns.is_none());
        assert_eq!(server.count(), usize::from(running));
    }
}

#[test]
fn complete_staging_or_missing_unpublished_manifest_resumes_original_receipt() {
    for staging in [true, false] {
        let (mut f, server, mut p) = settled();
        let summary = complete(&mut f, &mut p).unwrap();
        let bytes = fs::read(manifest(&f)).unwrap();
        let ledger = fs::read(f.ledger()).unwrap();
        // Retained images of a crash after PublishReady, before create-only publication.
        if staging {
            fs::rename(
                manifest(&f),
                manifest(&f).with_file_name("manifest.pending"),
            )
            .unwrap();
        } else {
            fs::remove_file(manifest(&f)).unwrap();
        }
        assert!(f.store.list_backups(None, limits()).unwrap().is_empty());
        let mut f = f.reopen();
        assert_eq!(complete(&mut f, &mut p).unwrap(), summary);
        assert_eq!(fs::read(manifest(&f)).unwrap(), bytes);
        assert_eq!(fs::read(f.ledger()).unwrap(), ledger);
        assert_eq!(server.count(), 1);
    }
}

#[test]
fn partial_changed_or_duplicate_manifests_are_retained_without_repair() {
    for fault in ["partial", "changed", "duplicate", "whitespace"] {
        let (mut f, server, mut p) = settled();
        complete(&mut f, &mut p).unwrap();
        let ledger = fs::read(f.ledger()).unwrap();
        let path = if fault == "partial" {
            let pending = manifest(&f).with_file_name("manifest.pending");
            fs::rename(manifest(&f), &pending).unwrap();
            pending
        } else {
            manifest(&f)
        };
        let mut bytes = fs::read(&path).unwrap();
        match fault {
            "partial" => {
                bytes.truncate(bytes.len() / 2);
                fs::write(&path, &bytes).unwrap();
            }
            "changed" => {
                bytes[0] ^= 1;
                fs::write(&path, &bytes).unwrap();
            }
            "whitespace" => {
                bytes.push(b'\n');
                fs::write(&path, &bytes).unwrap();
            }
            "duplicate" => {
                fs::copy(&path, path.with_file_name("manifest.pending")).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(matches!(
            complete(&mut f, &mut p),
            Err(CaptureCompletionError::Capture(CaptureError::Changed))
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(fs::read(f.ledger()).unwrap(), ledger);
        let Fixture { op, store, p, .. } = f;
        drop(op);
        assert!(
            store
                .open_capture(&p.request().operation_id, p.digest())
                .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), bytes);
        assert_eq!(server.count(), 1);
    }
}

#[test]
fn uncertain_publish_ready_is_spent_once_and_partial_tail_refuses() {
    for partial in [false, true] {
        let (mut f, server, mut p) = settled();
        let event = EventRecord::PublishReady {
            completed_at_ns: 42,
        };
        assert_eq!(
            serde_json::to_vec(&event).unwrap(),
            b"{\"kind\":\"publish_ready\",\"completed_at_ns\":42}"
        );
        assert!(
            f.op.commit_with(&event, None, |file, bytes| {
                file.write_all(if partial {
                    &bytes[..bytes.len() / 2]
                } else {
                    bytes
                })?;
                file.sync_all()?;
                Err(std::io::Error::other("uncertain publication readiness"))
            })
            .is_err()
        );
        let before = fs::read(f.ledger()).unwrap();
        if partial {
            let path = f.ledger();
            let Fixture { op, store, p, .. } = f;
            drop(op);
            assert!(
                store
                    .open_capture(&p.request().operation_id, p.digest())
                    .is_err()
            );
            assert_eq!(fs::read(path).unwrap(), before);
        } else {
            let mut f = f.reopen();
            assert_eq!(complete(&mut f, &mut p).unwrap().completed_at_ns, 42);
            assert_eq!(fs::read(f.ledger()).unwrap(), before);
        }
        assert_eq!(server.count(), 1);
    }
}

#[test]
fn publication_capacity_is_reserved_before_freezing_the_ledger_or_creating_manifest() {
    let mut f = Fixture::with_limits(false, false, 20, 7_000_000);
    for n in 0..2 {
        f.advance(n).unwrap();
    }
    runtime()
        .block_on(f.op.assemble_data(&f.t, &mut f.provider))
        .unwrap();
    let r = request(&f, false);
    let e = result(&f, &r, false);
    let (server, mut p) = provider(&f, r, &e);
    let a = f.a.call.clone();
    advance(&mut f, &a, &mut p).unwrap();
    p.replay = true;
    let before = fs::read(f.ledger()).unwrap();
    assert!(matches!(
        complete(&mut f, &mut p),
        Err(CaptureCompletionError::Capture(CaptureError::Ledger(_)))
    ));
    assert_eq!(fs::read(f.ledger()).unwrap(), before);
    assert!(f.op.check().unwrap().completed_at_ns.is_none());
    assert!(!manifest(&f).exists());
    assert!(!manifest(&f).with_file_name("manifest.pending").exists());
    assert_eq!(server.count(), 1);
}
