//! Changed original region files stop further writes without repairing evidence.

use super::file_change::Replacement;
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RegionFault {
    Replaced,
    Symlink,
    Directory,
    Missing,
    Truncated,
    Extended,
}

#[test]
fn changed_region_identity_or_extent_stops_an_unrelated_append_without_writes() {
    for action in [
        RegionFault::Replaced,
        RegionFault::Symlink,
        RegionFault::Directory,
        RegionFault::Missing,
        RegionFault::Truncated,
        RegionFault::Extended,
    ] {
        let root = with_created(
            |CreatedArtifact {
                 root,
                 raw,
                 metadata,
                 mut journal,
                 ..
             }| {
                let original = fs::read(journal.path()).unwrap();
                let writer = journal
                    .stage_ic_snapshot_artifact(TOKEN, metadata, raw)
                    .unwrap();
                let writer = append(
                    writer,
                    SnapshotDataKind::WasmModule { offset: 0, size: 1 },
                    vec![8],
                )
                .unwrap();
                let path = writer.path().to_owned();
                let region = path.join(REGIONS[0]);
                let retained = root.join("retained-module");
                match action {
                    RegionFault::Replaced => Replacement::File(&[8]).apply(&region, &retained),
                    RegionFault::Symlink => Replacement::Symlink.apply(&region, &retained),
                    RegionFault::Directory => Replacement::Directory.apply(&region, &retained),
                    RegionFault::Missing => Replacement::Missing.apply(&region, &retained),
                    RegionFault::Truncated => fs::write(&region, []).unwrap(),
                    RegionFault::Extended => fs::write(&region, [8, 99]).unwrap(),
                }
                let error = append(
                    writer,
                    SnapshotDataKind::WasmMemory { offset: 0, size: 1 },
                    vec![42],
                )
                .expect_err("changed original region must stop further writes");
                if action == RegionFault::Missing {
                    assert!(
                        matches!(error, IcSnapshotArtifactError::Io(ref error) if error.kind() == io::ErrorKind::NotFound)
                    );
                } else {
                    assert!(matches!(error, IcSnapshotArtifactError::FileShape));
                }
                assert_eq!(fs::read(path.join(REGIONS[1])).unwrap(), [] as [u8; 0]);
                assert_eq!(fs::read(path.join(REGIONS[2])).unwrap(), [] as [u8; 0]);
                if retained.exists() {
                    assert_eq!(fs::read(&retained).unwrap(), [8]);
                }
                match action {
                    RegionFault::Replaced | RegionFault::Symlink => {
                        assert_eq!(fs::read(&region).unwrap(), [8]);
                    }
                    RegionFault::Directory => assert_eq!(fs::read_dir(&region).unwrap().count(), 0),
                    RegionFault::Missing => assert!(!region.exists()),
                    RegionFault::Truncated => assert_eq!(fs::read(&region).unwrap(), [] as [u8; 0]),
                    RegionFault::Extended => assert_eq!(fs::read(&region).unwrap(), [8, 99]),
                }
                assert_eq!(fs::read(journal.path()).unwrap(), original);
                assert_eq!(
                    journal.record().unwrap().artifacts()[0].state(),
                    ArtifactStateRecord::Created
                );
                assert!(!root.join(format!("artifacts/{TARGET}")).exists());
                assert!(
                    journal
                        .stage_ic_snapshot_artifact(TOKEN, metadata, raw)
                        .is_err()
                );
                assert_eq!(fs::read(journal.path()).unwrap(), original);
                drop(journal);
            },
        );
        fs::remove_dir_all(root).unwrap();
    }
}
