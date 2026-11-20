//! Inner management call selected by the fixed procedure. Live execution and
//! replay derive this value from the same approved inputs and preceding replies.

use ic_platform_protocol::management_snapshots::{
    CanisterArgs, LoadSnapshotArgs, ReadDataArgs, SnapshotArgs, TakeSnapshotArgs, UploadDataArgs,
    UploadMetadataArgs,
};

///
/// Effect
/// Typed snapshot operation selected by the fixed capture or restore procedure.
///

pub enum Effect {
    /// Inspect stopped state and controllers.
    Status(CanisterArgs),
    /// List original snapshots.
    List(CanisterArgs),
    /// Take a new snapshot.
    Take(TakeSnapshotArgs),
    /// Read snapshot metadata.
    Metadata(SnapshotArgs),
    /// Read a snapshot extent.
    Read(ReadDataArgs),
    /// Allocate a restore snapshot.
    Allocate(UploadMetadataArgs),
    /// Upload an extent.
    Upload(UploadDataArgs),
    /// Load the verified snapshot.
    Load(LoadSnapshotArgs),
}

impl Effect {
    pub(in crate::store) fn encode(
        &self,
        limit: usize,
    ) -> Result<(&'static str, Vec<u8>), ic_platform_protocol::management_snapshots::EncodeError>
    {
        Ok(match self {
            Self::Status(a) => ("canister_status", a.encode(limit)?),
            Self::List(a) => ("list_canister_snapshots", a.encode(limit)?),
            Self::Take(a) => ("take_canister_snapshot", a.encode(limit)?),
            Self::Metadata(a) => ("read_canister_snapshot_metadata", a.encode(limit)?),
            Self::Read(a) => ("read_canister_snapshot_data", a.encode(limit)?),
            Self::Allocate(a) => ("upload_canister_snapshot_metadata", a.encode(limit)?),
            Self::Upload(a) => ("upload_canister_snapshot_data", a.encode(limit)?),
            Self::Load(a) => ("load_canister_snapshot", a.encode(limit)?),
        })
    }
}
