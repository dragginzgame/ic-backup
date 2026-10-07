//! Native reserved-observation recovery evidence; no simulated or live IC effects.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{
            AttemptJournalRecord, ObservationOutcomeRecord, ObservationReceiptRequest,
        },
        ic_lifecycle_reply::IcLifecycleReply,
        ic_observation::{
            IcCaptureAttribution, IcCaptureSettlement, IcLifecycleAttribution,
            IcLifecycleSettlement, IcObservationRequest, IcObservationResponse,
            IcObservationResponseInput,
        },
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
        operation_plan::OperationPlanRecord,
        restore_references::RestoreReferencesRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_operation_plan},
    policy::ic_observation::{
        validate_capture_settlement, validate_lifecycle_settlement, validate_response,
    },
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
                retained_case(mutation, method, failure, None);
            }
        }
    }
}
fn retained_case(
    mutation_method: Method,
    method: Method,
    failure: Option<IcObservationProviderError>,
    settlement_outcome: Option<ObservationOutcomeRecord>,
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
    if mutation_method == Method::TakeCanisterSnapshot && settlement_outcome.is_some() {
        // Retain the synthetic original list before reserving any mutation. This
        // proves local retention order only, never authenticated IC chronology.
        let baseline =
            candid::encode_one(Vec::<ic_management_canister_types::Snapshot>::new()).unwrap();
        fs::write(root.join("baseline.candid"), baseline).unwrap();
    }
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
    if let Some(outcome) = settlement_outcome {
        if mutation_method == Method::TakeCanisterSnapshot {
            record_qualified_capture_claim(&root, &request, response.as_ref().unwrap(), outcome);
        } else {
            record_qualified_lifecycle_claim(&root, &request, response.as_ref().unwrap(), outcome);
        }
        assert_eq!(provider.calls.len(), 1);
    }
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

#[test]
fn qualified_lifecycle_claims_require_explicit_receipts_and_keep_original_spending() {
    for method in [
        Method::StopCanister,
        Method::StartCanister,
        Method::LoadCanisterSnapshot,
    ] {
        for outcome in [
            ObservationOutcomeRecord::Applied,
            ObservationOutcomeRecord::NotApplied,
            ObservationOutcomeRecord::Uncertain,
        ] {
            retained_case(method, Method::CanisterStatus, None, Some(outcome));
        }
    }
}

fn record_qualified_lifecycle_claim(
    root: &Path,
    request: &IcObservationRequest<'_>,
    response: &IcObservationResponse,
    outcome: ObservationOutcomeRecord,
) {
    let claim = lifecycle_claim(request, response, outcome);
    let layout = BackupLayoutGuard::acquire(root).unwrap();
    let journal = AttemptJournalGuard::open(&layout, request.authority()).unwrap();
    let before = fs::read(journal.path()).unwrap();
    let view = validate_lifecycle_settlement(
        request,
        journal.record().unwrap(),
        response,
        &claim.challenge,
        &claim,
    )
    .unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), before);
    assert_eq!(view.outcome(), outcome);
    drop(journal);
    drop(layout);
    let recorded = record_receipt_and_reopen(root, request, outcome, &view.settlement().evidence);
    assert!(
        validate_lifecycle_settlement(request, &recorded, response, &claim.challenge, &claim)
            .is_err()
    );
}

fn record_receipt_and_reopen(
    root: &Path,
    request: &IcObservationRequest<'_>,
    outcome: ObservationOutcomeRecord,
    evidence: &ArtifactChecksumRecord,
) -> AttemptJournalRecord {
    let layout = BackupLayoutGuard::acquire(root).unwrap();
    let references = layout.restore_references().unwrap();
    let mut journal = AttemptJournalGuard::open(&layout, request.authority()).unwrap();
    let before = fs::read(journal.path()).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    // Pure admission cannot write this receipt. The integration explicitly invokes
    // the sole journal transition after independently qualifying its evidence.
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: request.observation_attempt(),
            request: request.payload().digest().hash().into(),
            outcome,
            evidence: evidence.hash().into(),
        })
        .unwrap();
    let recorded = fs::read(journal.path()).unwrap();
    assert_ne!(recorded, before);
    let progress = journal.record().unwrap().view();
    assert_eq!(
        (progress.mutations_used, progress.observations_used),
        (1, 1)
    );
    assert_eq!(
        (
            progress.mutations_remaining,
            progress.observations_remaining
        ),
        (0, 0)
    );
    assert_eq!(progress.pending_observation, None);
    assert_eq!(
        progress.pending_mutation,
        (outcome == ObservationOutcomeRecord::Uncertain).then_some(request.mutation_attempt())
    );
    assert_eq!(
        progress.applied,
        outcome == ObservationOutcomeRecord::Applied
    );
    assert!(journal.reserve_mutation().is_err());
    assert!(
        journal
            .reserve_observation(
                request.mutation_attempt(),
                request.payload().digest().hash()
            )
            .is_err()
    );
    assert_eq!(layout.restore_references().unwrap(), references);
    drop(journal);
    drop(layout);
    let layout = BackupLayoutGuard::acquire(root).unwrap();
    let journal = AttemptJournalGuard::open(&layout, request.authority()).unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), recorded);
    assert_eq!(journal.record().unwrap().view(), progress);
    assert_eq!(layout.restore_references().unwrap(), references);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    assert_eq!(
        fs::read(root.join("mutation.arguments")).unwrap(),
        request.mutation().arguments()
    );
    assert_eq!(
        fs::read(root.join("observation.arguments")).unwrap(),
        request.payload().arguments()
    );
    assert_eq!(
        fs::read(root.join("obligation.evidence")).unwrap(),
        b"native retained obligation"
    );
    journal.record().unwrap().clone()
}

