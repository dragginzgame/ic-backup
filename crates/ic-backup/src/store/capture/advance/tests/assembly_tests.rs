//! Original filesystem regressions informed by frozen legacy artifact custody tests.
use super::*;
use crate::store::{
    CaptureDataAdvanceError,
    capture::{
        advance::tests::fixtures::{data::Fixture, http::runtime},
        ledger::EventRecord,
    },
};
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    path::PathBuf,
};

fn acquired() -> Fixture {
    let mut f = Fixture::new(false, false);
    for n in 0..2 {
        f.advance(n).unwrap();
    }
    f
}
fn payload(f: &Fixture) -> PathBuf {
    f.root
        .join("bundles")
        .join(&f.p.request().operation_id)
        .join("payload")
}
fn assemble(
    f: &mut Fixture,
) -> Result<u64, CaptureDataAdvanceError<StatusHttpError, std::convert::Infallible>> {
    runtime().block_on(f.op.assemble_data(&f.t, &mut f.provider))
}
fn seed(f: &mut Fixture, bytes: &[u8]) {
    f.op.commit(&EventRecord::AssemblyStarted, None).unwrap();
    fs::create_dir(payload(f).parent().unwrap()).unwrap();
    fs::set_permissions(
        payload(f).parent().unwrap(),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    fs::create_dir(payload(f)).unwrap();
    fs::set_permissions(payload(f), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(payload(f).join("module.wasm"), bytes).unwrap();
    fs::set_permissions(
        payload(f).join("module.wasm"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
}

#[test]
fn assembly_retains_exact_files_and_replays_offline_without_journal_writes() {
    let mut f = acquired();
    assert_eq!(assemble(&mut f).unwrap(), 10);
    assert_eq!(
        fs::read(payload(&f).join("module.wasm")).unwrap(),
        b"helloworld"
    );
    assert_eq!(fs::read(payload(&f).join("heap.bin")).unwrap(), b"");
    assert_eq!(fs::read(payload(&f).join("stable.bin")).unwrap(), b"");
    assert_eq!(fs::read_dir(payload(&f)).unwrap().count(), 3);
    let journal = fs::read(f.ledger()).unwrap();
    assert_eq!(
        ic_host_artifacts::artifact::Sha256Digest::from_bytes(
            f.op.check().unwrap().assembly_sha256.unwrap()
        )
        .to_string(),
        "71f4aa3d4064e1e45191510033efb8118152f5c8e4bb41f40ee883bef04e43dc"
    );
    let mut f = f.reopen();
    f.signer.allow = false;
    assert_eq!(assemble(&mut f).unwrap(), 10);
    assert_eq!(fs::read(f.ledger()).unwrap(), journal);
    assert_eq!(f.server.count(), 2);
    assert_eq!(f.signer.calls, 2);
    assert!(!payload(&f).parent().unwrap().join("manifest.json").exists());
    assert!(
        f.store
            .list_backups(
                None,
                crate::store::ScanLimits {
                    entries: 1.try_into().unwrap(),
                    bytes: 1024.try_into().unwrap(),
                    results: 1.try_into().unwrap(),
                }
            )
            .unwrap()
            .is_empty()
    );
}

#[test]
fn preflight_refuses_incomplete_reads_and_wrong_module_without_assembly_intent() {
    let mut f = Fixture::new(false, false);
    assert!(assemble(&mut f).is_err());
    assert!(!f.op.check().unwrap().assembly_started);
    assert!(!payload(&f).exists());
    let mut f = Fixture::with_hash(
        false,
        false,
        20,
        16 * 1024 * 1024,
        ic_host_artifacts::artifact::Sha256Digest::compute(b"otherwasm!"),
    );
    for n in 0..2 {
        f.advance(n).unwrap();
    }
    assert!(matches!(
        assemble(&mut f),
        Err(CaptureDataAdvanceError::ModuleHash)
    ));
    assert!(!f.op.check().unwrap().assembly_started);
    assert!(!payload(&f).exists());
}

#[test]
fn complete_unrecorded_member_is_reused_but_partial_or_changed_bytes_are_preserved() {
    for bytes in [b"helloworld".as_slice(), b"hello", b"wrongbytes"] {
        let mut f = acquired();
        seed(&mut f, bytes);
        let original = fs::metadata(payload(&f).join("module.wasm")).unwrap().ino();
        let before = fs::read(f.ledger()).unwrap();
        let mut f = f.reopen();
        if bytes == b"helloworld" {
            assert_eq!(assemble(&mut f).unwrap(), 10);
            assert!(f.op.check().unwrap().assembly_sha256.is_some());
        } else {
            assert!(assemble(&mut f).is_err());
            assert!(matches!(f.op.inspect(), Err(CaptureError::Unusable)));
            assert_eq!(fs::read(f.ledger()).unwrap(), before);
            assert!(!payload(&f).join("heap.bin").exists());
        }
        assert_eq!(fs::read(payload(&f).join("module.wasm")).unwrap(), bytes);
        assert_eq!(
            fs::metadata(payload(&f).join("module.wasm")).unwrap().ino(),
            original
        );
    }
}

#[test]
fn verified_assembly_damage_never_recreates_or_repairs_members() {
    for fault in ["missing", "changed", "extra", "symlink", "hardlink"] {
        let mut f = acquired();
        assemble(&mut f).unwrap();
        let before = fs::read(f.ledger()).unwrap();
        let module = payload(&f).join("module.wasm");
        match fault {
            "missing" => fs::remove_file(&module).unwrap(),
            "changed" => fs::write(&module, b"wrongbytes").unwrap(),
            "extra" => fs::write(payload(&f).join("extra"), b"extra").unwrap(),
            "symlink" => {
                fs::rename(&module, f.root.join("retained-module")).unwrap();
                symlink(f.root.join("retained-module"), &module).unwrap();
            }
            "hardlink" => fs::hard_link(&module, f.root.join("alias-module")).unwrap(),
            _ => unreachable!(),
        }
        assert!(assemble(&mut f).is_err());
        assert_eq!(fs::read(f.ledger()).unwrap(), before);
        if fault == "missing" {
            assert!(!module.exists());
        }
        if fault == "changed" {
            assert_eq!(fs::read(module).unwrap(), b"wrongbytes");
        }
        let Fixture { op, store, p, .. } = f;
        drop(op);
        assert!(
            store
                .open_capture(&p.request().operation_id, p.digest())
                .is_err()
        );
    }
}

#[test]
fn uncertain_assembly_frames_preserve_full_records_and_refuse_partial_tails() {
    for verified in [false, true] {
        for partial in [false, true] {
            let mut f = acquired();
            let event = if verified {
                f.op.commit(&EventRecord::AssemblyStarted, None).unwrap();
                let prefix = fs::read(f.ledger()).unwrap();
                assemble(&mut f).unwrap();
                let sha256 = f.op.check().unwrap().assembly_sha256.unwrap();
                // Construct the pre-verification crash image with complete files.
                fs::write(f.ledger(), prefix).unwrap();
                EventRecord::AssemblyVerified { sha256 }
            } else {
                EventRecord::AssemblyStarted
            };
            assert!(
                f.op.commit_with(&event, None, |file, bytes| {
                    file.write_all(if partial {
                        &bytes[..bytes.len() / 2]
                    } else {
                        bytes
                    })?;
                    file.sync_all()?;
                    Err(std::io::Error::other("uncertain assembly frame"))
                })
                .is_err()
            );
            assert!(matches!(f.op.inspect(), Err(CaptureError::Unusable)));
            if partial {
                let before = fs::read(f.ledger()).unwrap();
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
                let before = fs::read(f.ledger()).unwrap();
                assert_eq!(assemble(&mut f).unwrap(), 10);
                if verified {
                    assert_eq!(fs::read(f.ledger()).unwrap(), before);
                }
            }
        }
    }
}

#[test]
fn all_regions_and_sorted_chunks_including_empty_chunk_keep_exact_bytes() {
    use ic_host_artifacts::artifact::Sha256Digest;
    let mut f = Fixture::with_parts(
        false,
        false,
        20,
        16 * 1024 * 1024,
        Sha256Digest::compute(b"helloworld"),
        true,
    );
    for n in 0..7 {
        f.advance(n).unwrap();
    }
    assert_eq!(assemble(&mut f).unwrap(), 24);
    for (name, bytes) in [
        ("module.wasm", b"helloworld".as_slice()),
        ("heap.bin", b"abcdef"),
        ("stable.bin", b"xyz"),
    ] {
        assert_eq!(fs::read(payload(&f).join(name)).unwrap(), bytes);
    }
    for bytes in [b"".as_slice(), b"chunk"] {
        assert_eq!(
            fs::read(payload(&f).join(format!("chunk-{}.bin", Sha256Digest::compute(bytes))))
                .unwrap(),
            bytes
        );
    }
    assert_eq!(fs::read_dir(payload(&f)).unwrap().count(), 5);
    let mut f = f.reopen();
    assert_eq!(assemble(&mut f).unwrap(), 24);
}

#[test]
fn assembly_reserves_both_frames_before_creating_a_bundle() {
    let mut f = Fixture::with_limits(false, false, 20, 4_690_000);
    for n in 0..2 {
        f.advance(n).unwrap();
    }
    let before = fs::read(f.ledger()).unwrap();
    assert!(matches!(
        assemble(&mut f),
        Err(CaptureDataAdvanceError::Capture(CaptureError::Ledger(_)))
    ));
    assert_eq!(fs::read(f.ledger()).unwrap(), before);
    assert!(!payload(&f).parent().unwrap().exists());
    assert!(!f.op.check().unwrap().assembly_started);
}

#[test]
fn assembly_event_order_is_closed_and_start_encoding_is_stable() {
    let mut f = acquired();
    assert_eq!(
        serde_json::to_vec(&EventRecord::AssemblyStarted).unwrap(),
        b"{\"kind\":\"assembly_started\"}"
    );
    assert!(
        f.op.check()
            .unwrap()
            .candidate(&EventRecord::AssemblyVerified { sha256: [0; 32] }, &f.p)
            .is_err()
    );
    f.op.commit(&EventRecord::AssemblyStarted, None).unwrap();
    assert!(
        f.op.check()
            .unwrap()
            .candidate(&EventRecord::AssemblyStarted, &f.p)
            .is_err()
    );
    assemble(&mut f).unwrap();
    assert!(
        f.op.check()
            .unwrap()
            .candidate(&EventRecord::AssemblyVerified { sha256: [0; 32] }, &f.p)
            .is_err()
    );
}
