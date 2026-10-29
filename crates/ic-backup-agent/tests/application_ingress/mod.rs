//! Real HTTP failure/cancellation and original-reservation admission; no simulated IC effects.
use super::{Server, TARGET, support};
use ic_backup::{
    model::{
        application_ingress::{
            ApplicationIngressPayload, ApplicationMutationRequest, ApplicationObservationRequest,
        },
        attempt_journal::AttemptJournalRecord,
        operation_plan::{PlanContextRecord, PlanContextRequest},
    },
    ops::persistence::BackupLayoutGuard,
};
use ic_backup_agent::{
    AgentApplicationIngressProvider, AgentTransport, ApplicationIngressProviderError as Error,
    PreparedUpdate, ReservedUpdate,
};
use ic_platform::agent::RequestId;
use std::{fs, sync::Arc, time::Duration};

#[tokio::test]
async fn application_updates_and_recovery_retain_once_and_preserve_pending_on_every_failure() {
    for recovery in [false, true] {
        for mode in ["refuse", "accepted", "lost", "cancelled"] {
            exercise(recovery, mode).await;
        }
    }
}
async fn exercise(recovery: bool, mode: &str) {
    let root = support::root("application-http");
    let mut server = Server::new(
        &root,
        "202 Accepted",
        vec![],
        mode == "lost",
        mode == "cancelled",
    );
    let payload =
        ApplicationIngressPayload::new(TARGET, TARGET, "apply", b"original opaque bytes", 128)
            .unwrap();
    let read =
        ApplicationIngressPayload::new(TARGET, TARGET, "observe", b"original challenge", 128)
            .unwrap();
    let plan = support::plan(TARGET, &payload.digest(), 1);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let mut guard = support::retain(&layout, &plan, 1);
    if recovery {
        guard
            .reserve_planned_observation(&plan.digest(), 1, read.digest().hash())
            .unwrap();
    }
    let before = fs::read(guard.path()).unwrap();
    let transport = AgentTransport::new(
        plan.context().clone(),
        &server.endpoint,
        support::identity(),
        vec![1],
        Duration::from_secs(2),
    )
    .unwrap();
    let mut provider = AgentApplicationIngressProvider::new(
        &transport,
        |actual: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            assert_eq!(
                matches!(actual, ReservedUpdate::ApplicationObservation(_)),
                recovery
            );
            if mode == "refuse" || root.join("signed-ingress.cbor").exists() {
                return Err(Error::Unavailable);
            }
            support::retain_signed(&root, prepared).map_err(|_| Error::Unavailable)
        },
        |_: &ReservedUpdate<'_>, _: &RequestId, _: &[u8]| {
            panic!("accepted/lost/cancelled calls have no complete authenticated reply")
        },
    );
    let submit = async {
        if recovery {
            let request =
                ApplicationObservationRequest::new(&plan, 1, guard.record().unwrap(), &read)
                    .unwrap();
            provider
                .submit_observation(&request, guard.record().unwrap())
                .await
        } else {
            let request =
                ApplicationMutationRequest::new(&plan, 1, guard.record().unwrap(), &payload)
                    .unwrap();
            provider
                .submit_mutation(&request, guard.record().unwrap())
                .await
        }
    };
    finish_call(submit, mode, &server.requests).await;
    let result = if recovery {
        let request =
            ApplicationObservationRequest::new(&plan, 1, guard.record().unwrap(), &read).unwrap();
        provider
            .submit_observation(&request, guard.record().unwrap())
            .await
    } else {
        let request =
            ApplicationMutationRequest::new(&plan, 1, guard.record().unwrap(), &payload).unwrap();
        provider
            .submit_mutation(&request, guard.record().unwrap())
            .await
    };
    assert_eq!(result.unwrap_err(), Error::Unavailable);
    server.finish().unwrap();
    assert_eq!(
        server.requests.lock().unwrap().len(),
        usize::from(mode != "refuse")
    );
    if mode != "refuse" {
        assert_signed_request(&server, &root);
    }
    assert_eq!(fs::read(guard.path()).unwrap(), before);
    assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
    assert_eq!(
        guard.record().unwrap().view().pending_observation.is_some(),
        recovery
    );
}

async fn finish_call(
    call: impl std::future::Future<Output = Result<ic_backup_agent::ApplicationIngressResponse, Error>>,
    mode: &str,
    requests: &std::sync::Mutex<Vec<super::http_capture::CapturedRequest>>,
) {
    let mut call = Box::pin(call);
    if mode == "cancelled" {
        tokio::select! {
            result = &mut call => panic!("original completed before cancellation: {result:?}"),
            () = async {
                tokio::time::timeout(Duration::from_secs(2), async {
                    while requests.lock().unwrap().is_empty() { tokio::time::sleep(Duration::from_millis(5)).await; }
                }).await.unwrap();
            } => {}
        }
    } else {
        assert_eq!(
            call.await.unwrap_err(),
            if mode == "refuse" {
                Error::Unavailable
            } else {
                Error::Indeterminate
            }
        );
    }
}

#[tokio::test]
async fn changed_context_and_original_journal_refuse_before_signing_or_http() {
    let root = support::root("application-admission");
    let mut server = Server::new(&root, "202 Accepted", vec![], false, false);
    let payload = ApplicationIngressPayload::new(TARGET, TARGET, "apply", b"opaque", 128).unwrap();
    let plan = support::plan(TARGET, &payload.digest(), 1);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let guard = support::retain(&layout, &plan, 1);
    let request =
        ApplicationMutationRequest::new(&plan, 1, guard.record().unwrap(), &payload).unwrap();
    assert!(matches!(
        AgentTransport::new(
            plan.context().clone(),
            &server.endpoint,
            Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(&[7; 32])),
            vec![1],
            Duration::from_secs(1)
        ),
        Err(ic_backup_agent::TransportError::Context)
    ));
    let changed = PlanContextRecord::new(&PlanContextRequest {
        network: "aa".repeat(32),
        caller: "2vxsx-fae".into(),
        release: plan.context().release().into(),
    })
    .unwrap();
    for (is_changed, context) in [(true, changed), (false, plan.context().clone())] {
        let transport = AgentTransport::new(
            context,
            &server.endpoint,
            support::identity(),
            vec![1],
            Duration::from_secs(1),
        )
        .unwrap();
        let mut provider = AgentApplicationIngressProvider::new(
            &transport,
            |_: &ReservedUpdate<'_>, _: &PreparedUpdate<'_>| panic!("rejected before retention"),
            |_: &ReservedUpdate<'_>, _: &RequestId, _: &[u8]| panic!("rejected before dispatch"),
        );
        let stale = AttemptJournalRecord::new(plan.attempt_authority(1).unwrap());
        assert_eq!(
            provider
                .submit_mutation(&request, &stale)
                .await
                .unwrap_err(),
            Error::Unsupported
        );
        if is_changed {
            assert_eq!(
                provider
                    .submit_mutation(&request, guard.record().unwrap())
                    .await
                    .unwrap_err(),
                Error::Unsupported
            );
        }
    }
    server.finish().unwrap();
    assert!(server.requests.lock().unwrap().is_empty());
}

fn assert_signed_request(server: &Server, root: &std::path::Path) {
    let requests = server.requests.lock().unwrap();
    let bytes = requests[0].complete_bytes(root);
    let body = bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .unwrap()
        + 4;
    assert_eq!(
        &bytes[body..],
        fs::read(root.join("signed-ingress.cbor")).unwrap()
    );
}
