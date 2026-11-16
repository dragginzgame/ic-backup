//! Fixed capture policy derived from original authenticated call results.
//!
//! Retained calls are authenticated against the exact request and original
//! intent before reply policy derives the next request;
//! it grants no signing, filesystem, submission or application authority.
use super::intent::CapturePlan;
use super::{
    CaptureStatusReplyError, DataReadError, MetadataReadError, SnapshotReplyError,
    TransferDataError, TransferError, TransferMetadataError, decode_snapshot_list,
    decode_taken_snapshot,
};
#[cfg(test)]
use crate::store::PreparedStatus;
#[cfg(test)]
use crate::store::StatusEvidence;
use crate::store::operation::Step;
#[cfg(test)]
use crate::store::operation::intent::Intent;
use ic_platform::{
    candid_codec,
    management_canister_types::{
        CanisterIdRecord, ReadCanisterSnapshotMetadataResult, Snapshot, SnapshotDataKind,
    },
};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum Next {
    /// Sign the exact derived management call; never submit it.
    Call(Step),
    Materialize,
    Publish,
}

pub(in crate::store) struct Procedure<'a> {
    intent: &'a CapturePlan,
    next: Next,
    failed: bool,
    initial_request: Option<[u8; 32]>,
    inventory: Vec<Snapshot>,
    snapshot: Option<Snapshot>,
    metadata: Option<ReadCanisterSnapshotMetadataResult>,
    module: Option<(Sha256, u64)>,
}

