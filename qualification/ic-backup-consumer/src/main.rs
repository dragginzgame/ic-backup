//! Test-only public-API consumer, built alone without Testkit/PocketIC in its graph.
use ic_backup::store::{
    BackupConnection, BackupDigest, BackupHttpTransport, BackupStore, OperationProgress,
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize},
    },
    time::Duration,
};
mod authority;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(args.len() == 5 || args.len() == 8);
    let store = BackupStore::open(Path::new(&args[2])).unwrap();
    let approval: BackupDigest = args[4].parse().unwrap();
    match args[1].as_str() {
        "capture-inspect" => {
            assert!(
                store
                    .open_capture(&args[3], approval)
                    .unwrap()
                    .inspect()
                    .unwrap()
                    .completed
                    .is_some()
            );
            return;
        }
        "restore-inspect" => {
            assert!(
                store
                    .open_restore(&args[3], approval)
                    .unwrap()
                    .inspect()
                    .unwrap()
                    .completed
                    .is_some()
            );
            return;
        }
        _ => assert_eq!(args.len(), 8),
    }
    let mut connection = BackupConnection::new(
        BackupHttpTransport::new(
            &args[5],
            args[7].parse().unwrap(),
            std::fs::read(&args[6]).unwrap(),
            Duration::from_secs(30),
        )
        .unwrap(),
        authority::FixtureSigner(Arc::new(AtomicUsize::new(0))),
        authority::FixtureGuard(Arc::new(AtomicBool::new(true))),
    );
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        match args[1].as_str() {
            "capture-step" | "capture-finish" => {
                let mut operation = store.open_capture(&args[3], approval).unwrap();
                for _ in 0..128 {
                    match operation.advance(&mut connection).await.unwrap() {
                        OperationProgress::Advanced if args[1] == "capture-step" => return,
                        OperationProgress::Advanced => (),
                        OperationProgress::Complete(_) => return,
                        OperationProgress::NeedsAttention(step) => {
                            panic!("unexpected pending {step:?}")
                        }
                    }
                }
            }
            "restore-step" | "restore-finish" => {
                let mut operation = store.open_restore(&args[3], approval).unwrap();
                for _ in 0..128 {
                    match operation.advance(&mut connection).await.unwrap() {
                        OperationProgress::Advanced if args[1] == "restore-step" => return,
                        OperationProgress::Advanced => (),
                        OperationProgress::Complete(_) => return,
                        OperationProgress::NeedsAttention(step) => {
                            panic!("unexpected pending {step:?}")
                        }
                    }
                }
            }
            _ => panic!("unknown fixture command"),
        }
        panic!("fixture exceeded finite command bound");
    });
}