fn lifecycle_claim(
    request: &IcObservationRequest<'_>,
    response: &IcObservationResponse,
    outcome: ObservationOutcomeRecord,
) -> IcLifecycleSettlement {
    let hash = |bytes: &[u8]| ArtifactChecksumRecord::from_bytes(bytes);
    // Synthetic passive proof qualifies local record/accounting only, never IC effects.
    let attribution = match outcome {
        ObservationOutcomeRecord::Applied => IcLifecycleAttribution::Applied {
            attribution: hash(b"native original attribution"),
        },
        ObservationOutcomeRecord::NotApplied => IcLifecycleAttribution::NotApplied {
            exclusion: hash(b"native original exclusion"),
        },
        ObservationOutcomeRecord::Uncertain => IcLifecycleAttribution::Unresolved {
            uncertainty: hash(b"native settled uncertainty"),
        },
    };
    let challenge = hash(b"native current qualification");
    IcLifecycleSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge,
        status: IcLifecycleReply::decode(request.payload(), &response.input().reply)
            .unwrap()
            .digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution,
        evidence: hash(b"native complete qualification"),
    }
}

#[test]
fn qualified_capture_claims_keep_baseline_spending_and_references_through_reopen() {
    for outcome in [
        ObservationOutcomeRecord::Applied,
        ObservationOutcomeRecord::NotApplied,
        ObservationOutcomeRecord::Uncertain,
    ] {
        retained_case(
            Method::TakeCanisterSnapshot,
            Method::ListCanisterSnapshots,
            None,
            Some(outcome),
        );
    }
}

fn record_qualified_capture_claim(
    root: &Path,
    request: &IcObservationRequest<'_>,
    response: &IcObservationResponse,
    outcome: ObservationOutcomeRecord,
) {
    let bytes = fs::read(root.join("baseline.candid")).unwrap();
    let baseline = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    let proof = ArtifactChecksumRecord::from_bytes(b"synthetic native capture qualification");
    let attribution = match outcome {
        ObservationOutcomeRecord::Applied => IcCaptureAttribution::Applied {
            snapshot_id: vec![255],
            attribution: proof.clone(),
        },
        ObservationOutcomeRecord::NotApplied => IcCaptureAttribution::NotApplied {
            exclusion: proof.clone(),
        },
        ObservationOutcomeRecord::Uncertain => IcCaptureAttribution::Unresolved {
            uncertainty: proof.clone(),
        },
    };
    let claim = IcCaptureSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: ArtifactChecksumRecord::from_bytes(b"native capture challenge"),
        baseline: baseline.digest(),
        inventory: IcSnapshotReply::decode(request.payload(), &response.input().reply)
            .unwrap()
            .digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution,
        evidence: proof,
    };
    let layout = BackupLayoutGuard::acquire(root).unwrap();
    let journal = AttemptJournalGuard::open(&layout, request.authority()).unwrap();
    let before = fs::read(journal.path()).unwrap();
    let view = validate_capture_settlement(
        request,
        journal.record().unwrap(),
        &baseline,
        response,
        &claim.challenge,
        &claim,
    )
    .unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), before);
    assert_eq!(view.outcome(), outcome);
    assert_eq!(
        view.captured_snapshot().map(IcSnapshotInfo::id),
        (outcome == ObservationOutcomeRecord::Applied).then_some([255_u8].as_slice())
    );
    drop(journal);
    drop(layout);
    let recorded = record_receipt_and_reopen(root, request, outcome, &view.settlement().evidence);
    assert!(
        validate_capture_settlement(
            request,
            &recorded,
            &baseline,
            response,
            &claim.challenge,
            &claim
        )
        .is_err()
    );
    assert_eq!(fs::read(root.join("baseline.candid")).unwrap(), bytes);
}
