//! Finite supplier fixtures and independent retained wire inputs for owning tests.
use ic_platform_protocol::management_snapshots::{self as management, fixtures};

pub(in crate::store) trait EncodeFixture {
    fn encoded(&self, maximum: usize) -> Vec<u8>;
}
pub(in crate::store) fn encode(value: &impl EncodeFixture, maximum: usize) -> Vec<u8> {
    value.encoded(maximum)
}
macro_rules! reply {
    ($kind:ty, $function:ident) => {
        impl EncodeFixture for $kind {
            fn encoded(&self, maximum: usize) -> Vec<u8> {
                fixtures::$function(self, maximum).unwrap()
            }
        }
    };
}
reply!(management::CaptureStatus, capture_status);
reply!(management::Snapshot, take_snapshot);
reply!(Vec<management::Snapshot>, list_snapshots);
reply!(management::SnapshotMetadata, snapshot_metadata);
reply!(management::SnapshotData, snapshot_data);
macro_rules! request {
    ($($kind:ty),*) => {$(
        impl EncodeFixture for $kind {
            fn encoded(&self, maximum: usize) -> Vec<u8> { self.encode(maximum).unwrap() }
        }
    )*};
}
request!(
    management::CanisterArgs,
    management::TakeSnapshotArgs,
    management::SnapshotArgs,
    management::ReadDataArgs,
    management::LoadSnapshotArgs,
    management::UploadMetadataArgs,
    management::UploadDataArgs
);

mod retained_tests;
pub(in crate::store) use retained_tests::frozen;
