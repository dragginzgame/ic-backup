//! Native reserved-observation recovery evidence; no simulated or live IC effects.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_observation::{IcObservationRequest, IcObservationResponse, IcObservationResponseInput},
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        operation_plan::OperationPlanRecord,
        restore_references::RestoreReferencesRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_operation_plan},
    policy::ic_observation::validate_response,
    ports::ic_observation::{IcObservationProvider, IcObservationProviderError},
};
use serde_json::json;
use std::{fs, path::Path};

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
fn payload(method: Method) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: TARGET.into(),
        snapshot_id: (method == Method::LoadCanisterSnapshot).then(|| vec![0, 255, 128]),
    })
    .unwrap()
}
fn wire(method: Method) -> Vec<u8> {
    let file = if method == Method::CanisterStatus {
        include_str!("../src/model/ic_lifecycle_reply/tests/golden.json")
    } else {
        include_str!("../src/model/ic_snapshot_reply/tests/golden.json")
    };
    let cases: Vec<serde_json::Value> = serde_json::from_str(file).unwrap();
    let case = cases
        .iter()
        .find(|case| case["method"] == serde_json::to_value(method).unwrap())
        .unwrap();
    case["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
/// Passive callback fixture records the requested envelope but implements no IC behavior.
struct NativeProvider {
    failure: Option<IcObservationProviderError>,
    calls: Vec<(String, String, String, Vec<u8>, u32, u32)>,
}
struct RetainedEvidence {
    journal: Vec<u8>,
    plan: Vec<u8>,
    references: RestoreReferencesRecord,
}
impl IcObservationProvider for NativeProvider {
    fn observe(
        &mut self,
        request: &IcObservationRequest<'_>,
    ) -> Result<IcObservationResponse, IcObservationProviderError> {
        let payload = request.payload();
        self.calls.push((
            payload.receiver().into(),
            payload.target().into(),
            payload.method().name().into(),
            payload.arguments().into(),
            request.mutation_attempt(),
            request.observation_attempt(),
        ));
        if let Some(error) = self.failure {
            return Err(error);
        }
        IcObservationResponse::new(IcObservationResponseInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            observation_attempt: request.observation_attempt(),
            request: payload.digest(),
            context: request.plan().context().clone(),
            target: payload.target().into(),
            reply: wire(payload.method()),
            evidence: ArtifactChecksumRecord::from_bytes(b"passive native association"),
        })
        .map_err(|_| IcObservationProviderError::Indeterminate)
    }
}

#[test]
fn reserved_observations_and_lost_replies_reopen_without_reissue_or_settlement() {
    for mutation in [
        Method::TakeCanisterSnapshot,
        Method::LoadCanisterSnapshot,
        Method::StartCanister,
        Method::StopCanister,
    ] {
        for method in [Method::CanisterStatus, Method::ListCanisterSnapshots] {
            for failure in [
                None,
                Some(IcObservationProviderError::Unavailable),
                Some(IcObservationProviderError::Unsupported),
                Some(IcObservationProviderError::Indeterminate),
            ] {
                retained_case(mutation, method, failure);
            }
        }
    }
}
fn retained_case(
    mutation_method: Method,
    method: Method,
    failure: Option<IcObservationProviderError>,
) {
    let mutation = payload(mutation_method);
    let observation = payload(method);
    let plan: OperationPlanRecord = serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":TARGET,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[TARGET],"graph":{"version":1,"nodes":[{"operation_sequence":42,"depends_on":[]}]},
        "operations":[{"operation_sequence":42,"target":TARGET,"request":mutation.digest().hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
    })).unwrap();
    let root = support::temp_root("ic-observation");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_operation_plan(&layout, &plan).unwrap();
    fs::write(root.join("mutation.arguments"), mutation.arguments()).unwrap();
    fs::write(root.join("observation.arguments"), observation.arguments()).unwrap();
    // Opaque native retention marker, not a qualified application fence.
    fs::write(
        root.join("obligation.evidence"),
        b"native retained obligation",
    )
    .unwrap();
    layout
        .retain_restore(&root.join("restore-reference.json"), plan.digest().hash())
        .unwrap();
    let references = layout.restore_references().unwrap();
    let authority = plan.attempt_authority(42).unwrap();
    let mut journal = AttemptJournalGuard::create(&layout, authority.clone()).unwrap();
    let mutation_attempt = journal.reserve_mutation().unwrap();
    let observation_attempt = journal
        .reserve_observation(mutation_attempt, observation.digest().hash())
        .unwrap();
    let journal_bytes = fs::read(journal.path()).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let evidence = RetainedEvidence {
        journal: journal_bytes,
        plan: plan_bytes,
        references,
    };
    let request = IcObservationRequest::new(
        &plan,
        42,
        journal.record().unwrap(),
        &mutation,
        &observation,
    )
    .unwrap();
    let mut provider = NativeProvider {
        failure,
        calls: vec![],
    };
    let response = match provider.observe(&request) {
        Ok(response) => {
            validate_response(&request, journal.record().unwrap(), &response).unwrap();
            fs::write(root.join("reply.candid"), &response.input().reply).unwrap();
            Some(response)
        }
        Err(error) => {
            assert_eq!(Some(error), failure);
            None
        }
    };
    assert_eq!(
        provider.calls,
        vec![(
            "aaaaa-aa".into(),
            TARGET.into(),
            method.name().into(),
            observation.arguments().into(),
            mutation_attempt,
            observation_attempt
        )]
    );
    assert_eq!(fs::read(journal.path()).unwrap(), evidence.journal);
    drop(journal);
    drop(layout);
    assert_reopened(&root, &request, &evidence, response.as_ref());
    assert_eq!(provider.calls.len(), 1);
    fs::remove_dir_all(root).unwrap();
}
fn assert_reopened(
    root: &Path,
    original: &IcObservationRequest<'_>,
    evidence: &RetainedEvidence,
    response: Option<&IcObservationResponse>,
) {
    let layout = BackupLayoutGuard::acquire(root).unwrap();
    let mut journal = AttemptJournalGuard::open(&layout, original.authority()).unwrap();
    let request = IcObservationRequest::new(
        original.plan(),
        42,
        journal.record().unwrap(),
        original.mutation(),
        original.payload(),
    )
    .unwrap();
    if let Some(response) = response {
        assert_eq!(
            fs::read(root.join("reply.candid")).unwrap(),
            response.input().reply
        );
        // Local late-reply association makes no provider call and writes no receipt.
        validate_response(&request, journal.record().unwrap(), response).unwrap();
    }
    assert!(journal.reserve_mutation().is_err());
    assert!(
        journal
            .reserve_observation(
                original.mutation_attempt(),
                original.payload().digest().hash()
            )
            .is_err()
    );
    let progress = journal.record().unwrap().view();
    assert_eq!(progress.pending_mutation, Some(original.mutation_attempt()));
    assert_eq!(
        progress.pending_observation,
        Some(original.observation_attempt())
    );
    assert_eq!(
        (
            progress.mutations_remaining,
            progress.observations_remaining
        ),
        (0, 0)
    );
    assert_eq!(
        (progress.mutations_used, progress.observations_used),
        (1, 1)
    );
    assert!(!progress.applied);
    assert_eq!(fs::read(journal.path()).unwrap(), evidence.journal);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        evidence.plan
    );
    assert_eq!(
        fs::read(root.join("mutation.arguments")).unwrap(),
        original.mutation().arguments()
    );
    assert_eq!(
        fs::read(root.join("observation.arguments")).unwrap(),
        original.payload().arguments()
    );
    assert_eq!(
        fs::read(root.join("obligation.evidence")).unwrap(),
        b"native retained obligation"
    );
    assert_eq!(layout.restore_references().unwrap(), evidence.references);
    drop(journal);
    drop(layout);
}
