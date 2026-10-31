//! Real HTTP retention/cancellation of an opaque application update; no acquisition semantics.
use super::{Server, TARGET, support};
use ic_backup::{
    model::{
        attempt_journal::AttemptJournalRecord, fence_acquisition::FenceAcquisitionPayload,
        fence_acquisition::FenceAcquisitionRequest,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard},
    ports::fence_acquisition::{FenceAcquisitionProvider, FenceAcquisitionProviderError},
};
use ic_backup_agent::{AgentFenceAcquisitionProvider, AgentTransport, PreparedUpdate};
use std::{fs, time::Duration};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FenceFailure {
    RefuseIngress,
    Accepted,
    Disconnect,
    Cancel,
}
impl FenceFailure {
    const ALL: [Self; 4] = [
        Self::RefuseIngress,
        Self::Accepted,
        Self::Disconnect,
        Self::Cancel,
    ];
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "keep original custody and exact HTTP cancellation trace chronological"
)]
async fn original_fence_update_requires_retention_and_never_reissues_after_cancellation() {
    for mode in FenceFailure::ALL {
        let root = support::root("fence-http");
        let mut server = Server::new(
            &root,
            "202 Accepted",
            vec![],
            mode == FenceFailure::Disconnect,
            mode == FenceFailure::Cancel,
        );
        let payload =
            FenceAcquisitionPayload::new(TARGET, "acquire_fixture", b"original opaque unit")
                .unwrap();
        let plan = support::plan(TARGET, &payload.digest(), 1);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let obligation = retain_fence_originals(&layout, &plan, 1);
        let original_obligation = fs::read(root.join("fence-obligation.json")).unwrap();
        let mut guard =
            AttemptJournalGuard::create(&layout, plan.attempt_authority(1).unwrap()).unwrap();
        guard.reserve_planned_mutation(&plan.digest()).unwrap();
        let before = fs::read(guard.path()).unwrap();
        let request =
            FenceAcquisitionRequest::new(&plan, &obligation, guard.record().unwrap(), &payload)
                .unwrap();
        let transport = AgentTransport::new(
            plan.context().clone(),
            &server.endpoint,
            support::identity(),
            vec![1],
            Duration::from_secs(1),
        )
        .unwrap();
        let mut provider = AgentFenceAcquisitionProvider::new(
            &transport,
            |actual: &FenceAcquisitionRequest<'_>, prepared: &PreparedUpdate<'_>| {
                assert_eq!(actual.obligation(), request.obligation());
                assert!(AttemptJournalGuard::open(&layout, actual.authority()).is_err());
                if mode == FenceFailure::RefuseIngress || root.join("signed-ingress.cbor").exists()
                {
                    return Err(FenceAcquisitionProviderError::Unavailable);
                }
                assert!(server.requests.lock().unwrap().is_empty());
                support::retain_signed(&root, prepared)
                    .map_err(|_| FenceAcquisitionProviderError::Unavailable)
            },
            |_: &FenceAcquisitionRequest<'_>, _: &ic_platform::agent::RequestId, _: &[u8]| {
                panic!("accepted/lost/cancelled replies cannot reach reply retention")
            },
        );
        let stale = AttemptJournalRecord::new(plan.attempt_authority(1).unwrap());
        assert_eq!(
            provider.acquire_fence(&request, &stale).await.unwrap_err(),
            FenceAcquisitionProviderError::Unsupported
        );
        assert!(!root.join("signed-ingress.cbor").exists());
        if mode == FenceFailure::Cancel {
            let mut pending = Box::pin(provider.acquire_fence(&request, guard.record().unwrap()));
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
                provider
                    .acquire_fence(&request, guard.record().unwrap())
                    .await
                    .unwrap_err(),
                if mode == FenceFailure::RefuseIngress {
                    FenceAcquisitionProviderError::Unavailable
                } else {
                    FenceAcquisitionProviderError::Indeterminate
                }
            );
        }
        assert_eq!(
            provider
                .acquire_fence(&request, guard.record().unwrap())
                .await
                .unwrap_err(),
            FenceAcquisitionProviderError::Unavailable
        );
        server.finish().unwrap();
        assert_eq!(
            server.requests.lock().unwrap().len(),
            usize::from(mode != FenceFailure::RefuseIngress)
        );
        assert_eq!(fs::read(guard.path()).unwrap(), before);
        assert_eq!(
            fs::read(root.join("fence-obligation.json")).unwrap(),
            original_obligation
        );
        assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
        if mode != FenceFailure::RefuseIngress {
            let requests = server.requests.lock().unwrap();
            let bytes = requests[0].complete_bytes(&root);
            let end = bytes.windows(4).position(|p| p == b"\r\n\r\n").unwrap();
            assert_eq!(
                &bytes[end + 4..],
                fs::read(root.join("signed-ingress.cbor")).unwrap()
            );
        }
    }
}

/// Retain declared capture obligations before any acquisition reservation; no Active fence is proven.
fn retain_fence_originals(
    layout: &BackupLayoutGuard,
    plan: &ic_backup::model::operation_plan::OperationPlanRecord,
    sequence: u64,
) -> ic_backup::model::fence_obligation::FenceObligationRecord {
    use ic_backup::{
        model::{
            artifacts::ArtifactChecksumRecord,
            consistency::{
                ApplicationFenceBinding, ConsistencyGuaranteeRecord, ConsistencyRequirementRecord,
            },
            fence_obligation::FenceObligationRecord,
        },
        ops::persistence::{
            FenceObligationRequirement, create_consistency_requirement, create_fence_obligation,
            create_operation_plan,
        },
    };
    create_operation_plan(layout, plan).unwrap();
    let requirement =
        ConsistencyRequirementRecord::new(plan, ConsistencyGuaranteeRecord::ApplicationCoordinated);
    create_consistency_requirement(layout, plan, &requirement).unwrap();
    let fence = ApplicationFenceBinding {
        identity: ArtifactChecksumRecord::from_bytes(b"fixture fence declaration"),
        membership_revision: ArtifactChecksumRecord::from_bytes(b"fixture membership declaration"),
    };
    let obligation =
        FenceObligationRecord::for_capture(plan, &requirement, sequence, &fence).unwrap();
    create_fence_obligation(
        layout,
        plan,
        FenceObligationRequirement::Capture {
            requirement: &requirement,
            fence: &fence,
        },
        &obligation,
    )
    .unwrap();
    obligation
}
