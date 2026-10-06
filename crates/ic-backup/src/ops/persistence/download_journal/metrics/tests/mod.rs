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