#[derive(Debug, Error)]
/// Authenticated call results cannot advance the fixed capture procedure.
pub enum Error {
    #[error(transparent)]
    /// Original retained intent or evidence failed authentication.
    Authentication(#[from] super::intent::Error),
    #[error("original call was certified rejected")]
    /// Original call was certified rejected; its reservations remain spent.
    Rejected,
    #[error("capture policy is blocked or local prerequisites are incomplete")]
    /// Required fixed capture prerequisites are missing or blocked.
    Order,
    #[error(transparent)]
    /// Stopped/module/controller policy refused.
    Status(#[from] CaptureStatusReplyError),
    #[error(transparent)]
    /// Snapshot inventory, capacity or take result refused.
    Snapshot(#[from] SnapshotReplyError),
    #[error(transparent)]
    /// Read the exact retained snapshot metadata.
    Metadata(#[from] TransferMetadataError),
    #[error(transparent)]
    /// Deterministic payload schedule exceeds the original profile.
    Schedule(#[from] TransferError),
    #[error(transparent)]
    /// Read one contiguous deterministic payload ordinal.
    Data(#[from] TransferDataError),
    #[error(transparent)]
    /// Exact metadata request encoding refused.
    MetadataArguments(#[from] MetadataReadError),
    #[error(transparent)]
    /// Exact scheduled payload request encoding refused.
    DataArguments(#[from] DataReadError),
    #[error("capture argument encoding refused")]
    /// Canonical management arguments could not be encoded.
    Arguments,
    #[error("metadata differs from the retained take timestamp")]
    /// Metadata differs from the authenticated take timestamp.
    SnapshotTimestamp,
    #[error("captured module differs from the approved module")]
    /// Complete captured module differs from the original approval.
    ModuleHash,
    #[error("final status must have a distinct original request")]
    /// Final status reused the original initial-status request.
    ReusedStatus,
}

impl<'a> Procedure<'a> {
    pub(in crate::store) fn new(intent: &'a CapturePlan) -> Result<Self, Error> {
        // Validate the selected route and capacity before any work is proposed.
        intent
            .plan()
            .take_snapshot_arguments(&[], intent.profile().snapshot_capacity)?;
        Ok(Self {
            intent,
            next: Next::Call(Step::InitialStatus),
            failed: false,
            initial_request: None,
            inventory: Vec::new(),
            snapshot: None,
            metadata: None,
            module: None,
        })
    }

    pub(in crate::store) fn next(&self) -> Result<Next, Error> {
        if self.failed {
            return Err(Error::Order);
        }
        Ok(self.next)
    }

    // The consumer cannot choose a phase, ordinal, target, method or arguments.
    pub(in crate::store) fn request(&self) -> Result<(&'static str, Vec<u8>), Error> {
        let Next::Call(step) = self.next()? else {
            return Err(Error::Order);
        };
        let (method, arguments) = match step {
            Step::InitialStatus | Step::FinalStatus | Step::ListSnapshots => (
                if step == Step::ListSnapshots {
                    "list_canister_snapshots"
                } else {
                    "canister_status"
                },
                candid_codec::encode_one(
                    &CanisterIdRecord {
                        canister_id: self.intent.plan().request().target.canister,
                    },
                    256,
                )
                .map_err(|_| Error::Arguments)?,
            ),
            Step::TakeSnapshot => (
                "take_canister_snapshot",
                candid_codec::encode_one(
                    &self.intent.plan().take_snapshot_arguments(
                        &self.inventory,
                        self.intent.profile().snapshot_capacity,
                    )?,
                    512,
                )
                .map_err(|_| Error::Arguments)?,
            ),
            Step::Metadata => (
                "read_canister_snapshot_metadata",
                self.intent
                    .plan()
                    .metadata_read_arguments(self.snapshot.as_ref().ok_or(Error::Order)?)?,
            ),
            Step::Data(n) => (
                "read_canister_snapshot_data",
                self.transfer()?.data_arguments(n)?,
            ),
            _ => return Err(Error::Order),
        };
        Ok((method, arguments))
    }

    #[cfg(test)]
    pub(in crate::store) fn accept_authenticated(
        &mut self,
        step: Step,
        reference: u64,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), Error> {
        let result = (|| {
            let (method, arguments) = self.request()?;
            self.intent
                .view()
                .authenticate(reference, prepared, evidence, method, &arguments)?;
            self.accept(
                evidence.rejected,
                step,
                prepared.request_id,
                &evidence.response,
            )
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    pub(in crate::store) fn accept(
        &mut self,
        rejected: bool,
        step: Step,
        request_id: [u8; 32],
        reply: &[u8],
    ) -> Result<(), Error> {
        let result = if rejected {
            Err(Error::Rejected)
        } else {
            self.accept_reply(step, request_id, reply)
        };
        // Invalid retained evidence must not become permission to try another reply
        // on this projection. Reopen has to replay the same original evidence.
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn accept_reply(
        &mut self,
        step: Step,
        request_id: [u8; 32],
        reply: &[u8],
    ) -> Result<(), Error> {
        if self.next()? != Next::Call(step) {
            return Err(Error::Order);
        }
        self.next = match step {
            Step::InitialStatus => {
                self.intent.plan().validate_status_reply(reply)?;
                self.initial_request = Some(request_id);
                Next::Call(Step::ListSnapshots)
            }
            Step::ListSnapshots => {
                let inventory = decode_snapshot_list(reply)?;
                self.intent
                    .plan()
                    .take_snapshot_arguments(&inventory, self.intent.profile().snapshot_capacity)?;
                self.inventory = inventory;
                Next::Call(Step::TakeSnapshot)
            }
            Step::TakeSnapshot => {
                self.snapshot = Some(decode_taken_snapshot(reply, &self.inventory)?);
                self.inventory.clear();
                Next::Call(Step::Metadata)
            }
            Step::Metadata => {
                let snapshot = self.snapshot.as_ref().ok_or(Error::Order)?;
                let metadata = self.intent.plan().decode_metadata(&snapshot.id, reply)?;
                if metadata.taken_at_timestamp != snapshot.taken_at_timestamp {
                    return Err(Error::SnapshotTimestamp);
                }
                self.module = Some((Sha256::new(), 0));
                self.metadata = Some(metadata);
                Next::Call(Step::Data(0))
            }
            Step::Data(n) => {
                let transfer = self.transfer()?;
                let data = transfer.decode_data(n, reply)?;
                let module = matches!(
                    transfer
                        .requests()
                        .nth(n as usize)
                        .ok_or(Error::Order)?
                        .kind,
                    SnapshotDataKind::WasmModule { .. }
                );
                let last = n + 1 == transfer.data_calls();
                if module {
                    let (hash, bytes) = self.module.as_mut().ok_or(Error::Order)?;
                    *bytes = bytes
                        .checked_add(data.len() as u64)
                        .filter(|n| *n <= self.metadata.as_ref().map_or(0, |m| m.wasm_module_size))
                        .ok_or(Error::ModuleHash)?;
                    hash.update(&data);
                }
                if last {
                    let (hash, bytes) = self.module.take().ok_or(Error::Order)?;
                    if bytes != self.metadata.as_ref().ok_or(Error::Order)?.wasm_module_size
                        || hash.finalize().as_slice()
                            != self.intent.plan().request().module_sha256.as_bytes()
                    {
                        return Err(Error::ModuleHash);
                    }
                    Next::Materialize
                } else {
                    Next::Call(Step::Data(n + 1))
                }
            }
            Step::FinalStatus => {
                if self.initial_request == Some(request_id) {
                    return Err(Error::ReusedStatus);
                }
                self.intent.plan().validate_status_reply(reply)?;
                Next::Publish
            }
            _ => return Err(Error::Order),
        };
        Ok(())
    }

    // Called only by the private completion owner after it has verified exact
    // materialized files. This local transition is not a persisted verification
    // event and must never become a public caller-selected phase command.
    pub(in crate::store) fn after_materialization(&mut self) -> Result<(), Error> {
        if self.next()? != Next::Materialize {
            return Err(Error::Order);
        }
        self.next = Next::Call(Step::FinalStatus);
        Ok(())
    }

    pub(in crate::store) fn transfer(&self) -> Result<super::CaptureTransfer<'_>, Error> {
        Ok(self.intent.plan().plan_transfer(
            &self.snapshot.as_ref().ok_or(Error::Order)?.id,
            self.metadata.as_ref().ok_or(Error::Order)?,
        )?)
    }
}
