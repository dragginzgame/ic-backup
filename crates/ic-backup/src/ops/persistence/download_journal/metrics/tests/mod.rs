use super::*;

#[test]
fn duration_conversion_preserves_exact_nanoseconds_and_rejects_overflow() {
    for (elapsed, expected) in [
        (Duration::ZERO, 0),
        (Duration::from_nanos(17), 17),
        (Duration::from_nanos(u64::MAX), u64::MAX),
    ] {
        let mut metrics = IcSnapshotLocalMetrics::default();
        metrics.record(LocalOperation::UploadData, elapsed, true, Some(1));
        assert_eq!(metrics.upload_data_success_ns().latest(), Some(expected));
        assert_eq!(metrics.prepared_chunk_bytes().maximum(), Some(1));
    }
}

#[test]
fn diagnostics_preserve_the_guard_and_views_send_sync_contract() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<DownloadJournalGuard<'_>>();
    require_send_sync::<IcSnapshotLocalMetrics>();
}

#[test]
fn prepared_chunks_use_inclusive_disjoint_bounds_and_one_aggregate_owner() {
    let mut metrics = IcSnapshotLocalMetrics::default();
    let empty = metrics.prepared_chunk_bytes_histogram();
    assert_eq!(empty.upper_bounds(), &[0, 32_768, 262_144, 1_048_576]);
    assert_eq!(empty.bucket_counts(), &[0; 4]);
    assert_eq!(empty.overflow(), 0);
    assert_eq!(empty.aggregate(), metrics.prepared_chunk_bytes());
    assert_eq!(
        empty.upper_bounds()[3],
        crate::model::ic_snapshot_data::MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64
    );
    for bytes in [
        0, 1, 32_768, 32_769, 262_144, 262_145, 1_048_576, 1_048_577, 0,
    ] {
        // Overflow exercises diagnostic routing; the payload owner rejects it.
        metrics.record(
            LocalOperation::UploadData,
            Duration::ZERO,
            true,
            Some(bytes),
        );
    }
    let histogram = metrics.prepared_chunk_bytes_histogram();
    assert_eq!(histogram.bucket_counts(), &[2, 2, 2, 2]);
    assert_eq!(histogram.overflow(), 1);
    assert_eq!(histogram.aggregate().samples(), 9);
    assert_eq!(histogram.aggregate(), metrics.prepared_chunk_bytes());
    for (operation, succeeded, bytes) in [
        (LocalOperation::UploadData, false, Some(10)),
        (LocalOperation::Verification, true, Some(10)),
        (LocalOperation::UploadMetadata, true, Some(10)),
        (LocalOperation::UploadData, true, None),
    ] {
        metrics.record(operation, Duration::ZERO, succeeded, bytes);
    }
    assert_eq!(metrics.prepared_chunk_bytes_histogram(), histogram);
    assert_eq!(metrics.upload_data_failure_ns().samples(), 1);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn wide_prepared_total_preserves_exact_distribution_and_mean() {
    let mut metrics = IcSnapshotLocalMetrics::default();
    for bytes in [usize::MAX, usize::MAX, 0, 1] {
        metrics.record(
            LocalOperation::UploadData,
            Duration::from_nanos(u64::MAX),
            true,
            Some(bytes),
        );
    }
    let histogram = metrics.prepared_chunk_bytes_histogram();
    assert_eq!(histogram.bucket_counts(), &[1, 1, 0, 0]);
    assert_eq!(histogram.overflow(), 2);
    assert_eq!(histogram.aggregate().samples(), 4);
    let total: u128 = histogram.aggregate().total();
    assert_eq!(total, 2 * u128::from(u64::MAX) + 1);
    assert_eq!(histogram.aggregate(), metrics.prepared_chunk_bytes());
    assert_eq!(
        metrics.upload_data_success_ns().total(),
        4 * u128::from(u64::MAX)
    );
    assert_eq!(metrics.upload_data_success_ns().latest(), Some(u64::MAX));
    assert!(!histogram.aggregate().total_saturated());
    assert!(!histogram.aggregate().samples_saturated());
    assert!(
        !metrics
            .upload_data_success_ns()
            .aggregate()
            .total_saturated()
    );
    assert_eq!(histogram.aggregate().mean(), Ok(Some(total / 4)));
    assert_eq!(
        metrics.upload_data_success_ns().mean(),
        Ok(Some(u128::from(u64::MAX)))
    );
    assert_eq!(histogram.cumulative_count(1), Ok(2));
}

#[test]
fn unrepresentable_durations_leave_each_lane_unchanged_and_preserve_bytes() {
    let mut metrics = IcSnapshotLocalMetrics::default();
    for operation in [
        LocalOperation::Verification,
        LocalOperation::UploadMetadata,
        LocalOperation::UploadData,
    ] {
        for succeeded in [false, true] {
            metrics.record(operation, Duration::MAX, succeeded, None);
        }
    }
    assert_eq!(metrics, IcSnapshotLocalMetrics::default());
    metrics.record(LocalOperation::UploadData, Duration::MAX, true, Some(0));
    assert_eq!(metrics.upload_data_success_ns(), MeasurementSummary::EMPTY);
    assert_eq!(metrics.prepared_chunk_bytes().samples(), 1);
    assert_eq!(metrics.prepared_chunk_bytes().mean(), Ok(Some(0)));
}

#[test]
fn duration_totals_cross_the_sample_cap_without_overflow_in_each_lane() {
    let mut metrics = IcSnapshotLocalMetrics::default();
    for operation in [
        LocalOperation::Verification,
        LocalOperation::UploadMetadata,
        LocalOperation::UploadData,
    ] {
        for succeeded in [false, true] {
            for nanos in [u64::MAX, 0, 1] {
                metrics.record(operation, Duration::from_nanos(nanos), succeeded, None);
            }
        }
    }
    for summary in [
        metrics.verification_success_ns(),
        metrics.verification_failure_ns(),
        metrics.upload_metadata_success_ns(),
        metrics.upload_metadata_failure_ns(),
        metrics.upload_data_success_ns(),
        metrics.upload_data_failure_ns(),
    ] {
        let total: u128 = summary.total();
        let mean: Option<u128> = summary.mean().unwrap();
        assert_eq!(total, u128::from(u64::MAX) + 1);
        assert_eq!(mean, Some(total / 3));
        assert_eq!(summary.samples(), 3);
        assert_eq!(summary.latest(), Some(1));
        assert_eq!(summary.maximum(), Some(u64::MAX));
        assert!(!summary.aggregate().samples_saturated());
        assert!(!summary.aggregate().total_saturated());
    }
    assert_eq!(metrics.prepared_chunk_bytes(), MeasurementAggregate::EMPTY);
}
