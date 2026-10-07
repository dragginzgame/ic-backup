//! Real original lifecycle settlement, independently qualified before recording.

use super::*;
use crate::pic_journey::{backend::Backend, management};
use ic_backup::{
    model::attempt_journal::AttemptAuthorityRecord,
    model::{
        ic_lifecycle_reply::{IcLifecycleReply, IcLifecycleReplyKind},
        ic_observation::{IcLifecycleAttribution, IcLifecycleSettlement},
        ic_request::IcManagementMethodRecord as Method,
    },
    ops::persistence::BackupLayoutGuard,
    policy::ic_observation::validate_lifecycle_settlement,
};
use ic_management_canister_types::CanisterStatusType;

pub(in crate::pic_journey) fn lifecycle(
    backend: &mut Backend,
    index: u64,
    mutation: &IcManagementRequestRecord,
    discard_observation: bool,
    verify: impl FnOnce(&mut Backend) -> Option<ArtifactChecksumRecord>,
) -> bool {
    let plan = backend.retained_plan(index);
    let layout = backend.layout(index);
    let mut journal =
        AttemptJournalGuard::open(&layout, &plan.attempt_authority(index).unwrap()).unwrap();
    assert!(matches!(
        journal.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { attempt: 1 }
        ))
    ));
    let status = management(backend, Method::CanisterStatus, None);
    fs::write(
        layout.root().join("observation-arguments.candid"),
        status.arguments(),
    )
    .unwrap();
    let observation = journal
        .reserve_observation(1, status.digest().hash())
        .unwrap();
    let request =
        IcObservationRequest::new(&plan, index, journal.record().unwrap(), mutation, &status)
            .unwrap();
    let raw = backend.management(status.method().name(), status.arguments());
    if discard_observation {
        assert_eq!(mutation.method(), Method::LoadCanisterSnapshot);
        fs::write(
            layout.root().join("discarded-status-oracle-reply.candid"),
            &raw,
        )
        .unwrap();
        // Only the test oracle reads this response to assert actual simulator state.
        // No response/settlement claim is constructed and no receipt is recorded.
        assert_status(&status, &raw, CanisterStatusType::Stopped);
        return false;
    }
    fs::write(layout.root().join("observation-reply.candid"), &raw).unwrap();
    let state = if mutation.method() == Method::StartCanister {
        CanisterStatusType::Running
    } else {
        CanisterStatusType::Stopped
    };
    assert_status(&status, &raw, state);
    let decoded = IcLifecycleReply::decode(&status, &raw).unwrap();
    let restored = verify(backend);
    assert_eq!(
        restored.is_some(),
        mutation.method() == Method::LoadCanisterSnapshot
    );
    let association = backend.recovery_evidence(&plan, index, "exclusive original lifecycle ingress; load additionally matches complete freshly captured stopped state");
    let challenge = ArtifactChecksumRecord::from_bytes(
        format!("fresh lifecycle recovery {}:{index}", plan.digest().hash()).as_bytes(),
    );
    let evidence = ArtifactChecksumRecord::from_bytes(&serde_json::to_vec(&serde_json::json!({"association":association,"restored_state":restored,"authority":request.authority().digest(),"mutation":1,"observation":observation,"status":decoded.digest(),"challenge":challenge})).unwrap());
    let response = IcObservationResponse::new(IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: 1,
        observation_attempt: observation,
        request: status.digest(),
        context: plan.context().clone(),
        target: backend.target.to_text(),
        reply: raw,
        evidence: association.clone(),
    })
    .unwrap();
    let settlement = IcLifecycleSettlement {
        authority: request.authority().digest(),
        mutation_attempt: 1,
        observation_attempt: observation,
        challenge: challenge.clone(),
        status: decoded.digest(),
        observation_evidence: association,
        attribution: IcLifecycleAttribution::Applied {
            attribution: evidence.clone(),
        },
        evidence,
    };
    let view = validate_lifecycle_settlement(
        &request,
        journal.record().unwrap(),
        &response,
        &challenge,
        &settlement,
    )
    .unwrap();
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: observation,
            request: status.digest().hash().into(),
            outcome: view.outcome(),
            evidence: settlement.evidence.hash().into(),
        })
        .unwrap();
    assert_settled_reopen(&layout, request.authority(), journal);
    true
}

fn assert_status(request: &IcManagementRequestRecord, raw: &[u8], expected: CanisterStatusType) {
    let decoded = IcLifecycleReply::decode(request, raw).unwrap();
    let IcLifecycleReplyKind::Status(info) = decoded.kind() else {
        panic!("status reply")
    };
    assert_eq!(info.status(), expected);
    assert_eq!(
        info.controllers().principals(),
        [candid::Principal::anonymous().to_text()]
    );
}

fn assert_settled_reopen(
    layout: &BackupLayoutGuard,
    authority: &AttemptAuthorityRecord,
    mut journal: AttemptJournalGuard<'_>,
) {
    let view = journal.record().unwrap().view();
    assert!(view.applied);
    assert_eq!(
        (view.mutations_remaining, view.observations_remaining),
        (0, 0)
    );
    assert_eq!(
        (view.pending_mutation, view.pending_observation),
        (None, None)
    );
    let bytes = fs::read(journal.path()).unwrap();
    assert!(matches!(
        journal.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::AlreadyApplied
        ))
    ));
    assert_eq!(fs::read(journal.path()).unwrap(), bytes);
    drop(journal);
    let reopened = AttemptJournalGuard::open(layout, authority).unwrap();
    assert_eq!(fs::read(reopened.path()).unwrap(), bytes);
    assert_eq!(reopened.record().unwrap().view(), view);
}
