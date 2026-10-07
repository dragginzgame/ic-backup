use super::*;

#[test]
fn duration_conversion_preserves_nanoseconds_and_clamps_overflow() {
    for (elapsed, expected) in [
        (Duration::ZERO, 0),
        (Duration::from_nanos(17), 17),
        (Duration::MAX, u64::MAX),
    ] {
        let mut metrics = IcSnapshotLocalMetrics::default();
        metrics.record(LocalOperation::UploadData, elapsed, true, Some(1));
        assert_eq!(metrics.upload_data_success_ns().latest(), Some(expected));
        assert_eq!(metrics.prepared_chunk_bytes().latest(), Some(1));
    }
}

#[test]
fn diagnostics_preserve_the_guard_and_views_send_sync_contract() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<DownloadJournalGuard<'_>>();
    require_send_sync::<IcSnapshotLocalMetrics>();
}

#[test]
fn prepared_chunks_use_inclusive_disjoint_bounds_and_one_summary_owner() {
    let mut metrics = IcSnapshotLocalMetrics::default();
    let empty = metrics.prepared_chunk_bytes_histogram();
    assert_eq!(empty.upper_bounds(), &[0, 32_768, 262_144, 1_048_576]);
    assert_eq!(empty.bucket_counts(), &[0; 4]);
    assert_eq!(empty.overflow(), 0);
    assert_eq!(empty.summary(), metrics.prepared_chunk_bytes());
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
    assert_eq!(histogram.summary().samples(), 9);
    assert_eq!(histogram.summary().latest(), Some(0));
    assert_eq!(histogram.summary(), metrics.prepared_chunk_bytes());
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
fn saturated_prepared_total_preserves_unsaturated_distribution() {
    let mut metrics = IcSnapshotLocalMetrics::default();
    for bytes in [usize::MAX, usize::MAX, 0, 1] {
        metrics.record(LocalOperation::UploadData, Duration::MAX, true, Some(bytes));
    }
    let histogram = metrics.prepared_chunk_bytes_histogram();
    assert_eq!(histogram.bucket_counts(), &[1, 1, 0, 0]);
    assert_eq!(histogram.overflow(), 2);
    assert_eq!(histogram.summary().samples(), 4);
    assert_eq!(histogram.summary().total(), u64::MAX);
    assert_eq!(histogram.summary(), metrics.prepared_chunk_bytes());
    assert_eq!(metrics.upload_data_success_ns().total(), u64::MAX);
    assert_eq!(metrics.upload_data_success_ns().latest(), Some(u64::MAX));
}
