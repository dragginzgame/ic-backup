//! Native interface checks over declared bytes; no management behavior is simulated.

use super::*;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_upload::IcSnapshotUploadRequest,
    },
    test_support::ic_snapshot_upload::{
        DESTINATION_ID, SOURCE_ID, TARGET, raw, source_plan, upload_plan,
    },
};
use ic_management_canister_types::SnapshotDataKind;

struct RefusingProvider {
    error: IcMutationProviderError,
    calls: usize,
    expected_binding: ArtifactChecksumRecord,
    expected_arguments: Vec<u8>,
}

impl IcSnapshotUploadProvider for RefusingProvider {
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        self.calls += 1;
        assert_eq!(self.calls, 1, "each independent provider refuses once");
        assert_eq!(request.payload().receiver(), "aaaaa-aa");
        assert_eq!(request.payload().target(), TARGET);
        assert_eq!(request.payload().binding_digest(), self.expected_binding);
        assert_eq!(request.payload().arguments(), self.expected_arguments);
        assert_eq!(request.mutation_attempt(), 1);
        Err(self.error)
    }
}

#[test]
fn upload_port_returns_shared_failures_without_changing_original_accounting() {
    let source_plan = source_plan();
    let read = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&read, &raw("upload-source")).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &source_plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"declared source tree"),
    )
    .unwrap();
    let data = IcSnapshotUploadRequest::data(
        &upload,
        DESTINATION_ID,
        SnapshotDataKind::WasmModule { offset: 5, size: 3 },
        SOURCE_ID,
    )
    .unwrap();
    for payload in [&upload, &data] {
        for error in [
            IcMutationProviderError::Unavailable,
            IcMutationProviderError::Unsupported,
            IcMutationProviderError::Indeterminate,
        ] {
            // Each case has independent original accounting; no failed effect is retried.
            let plan = upload_plan(payload);
            let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
            journal.reserve_mutation().unwrap();
            let before = journal.clone();
            let attempt = IcSnapshotUploadAttempt::new(&plan, 7, &journal, payload).unwrap();
            let mut provider = RefusingProvider {
                error,
                calls: 0,
                expected_binding: payload.binding_digest(),
                expected_arguments: payload.arguments().to_vec(),
            };
            let port: &mut dyn IcSnapshotUploadProvider = &mut provider;
            assert_eq!(port.submit_upload(&attempt).unwrap_err(), error);
            assert_eq!(provider.calls, 1);
            assert_eq!(journal, before);
            assert_eq!(journal.view().pending_mutation, Some(1));
            assert_eq!(journal.view().mutations_remaining, 0);
            assert!(!journal.view().applied);
            assert!(journal.reserve_mutation().is_err());
            assert_eq!(journal, before);
        }
    }
}
