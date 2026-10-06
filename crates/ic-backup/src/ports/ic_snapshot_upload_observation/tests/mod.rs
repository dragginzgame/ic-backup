use super::*;
use crate::test_support::ic_snapshot_upload::with_observation;

struct RefusingProvider {
    calls: usize,
    error: IcObservationProviderError,
}

impl IcSnapshotUploadObservationProvider for RefusingProvider {
    fn observe_upload(
        &mut self,
        request: &IcSnapshotUploadObservationRequest<'_, '_>,
    ) -> Result<IcObservationResponse, IcObservationProviderError> {
        self.calls += 1;
        assert_eq!(self.calls, 1);
        assert_eq!(
            request.payload().method(),
            crate::model::ic_request::IcManagementMethodRecord::ListCanisterSnapshots
        );
        Err(self.error)
    }
}

#[test]
fn all_existing_provider_failures_leave_original_attempts_and_allowances_pending() {
    for error in [
        IcObservationProviderError::Unavailable,
        IcObservationProviderError::Unsupported,
        IcObservationProviderError::Indeterminate,
    ] {
        with_observation(|plan, upload, list, journal| {
            let before = journal.clone();
            let request =
                IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
            let mut provider = RefusingProvider { calls: 0, error };
            assert_eq!(provider.observe_upload(&request).unwrap_err(), error);
            assert_eq!(provider.calls, 1);
            assert_eq!(journal, &before);
            assert_eq!(journal.view().pending_mutation, Some(1));
            assert_eq!(journal.view().pending_observation, Some(2));
            assert_eq!(journal.view().observations_remaining, 0);
            assert!(!journal.view().applied);
        });
    }
}
