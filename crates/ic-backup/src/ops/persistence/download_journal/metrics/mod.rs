//! Caller-owned host diagnostics using shared arithmetic; no retained authority.

use super::DownloadJournalGuard;
/// Canonical shared arithmetic for local diagnostic summaries.
///
/// Re-exported so callers can name returned summaries without selecting a separate
/// Metrics dependency. This is the shared type, with no local wrapper or arithmetic.
pub use ic_metrics::MeasurementSummary;
use std::{
    sync::PoisonError,
    time::{Duration, Instant},
};

/// Read-only diagnostic snapshot for one opened download-journal guard.
///
/// Successful and failed returned calls have separate duration summaries in
/// nanoseconds. Verification includes internal upload-preparation checks; preparation
/// durations include those checks and must not be summed with them as exclusive work.
/// Prepared bytes count successful data payloads only, including empty known chunks;
/// repeated preparation records another sample, not unique or transferred bytes.
///
/// Sampling starts empty on create/open and is never persisted or serialized
/// or reconstructed from journal evidence. Counts/totals saturate independently;
/// `u64::MAX` is unavailable for exact interval arithmetic. These values establish
/// no IC cost, complete transfer, spending, receipt, freshness or outcome authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IcSnapshotLocalMetrics {
    verification_success_ns: MeasurementSummary,
    verification_failure_ns: MeasurementSummary,
    upload_metadata_success_ns: MeasurementSummary,
    upload_metadata_failure_ns: MeasurementSummary,
    upload_data_success_ns: MeasurementSummary,
    upload_data_failure_ns: MeasurementSummary,
    prepared_chunk_bytes: MeasurementSummary,
}

impl IcSnapshotLocalMetrics {
    /// Read successful local IC-tree verification durations in nanoseconds.
    #[must_use]
    pub const fn verification_success_ns(self) -> MeasurementSummary {
        self.verification_success_ns
    }
    /// Read rejected local IC-tree verification durations in nanoseconds.
    #[must_use]
    pub const fn verification_failure_ns(self) -> MeasurementSummary {
        self.verification_failure_ns
    }
    /// Read successful local upload-metadata preparation durations in nanoseconds.
    #[must_use]
    pub const fn upload_metadata_success_ns(self) -> MeasurementSummary {
        self.upload_metadata_success_ns
    }
    /// Read rejected local upload-metadata preparation durations in nanoseconds.
    #[must_use]
    pub const fn upload_metadata_failure_ns(self) -> MeasurementSummary {
        self.upload_metadata_failure_ns
    }
    /// Read successful local upload-data preparation durations in nanoseconds.
    #[must_use]
    pub const fn upload_data_success_ns(self) -> MeasurementSummary {
        self.upload_data_success_ns
    }
    /// Read rejected local upload-data preparation durations in nanoseconds.
    #[must_use]
    pub const fn upload_data_failure_ns(self) -> MeasurementSummary {
        self.upload_data_failure_ns
    }
    /// Read successful prepared chunk sizes in bytes, including zero and repeated work.
    #[must_use]
    pub const fn prepared_chunk_bytes(self) -> MeasurementSummary {
        self.prepared_chunk_bytes
    }

    fn record(
        &mut self,
        operation: LocalOperation,
        elapsed: Duration,
        succeeded: bool,
        chunk_bytes: Option<usize>,
    ) {
        let duration = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
        let summary = match (operation, succeeded) {
            (LocalOperation::Verification, true) => &mut self.verification_success_ns,
            (LocalOperation::Verification, false) => &mut self.verification_failure_ns,
            (LocalOperation::UploadMetadata, true) => &mut self.upload_metadata_success_ns,
            (LocalOperation::UploadMetadata, false) => &mut self.upload_metadata_failure_ns,
            (LocalOperation::UploadData, true) => &mut self.upload_data_success_ns,
            (LocalOperation::UploadData, false) => &mut self.upload_data_failure_ns,
        };
        summary.record(duration);
        if let (LocalOperation::UploadData, true, Some(bytes)) = (operation, succeeded, chunk_bytes)
        {
            self.prepared_chunk_bytes
                .record(u64::try_from(bytes).unwrap_or(u64::MAX));
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum LocalOperation {
    Verification,
    UploadMetadata,
    UploadData,
}

impl DownloadJournalGuard<'_> {
    /// Read a copied local diagnostic snapshot without filesystem IO or fresh checks.
    ///
    /// Sampling is per guard lifetime, including rejected calls; ordinary journal
    /// replay never supplies samples. Poison recovery is diagnostic only and cannot
    /// change an operation result. No labels, IDs, byte contents or global registry
    /// are retained. Host monotonic durations measure inclusive local work, not IC
    /// instructions/cycles, unique transfer progress or an authoritative receipt.
    #[must_use]
    pub fn ic_snapshot_metrics(&self) -> IcSnapshotLocalMetrics {
        *self
            .ic_snapshot_metrics
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn record_ic_snapshot_metrics(
        &self,
        operation: LocalOperation,
        started: Instant,
        succeeded: bool,
        chunk_bytes: Option<usize>,
    ) {
        let elapsed = started.elapsed();
        self.ic_snapshot_metrics
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .record(operation, elapsed, succeeded, chunk_bytes);
    }
}

#[cfg(test)]
mod tests;
