//! Real HTTP admission/cancellation for separately reserved recovery; no management effects.
use super::{Server, TARGET, support};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        ic_observation::IcObservationRequest,
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_data::IcSnapshotDataRequest,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_upload::IcSnapshotUploadRequest,
        ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationRequest,
        ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard},
    ports::{
        ic_observation::{IcObservationProvider, IcObservationProviderError},
        ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationProvider,
        ic_snapshot_upload_observation::IcSnapshotUploadObservationProvider,
    },
};
use ic_backup_agent::{AgentObservationProvider, AgentTransport, PreparedUpdate, ReservedUpdate};
use ic_platform::candid::contract;
use ic_platform::management_canister_types::{SnapshotDataKind, SnapshotSource};
use std::{fs, time::Duration};

enum RecoveryRequest<'a> {
    Original(IcObservationRequest<'a>),
    Upload(IcSnapshotUploadObservationRequest<'a, 'a>),
    Data(IcSnapshotUploadDataObservationRequest<'a, 'a, 'a>),
}
impl RecoveryRequest<'_> {
    async fn observe<R>(
        &self,
        provider: &mut AgentObservationProvider<'_, R>,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcObservationProviderError>
    where
        R: FnMut(
            &ReservedUpdate<'_>,
            &PreparedUpdate<'_>,
        ) -> Result<(), IcObservationProviderError>,
    {
        match self {
            Self::Original(r) => provider.observe(r, journal).await.map(|_| ()),
            Self::Upload(r) => provider.observe_upload(r, journal).await.map(|_| ()),
            Self::Data(r) => provider.observe_upload_data(r, journal).await.map(|_| ()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecoveryLane {
    Status,
    Capture,
    Allocation,
    Data,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecoveryMode {
    RefuseRetention,
    Accepted,
    Disconnect,
    Cancel,
}
impl RecoveryMode {
    const ALL: [Self; 4] = [
        Self::RefuseRetention,
        Self::Accepted,
        Self::Disconnect,
        Self::Cancel,
    ];
}

#[expect(
    clippy::too_many_lines,
    reason = "one chronological original-journal and signed-body trace covers all four recovery payloads and interruption modes"
)]
async fn exercise(lane: RecoveryLane, mode: RecoveryMode) {
    let root = support::root("recovery-http");
    let mut server = Server::new(
        &root,
        "202 Accepted",
        vec![],
        mode == RecoveryMode::Disconnect,
        mode == RecoveryMode::Cancel,
    );
    let metadata_request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(
        &metadata_request,
        include_bytes!("../upload-metadata.candid"),
    )
    .unwrap();
    let source = support::plan(TARGET, &metadata_request.digest(), 11);
    let upload = IcSnapshotUploadRequest::metadata(
        &source,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"declared tree"),
    )
    .unwrap();
    let kind = SnapshotDataKind::WasmModule { offset: 0, size: 3 };
    let data =
        IcSnapshotUploadRequest::data(&upload, &[9, 0, 255], kind.clone(), &[7, 8, 9]).unwrap();
    let mut values = metadata.metadata().clone();
    values.source = Some(SnapshotSource::MetadataUpload(contract::Reserved));
    let destination_request = IcSnapshotMetadataRequest::new(TARGET, &[9, 0, 255]).unwrap();
    let destination = IcSnapshotMetadataReply::decode(
        &destination_request,
        &contract::encode_one(values).unwrap(),
    )
    .unwrap();
    let read = IcSnapshotDataRequest::new(&destination, kind).unwrap();
    let mutation = support::payload(
        if lane == RecoveryLane::Status {
            Method::StopCanister
        } else {
            Method::TakeCanisterSnapshot
        },
        TARGET,
        None,
    );
    let observation = support::payload(
        if lane == RecoveryLane::Status {
            Method::CanisterStatus
        } else {
            Method::ListCanisterSnapshots
        },
        TARGET,
        None,
    );
    let (original, digest) = match lane {
        RecoveryLane::Status | RecoveryLane::Capture => (mutation.digest(), observation.digest()),
        RecoveryLane::Allocation => (upload.binding_digest(), observation.digest()),
        RecoveryLane::Data => (data.binding_digest(), read.digest()),
    };
    let plan = support::plan(TARGET, &original, 1);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let mut guard = support::retain(&layout, &plan, 1);
    guard.reserve_observation(1, digest.hash()).unwrap();
    let before = fs::read(guard.path()).unwrap();
    let request = match lane {
        RecoveryLane::Status | RecoveryLane::Capture => RecoveryRequest::Original(
            IcObservationRequest::new(&plan, 1, guard.record().unwrap(), &mutation, &observation)
                .unwrap(),
        ),
        RecoveryLane::Allocation => RecoveryRequest::Upload(
            IcSnapshotUploadObservationRequest::new(
                &plan,
                1,
                guard.record().unwrap(),
                &upload,
                &observation,
            )
            .unwrap(),
        ),
        RecoveryLane::Data => RecoveryRequest::Data(
            IcSnapshotUploadDataObservationRequest::new(
                &plan,
                1,
                guard.record().unwrap(),
                &data,
                &read,
            )
            .unwrap(),
        ),
    };
    let transport = AgentTransport::new(
        plan.context().clone(),
        &server.endpoint,
        support::identity(),
        vec![1],
        Duration::from_secs(1),
    )
    .unwrap();
    let mut provider = AgentObservationProvider::new(
        &transport,
        |actual: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            let authority = match actual {
                ReservedUpdate::Observation(r) => r.authority(),
                ReservedUpdate::UploadObservation(r) => r.authority(),
                ReservedUpdate::UploadDataObservation(r) => r.authority(),
                _ => panic!("only separately reserved recovery reaches this adapter"),
            };
            assert_eq!(
                authority.digest(),
                plan.attempt_authority(1).unwrap().digest()
            );
            assert!(AttemptJournalGuard::open(&layout, authority).is_err());
            if mode == RecoveryMode::RefuseRetention || root.join("signed-ingress.cbor").exists() {
                return Err(IcObservationProviderError::Unavailable);
            }
            assert!(server.requests.lock().unwrap().is_empty());
            support::retain_signed(&root, prepared)
                .map_err(|_| IcObservationProviderError::Unavailable)
        },
    );
    let stale = AttemptJournalRecord::new(plan.attempt_authority(1).unwrap());
    assert_eq!(
        request.observe(&mut provider, &stale).await.unwrap_err(),
        IcObservationProviderError::Unsupported
    );
    assert!(!root.join("signed-ingress.cbor").exists());
    if mode == RecoveryMode::Cancel {
        let mut pending = Box::pin(request.observe(&mut provider, guard.record().unwrap()));
        let sent = async {
            tokio::time::timeout(Duration::from_secs(2), async {
                while server.requests.lock().unwrap().is_empty() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
        };
        tokio::select! {
            result = &mut pending => panic!("delayed original finished before cancellation: {result:?}"),
            () = sent => {}
        }
        drop(pending);
    } else {
        assert_eq!(
            request
                .observe(&mut provider, guard.record().unwrap())
                .await
                .unwrap_err(),
            if mode == RecoveryMode::RefuseRetention {
                IcObservationProviderError::Unavailable
            } else {
                IcObservationProviderError::Indeterminate
            }
        );
    }
    // The mandatory retainer rejects prior dispatch, without another HTTP request.
    assert_eq!(
        request
            .observe(&mut provider, guard.record().unwrap())
            .await
            .unwrap_err(),
        IcObservationProviderError::Unavailable
    );
    server.finish().unwrap();
    assert_eq!(
        server.requests.lock().unwrap().len(),
        usize::from(mode != RecoveryMode::RefuseRetention)
    );
    assert_eq!(fs::read(guard.path()).unwrap(), before);
    assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
    assert_eq!(guard.record().unwrap().view().pending_observation, Some(2));
    if mode != RecoveryMode::RefuseRetention {
        let requests = server.requests.lock().unwrap();
        let bytes = requests[0].complete_bytes(&root);
        let end = bytes.windows(4).position(|p| p == b"\r\n\r\n").unwrap();
        assert_eq!(
            &bytes[end + 4..],
            fs::read(root.join("signed-ingress.cbor")).unwrap()
        );
    }
}

#[tokio::test]
async fn status_recovery_preserves_both_attempts_and_exact_zero_or_one_submission() {
    for mode in RecoveryMode::ALL {
        exercise(RecoveryLane::Status, mode).await;
    }
}
#[tokio::test]
async fn capture_list_recovery_preserves_both_attempts_and_exact_zero_or_one_submission() {
    for mode in RecoveryMode::ALL {
        exercise(RecoveryLane::Capture, mode).await;
    }
}
#[tokio::test]
async fn allocation_list_recovery_preserves_both_attempts_and_exact_zero_or_one_submission() {
    for mode in RecoveryMode::ALL {
        exercise(RecoveryLane::Allocation, mode).await;
    }
}
#[tokio::test]
async fn data_readback_recovery_preserves_both_attempts_and_exact_zero_or_one_submission() {
    for mode in RecoveryMode::ALL {
        exercise(RecoveryLane::Data, mode).await;
    }
}
