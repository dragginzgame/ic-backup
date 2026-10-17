//! Local bounded byte hashing and payload binding; no management effects.

use super::*;
use crate::{
    model::ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
    test_support::ic_snapshot_upload::{DESTINATION_ID, SOURCE_ID, TARGET, raw, source_plan},
};
use std::io::Cursor;

struct Counted<'a> {
    bytes: Cursor<&'a [u8]>,
    read_bytes: usize,
    largest_request: usize,
}
impl Read for Counted<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.largest_request = self.largest_request.max(output.len());
        let count = self.bytes.read(output)?;
        self.read_bytes += count;
        Ok(count)
    }
}

#[test]
fn streaming_bindings_hash_exact_consumed_bytes_once_and_reject_short_or_excess_input() {
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let mut values: ic_management_canister_types::ReadCanisterSnapshotMetadataResult =
        candid::decode_one(&raw("upload-source")).unwrap();
    let length = MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES + 1;
    values.wasm_module_size = length as u64;
    let raw = candid::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let members = member::expected(&metadata).unwrap();
    let module = members
        .iter()
        .find(|member| member.name == REGIONS[0])
        .unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"declared original source");
    let upload = IcSnapshotUploadRequest::metadata(&plan, &metadata, &checksum).unwrap();
    let kinds = [
        SnapshotDataKind::WasmModule {
            offset: 0,
            size: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
        },
        SnapshotDataKind::WasmModule {
            offset: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
            size: 1,
        },
    ];
    for delta in [-1, 0, 1] {
        let mut bytes = vec![7; length];
        bytes[length - 1] = 9;
        match delta {
            -1 => {
                bytes.pop();
            }
            1 => bytes.push(0),
            _ => {}
        }
        let mut reader = Counted {
            bytes: Cursor::new(&bytes),
            read_bytes: 0,
            largest_request: 0,
        };
        let mut remaining = kinds.iter().peekable();
        let mut bindings = vec![];
        let result = member_checksum(
            &mut reader,
            module,
            length as u64,
            &upload,
            DESTINATION_ID,
            &mut remaining,
            &mut bindings,
        );
        if delta == 0 {
            assert_eq!(result.unwrap(), ArtifactChecksumRecord::from_bytes(&bytes));
            assert_eq!(reader.read_bytes, length);
            assert!(remaining.next().is_none());
            for ((kind, chunk), binding) in kinds
                .iter()
                .zip(bytes.chunks(MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES))
                .zip(&bindings)
            {
                let expected =
                    IcSnapshotUploadRequest::data(&upload, DESTINATION_ID, kind.clone(), chunk)
                        .unwrap();
                assert_eq!(expected.binding_digest(), *binding);
            }
        } else {
            assert!(matches!(
                result,
                Err(IcSnapshotUploadArtifactError::Artifact(
                    IcSnapshotArtifactError::FileShape
                ))
            ));
        }
        assert!(reader.largest_request <= MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES);
    }
}
