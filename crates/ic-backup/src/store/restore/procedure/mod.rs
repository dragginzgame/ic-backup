//! Fixed source-bound allocation/upload/readback/load policy, without call mechanics.

#[cfg(test)]
mod tests;

use super::{RestorePlan, persistence::Source};
use crate::store::{
    CapturePlan, CaptureStatusReplyError, OperationStoreError, PayloadReadError, RestoreTransfer,
    TransferDataError,
    capture::{
        SnapshotReplyError, decode_snapshot_list,
        transfer::{self, readback::ReadbackMetadataError},
    },
    operation::{Step, effect::Effect},
};
use ic_platform_protocol::management_snapshots::{
    CanisterArgs, Extent, LoadSnapshotArgs, Offset, ReadDataArgs, ReadKind, Snapshot, SnapshotArgs,
    SnapshotChunk, SnapshotMetadata, UploadDataArgs, UploadMetadataArgs, WriteKind,
};
use ic_platform_protocol::{
    candid::{Error as DecodeError, Limits},
    management_snapshots::{decode_empty_response, decode_snapshot_allocation},
};
use ic_platform_types::Principal;

#[derive(Debug, thiserror::Error)]
/// Original restore source, bounded reply or exact readback policy refused.
pub enum Error {
    /// The fixed restore sequence is incomplete or blocked.
    #[error("restore sequence refused")]
    Order,
    /// The original certified call was rejected; spending remains consumed.
    #[error("restore call was certified rejected")]
    Rejected,
    /// Each status boundary must have a distinct original signed request.
    #[error("restore status request was reused")]
    StatusRequest,
    /// Initial/pre-load/post-load status refused.
    #[error(transparent)]
    Status(#[from] CaptureStatusReplyError),
    /// Snapshot inventory/capacity or allocated ID refused.
    #[error(transparent)]
    Snapshot(#[from] SnapshotReplyError),
    /// Bounded allocation or unit reply refused.
    #[error("restore reply decode refused")]
    Reply(DecodeError),
    /// Source metadata cannot be represented without defaults.
    #[error("source metadata is outside the restore profile")]
    MetadataProfile,
    /// Fixed argument encoding exceeded its original ceiling.
    #[error("restore argument encoding refused")]
    Encoding,
    /// Original source geometry refused.
    #[error(transparent)]
    Transfer(#[from] crate::store::RestoreTransferError),
    /// Original local payload/identity refused.
    #[error(transparent)]
    Source(#[from] PayloadReadError),
    /// Destination metadata differs in restorable state.
    #[error(transparent)]
    Metadata(#[from] ReadbackMetadataError),
    /// Bounded destination data reply refused.
    #[error(transparent)]
    Data(#[from] TransferDataError),
    /// Destination bytes differ from the held original source.
    #[error("restore readback differs from original source bytes")]
    Readback,
    /// Allocated destination is empty, excessive, reused or differs from original evidence.
    #[error("restore allocated snapshot identity refused")]
    Allocation,
    /// Original source projection cannot reconstruct its capture profile.
    #[error(transparent)]
    Capture(#[from] OperationStoreError),
}

pub(in crate::store) struct Procedure<'a> {
    plan: &'a RestorePlan,
    next: Option<Step>,
    failed: bool,
    transfer: RestoreTransfer,
    source_plan: CapturePlan,
    metadata: SnapshotMetadata,
    inventory: Vec<Snapshot>,
    status_requests: Vec<[u8; 32]>,
    destination: Option<Vec<u8>>,
}
impl<'a> Procedure<'a> {
    pub(in crate::store) fn new(plan: &'a RestorePlan, source: &Source<'_>) -> Result<Self, Error> {
        Ok(Self {
            plan,
            next: Some(Step::RestoreInitialStatus),
            failed: false,
            transfer: source.transfer(plan)?,
            source_plan: source.capture.plan().clone(),
            metadata: source.metadata.clone(),
            inventory: Vec::new(),
            status_requests: Vec::new(),
            destination: None,
        })
    }

    pub(in crate::store) fn next(&self) -> Result<Option<Step>, Error> {
        if self.failed {
            Err(Error::Order)
        } else {
            Ok(self.next)
        }
    }

    pub(in crate::store) fn destination(&self) -> Result<&[u8], Error> {
        self.destination.as_deref().ok_or(Error::Order)
    }

    pub(in crate::store) fn data_calls(&self) -> u32 {
        self.transfer.data_calls()
    }

    fn source_request(&self, n: u32) -> Result<ReadDataArgs, Error> {
        self.transfer
            .source_requests()
            .nth(n as usize)
            .ok_or(Error::Order)
    }

    #[cfg(test)]
    pub(in crate::store) fn request(
        &self,
        source: &mut Source<'_>,
    ) -> Result<(&'static str, Vec<u8>), Error> {
        let (effect, limit) = self.effect(source)?;

        effect.encode(limit).map_err(|_| Error::Encoding)
    }

    pub(in crate::store) fn effect(
        &self,
        source: &mut Source<'_>,
    ) -> Result<(Effect, usize), Error> {
        let target = self.plan.request().target.canister;
        let child = CanisterArgs {
            canister_id: target,
        };
        let effect = match self.next()?.ok_or(Error::Order)? {
            Step::RestoreInitialStatus | Step::BeforeLoadStatus | Step::AfterLoadStatus => {
                Effect::Status(child)
            }
            Step::RestoreListSnapshots => Effect::List(child),
            Step::AllocateSnapshot => {
                return Ok((
                    Effect::Allocate(upload_metadata(target, &self.metadata)?),
                    1024 * 1024,
                ));
            }
            Step::Upload(n) => {
                let request = self.source_request(n)?;
                let chunk = source.payload.read_data(&request)?;
                let kind = match request.kind {
                    ReadKind::WasmModule(Extent { offset, .. }) => {
                        WriteKind::WasmModule(Offset { offset })
                    }
                    ReadKind::WasmMemory(Extent { offset, .. }) => {
                        WriteKind::WasmMemory(Offset { offset })
                    }
                    ReadKind::StableMemory(Extent { offset, .. }) => {
                        WriteKind::StableMemory(Offset { offset })
                    }
                    ReadKind::WasmChunk(SnapshotChunk { .. }) => WriteKind::WasmChunk,
                };

                Effect::Upload(UploadDataArgs {
                    canister_id: target,
                    snapshot_id: self.destination()?.to_vec(),
                    kind,
                    chunk,
                })
            }
            Step::ReadbackMetadata => Effect::Metadata(SnapshotArgs {
                canister_id: target,
                snapshot_id: self.destination()?.to_vec(),
            }),
            Step::Readback(n) => {
                let mut request = self.source_request(n)?;
                request.snapshot_id = self.destination()?.to_vec();

                Effect::Read(request)
            }
            Step::LoadSnapshot => Effect::Load(LoadSnapshotArgs {
                canister_id: target,
                snapshot_id: self.destination()?.to_vec(),
                sender_canister_version: None,
            }),
            _ => return Err(Error::Order),
        };

        Ok((effect, crate::store::limits::REQUEST_BYTES as usize))
    }

    pub(in crate::store) fn accept(
        &mut self,
        step: Step,
        rejected: bool,
        request_id: [u8; 32],
        reply: &[u8],
        source: &mut Source<'_>,
    ) -> Result<(), Error> {
        let result = self.accept_reply(step, rejected, request_id, reply, source);
        if result.is_err() {
            self.failed = true;
        }

        result
    }

    fn accept_reply(
        &mut self,
        step: Step,
        rejected: bool,
        request_id: [u8; 32],
        reply: &[u8],
        source: &mut Source<'_>,
    ) -> Result<(), Error> {
        if self.next()? != Some(step) {
            return Err(Error::Order);
        }
        if rejected {
            return Err(Error::Rejected);
        }
        self.next = match step {
            Step::RestoreInitialStatus | Step::BeforeLoadStatus | Step::AfterLoadStatus => {
                if self.status_requests.contains(&request_id) {
                    return Err(Error::StatusRequest);
                }
                crate::store::operation::intent::Plan::Restore(self.plan)
                    .status_requirements()
                    .validate_status_reply(reply)?;
                self.status_requests.push(request_id);

                match step {
                    Step::RestoreInitialStatus => Some(Step::RestoreListSnapshots),
                    Step::BeforeLoadStatus => Some(Step::LoadSnapshot),
                    _ => None,
                }
            }
            Step::RestoreListSnapshots => {
                self.inventory = decode_snapshot_list(reply)?;
                if self.inventory.len() >= self.plan.profile().snapshot_capacity as usize {
                    return Err(SnapshotReplyError::NoCapacity.into());
                }

                Some(Step::AllocateSnapshot)
            }
            Step::AllocateSnapshot => {
                let value = decode_snapshot_allocation(reply, limits())?;
                if !(1..=256).contains(&value.snapshot_id.len())
                    || self.inventory.iter().any(|s| s.id == value.snapshot_id)
                    || self
                        .transfer
                        .source_requests()
                        .next()
                        .is_some_and(|r| r.snapshot_id == value.snapshot_id)
                {
                    return Err(Error::Allocation);
                }
                self.destination = Some(value.snapshot_id);

                Some(Step::Upload(0))
            }
            Step::Upload(n) => {
                unit(reply)?;

                Some(if n + 1 == self.transfer.data_calls() {
                    Step::ReadbackMetadata
                } else {
                    Step::Upload(n.checked_add(1).ok_or(Error::Order)?)
                })
            }
            Step::ReadbackMetadata => {
                let source_id = self
                    .transfer
                    .source_requests()
                    .next()
                    .ok_or(Error::Order)?
                    .snapshot_id;
                let transfer = self
                    .source_plan
                    .plan()
                    .plan_transfer(&source_id, &self.metadata)
                    .map_err(|_| Error::MetadataProfile)?;
                transfer.decode_uploaded_metadata(reply)?;

                Some(Step::Readback(0))
            }
            Step::Readback(n) => {
                let bytes = transfer::data::decode_reply(reply)?;
                let original = source.payload.read_data(&self.source_request(n)?)?;
                if bytes != original {
                    return Err(Error::Readback);
                }

                Some(if n + 1 == self.transfer.data_calls() {
                    Step::BeforeLoadStatus
                } else {
                    Step::Readback(n.checked_add(1).ok_or(Error::Order)?)
                })
            }
            Step::LoadSnapshot => {
                unit(reply)?;

                Some(Step::AfterLoadStatus)
            }
            _ => return Err(Error::Order),
        };

        Ok(())
    }
}

pub(in crate::store) fn upload_metadata_arguments(
    canister_id: Principal,
    metadata: &SnapshotMetadata,
    limit: usize,
) -> Result<Vec<u8>, Error> {
    upload_metadata(canister_id, metadata)?
        .encode(limit)
        .map_err(|_| Error::Encoding)
}

fn upload_metadata(
    canister_id: Principal,
    metadata: &SnapshotMetadata,
) -> Result<UploadMetadataArgs, Error> {
    let globals = metadata
        .globals
        .iter()
        .cloned()
        .collect::<Option<Vec<_>>>()
        .ok_or(Error::MetadataProfile)?;

    Ok(UploadMetadataArgs {
        canister_id,
        replace_snapshot: None,
        wasm_module_size: metadata.wasm_module_size,
        globals,
        wasm_memory_size: metadata.wasm_memory_size,
        stable_memory_size: metadata.stable_memory_size,
        certified_data: metadata.certified_data.clone(),
        global_timer: metadata.global_timer.clone(),
        on_low_wasm_memory_hook_status: metadata.on_low_wasm_memory_hook_status.clone(),
    })
}

fn limits() -> Limits {
    Limits {
        bytes: 4096,
        header: 2048,
        types: 16,
        work: 8192,
        allocation: 4096,
        vector: 256,
        skipping: 0,
    }
}

fn unit(reply: &[u8]) -> Result<(), Error> {
    decode_empty_response(reply, limits()).map_err(Error::from)
}
impl From<DecodeError> for Error {
    fn from(error: DecodeError) -> Self {
        Self::Reply(error)
    }
}
