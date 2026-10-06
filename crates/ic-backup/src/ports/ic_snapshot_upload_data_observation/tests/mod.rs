use super::*;
use crate::test_support::ic_snapshot_upload_data_observation::with_original;
use ic_management_canister_types::SnapshotDataKind;

struct RefusingProvider {
    calls: usize,
    error: IcObservationProviderError,
}
impl IcSnapshotUploadDataObservationProvider for RefusingProvider {
    fn observe_upload_data(
        &mut self,
        request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
    ) -> Result<IcSnapshotUploadDataObservationResponse, IcObservationProviderError> {
        self.calls += 1;
        assert_eq!(self.calls, 1);
        assert_eq!(request.payload().method(), "read_canister_snapshot_data");
        Err(self.error)
    }
}

#[test]
fn all_existing_provider_failures_retain_both_original_pending_attempts() {
    for error in [
        IcObservationProviderError::Unavailable,
        IcObservationProviderError::Unsupported,
        IcObservationProviderError::Indeterminate,
    ] {
        with_original(
            SnapshotDataKind::WasmModule { offset: 1, size: 2 },
            &[42; 2],
            |plan, upload, read, journal| {
                let before = journal.clone();
                let request =
                    IcSnapshotUploadDataObservationRequest::new(plan, 7, journal, upload, read)
                        .unwrap();
                let mut provider = RefusingProvider { calls: 0, error };
                assert_eq!(provider.observe_upload_data(&request).unwrap_err(), error);
                assert_eq!(provider.calls, 1);
                assert_eq!(journal, &before);
                assert_eq!(
                    (
                        journal.view().pending_mutation,
                        journal.view().pending_observation
                    ),
                    (Some(1), Some(2))
                );
                assert_eq!(
                    (
                        journal.view().mutations_remaining,
                        journal.view().observations_remaining
                    ),
                    (0, 0)
                );
                assert!(!journal.view().applied);
            },
        );
    }
}
