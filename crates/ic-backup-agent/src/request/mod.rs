//! Admission of existing reserved payload owners, without another wire encoder.

use ic_backup::model::{
    attempt_journal::AttemptJournalRecord,
    fence_acquisition::FenceAcquisitionRequest,
    ic_mutation::{IcMutationRequest, MAX_IC_MUTATION_REPLY_BYTES},
    ic_observation::{IcObservationRequest, MAX_IC_OBSERVATION_REPLY_BYTES},
    ic_snapshot_data::MAX_IC_SNAPSHOT_DATA_REPLY_BYTES,
    ic_snapshot_metadata::MAX_IC_SNAPSHOT_METADATA_BYTES,
    ic_snapshot_transfer_read::{IcSnapshotTransferReadPayload, IcSnapshotTransferReadRequest},
    ic_snapshot_upload::{IcSnapshotUploadAttempt, MAX_IC_SNAPSHOT_UPLOAD_REPLY_BYTES},
    ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationRequest,
    ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest,
    operation_plan::PlanContextRecord,
};

/// An exact existing request with its already consumed original reservation.
///
/// No branch proves fresh permission, application safety or never-dispatched custody.
/// Recovery observations consume their independently reserved observation allowance.
#[derive(Debug)]
pub enum ReservedUpdate<'a> {
    /// Capture, stop, load or start.
    Mutation(&'a IcMutationRequest<'a>),
    /// Original mutation recovery status/list.
    Observation(&'a IcObservationRequest<'a>),
    /// One metadata or data read, using replicated update ingress.
    TransferRead(&'a IcSnapshotTransferReadRequest<'a, 'a>),
    /// One metadata allocation or data write.
    Upload(&'a IcSnapshotUploadAttempt<'a, 'a>),
    /// Original metadata allocation recovery list.
    UploadObservation(&'a IcSnapshotUploadObservationRequest<'a, 'a>),
    /// Original data write recovery readback.
    UploadDataObservation(&'a IcSnapshotUploadDataObservationRequest<'a, 'a, 'a>),
    /// One application-owned fence acquisition update.
    FenceAcquisition(&'a FenceAcquisitionRequest<'a>),
}

pub(super) struct Wire<'a> {
    pub context: &'a PlanContextRecord,
    pub receiver: &'a str,
    pub target: &'a str,
    pub method: &'a str,
    pub arguments: &'a [u8],
    pub reply_limit: usize,
}

impl ReservedUpdate<'_> {
    pub(super) fn validate(&self, journal: &AttemptJournalRecord) -> bool {
        match self {
            Self::Mutation(r) => r.validate_journal(journal).is_ok(),
            Self::Observation(r) => r.validate_journal(journal).is_ok(),
            Self::TransferRead(r) => r.validate_journal(journal).is_ok(),
            Self::Upload(r) => r.validate_journal(journal).is_ok(),
            Self::UploadObservation(r) => r.validate_journal(journal).is_ok(),
            Self::UploadDataObservation(r) => r.validate_journal(journal).is_ok(),
            Self::FenceAcquisition(r) => r.validate_journal(journal).is_ok(),
        }
    }

    pub(super) fn wire(&self) -> Wire<'_> {
        match self {
            Self::Mutation(r) => Wire {
                context: r.plan().context(),
                receiver: r.payload().receiver(),
                target: r.payload().target(),
                method: r.payload().method().name(),
                arguments: r.payload().arguments(),
                reply_limit: MAX_IC_MUTATION_REPLY_BYTES,
            },
            Self::Observation(r) => Wire {
                context: r.plan().context(),
                receiver: r.payload().receiver(),
                target: r.payload().target(),
                method: r.payload().method().name(),
                arguments: r.payload().arguments(),
                reply_limit: MAX_IC_OBSERVATION_REPLY_BYTES,
            },
            Self::TransferRead(r) => match r.payload() {
                IcSnapshotTransferReadPayload::Metadata(p) => Wire {
                    context: r.plan().context(),
                    receiver: p.receiver(),
                    target: p.target(),
                    method: p.method(),
                    arguments: p.arguments(),
                    reply_limit: MAX_IC_SNAPSHOT_METADATA_BYTES,
                },
                IcSnapshotTransferReadPayload::Data(p) => Wire {
                    context: r.plan().context(),
                    receiver: p.receiver(),
                    target: p.target(),
                    method: p.method(),
                    arguments: p.arguments(),
                    reply_limit: MAX_IC_SNAPSHOT_DATA_REPLY_BYTES,
                },
            },
            Self::Upload(r) => Wire {
                context: r.plan().context(),
                receiver: r.payload().receiver(),
                target: r.payload().target(),
                method: r.payload().method(),
                arguments: r.payload().arguments(),
                reply_limit: MAX_IC_SNAPSHOT_UPLOAD_REPLY_BYTES,
            },
            Self::UploadObservation(r) => Wire {
                context: r.plan().context(),
                receiver: r.payload().receiver(),
                target: r.payload().target(),
                method: r.payload().method().name(),
                arguments: r.payload().arguments(),
                reply_limit: MAX_IC_OBSERVATION_REPLY_BYTES,
            },
            Self::UploadDataObservation(r) => Wire {
                context: r.plan().context(),
                receiver: r.payload().receiver(),
                target: r.payload().target(),
                method: r.payload().method(),
                arguments: r.payload().arguments(),
                reply_limit: MAX_IC_SNAPSHOT_DATA_REPLY_BYTES,
            },
            Self::FenceAcquisition(r) => Wire {
                context: r.plan().context(),
                receiver: r.payload().target(),
                target: r.payload().target(),
                method: r.payload().method(),
                arguments: r.payload().arguments(),
                reply_limit: crate::MAX_APPLICATION_REPLY_BYTES,
            },
        }
    }
}
