//! Changed original region files stop further writes without repairing evidence.

use super::*;

#[test]
fn changed_region_identity_or_extent_stops_an_unrelated_append_without_writes() {
    for action in [
        "replaced",
        "symlink",
        "directory",
        "missing",
        "truncated",
        "extended",
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut journal = journal(&layout, metadata.metadata().taken_at_timestamp);
        let original = fs::read(journal.path()).unwrap();
        let writer = journal
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
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
            "replaced" | "symlink" | "directory" | "missing" => {
                fs::rename(&region, &retained).unwrap();
                match action {
                    "replaced" => fs::write(&region, [8]).unwrap(),
                    "symlink" => symlink(&retained, &region).unwrap(),
                    "directory" => fs::create_dir(&region).unwrap(),
                    "missing" => {}
                    _ => unreachable!(),
                }
            }
            "truncated" => fs::write(&region, []).unwrap(),
            "extended" => fs::write(&region, [8, 99]).unwrap(),
            _ => unreachable!(),
        }
        let error = append(
            writer,
            SnapshotDataKind::WasmMemory { offset: 0, size: 1 },
            vec![42],
        )
        .expect_err("changed original region must stop further writes");
        if action == "missing" {
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
            "replaced" | "symlink" => assert_eq!(fs::read(&region).unwrap(), [8]),
            "directory" => assert_eq!(fs::read_dir(&region).unwrap().count(), 0),
            "missing" => assert!(!region.exists()),
            "truncated" => assert_eq!(fs::read(&region).unwrap(), [] as [u8; 0]),
            "extended" => assert_eq!(fs::read(&region).unwrap(), [8, 99]),
            _ => unreachable!(),
        }
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        assert_eq!(
            journal.record().unwrap().artifacts()[0].state(),
            ArtifactStateRecord::Created
        );
        assert!(!root.join(format!("artifacts/{TARGET}")).exists());
        assert!(
            journal
                .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
                .is_err()
        );
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        drop(journal);
        drop(layout);
        fs::remove_dir_all(root).unwrap();
    }
}
