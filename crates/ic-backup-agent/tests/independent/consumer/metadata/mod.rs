//! Fresh original request and raw reply ownership, with borrowing local decode.
use super::{Driver, Stage, error};
use ic_backup::model::ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest};
use std::io;

pub(super) struct MetadataEvidence {
    pub request: IcSnapshotMetadataRequest,
    pub raw: Vec<u8>,
}
impl MetadataEvidence {
    pub(super) fn decode(&self) -> io::Result<IcSnapshotMetadataReply<'_>> {
        IcSnapshotMetadataReply::decode(&self.request, &self.raw).map_err(error)
    }
}
impl Driver {
    pub(super) fn metadata_evidence(&self) -> io::Result<MetadataEvidence> {
        Ok(MetadataEvidence {
            request: self.metadata_request()?,
            raw: self.raw(Stage::Metadata)?,
        })
    }
}
