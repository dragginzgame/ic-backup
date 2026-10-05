//! Native original reservation/reply recovery qualification; no live IC management effects.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_mutation::{
            IcMutationAcknowledgement, IcMutationAcknowledgementInput, IcMutationRequest,
        },
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        operation_plan::OperationPlanRecord,
        restore_references::RestoreReferencesRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_operation_plan},
    policy::ic_mutation::{IcMutationAssociationError, validate_acknowledgement},
    ports::ic_mutation::{IcMutationProvider, IcMutationProviderError},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("ic-mutation-{}-{nonce}", std::process::id()))
}
struct Originals {
    payload: IcManagementRequestRecord,
    plan: OperationPlanRecord,
}
struct RetainedEvidence {
    journal: Vec<u8>,
    plan: Vec<u8>,
    references: RestoreReferencesRecord,
}
impl Originals {
    fn new(method: Method) -> Self {
        let payload = IcManagementRequestRecord::new(IcManagementRequest {
            method,
            target: TARGET.into(),
            snapshot_id: (method == Method::LoadCanisterSnapshot).then(|| vec![0, 255, 128]),
        })
        .unwrap();
        let plan = serde_json::from_value(json!({
            "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
            "inventory":{"version":1,"targets":[{"canister_id":TARGET,"parent_canister_id":null,"role":null,"module_hash":null}]},
            "selected_targets":[TARGET],"graph":{"version":1,"nodes":[{"operation_sequence":42,"depends_on":[]}]},
            "operations":[{"operation_sequence":42,"target":TARGET,"request":payload.digest().hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
        })).unwrap();
        Self { payload, plan }
    }
}
/// One passive fixture invocation logs exact bytes and returns association/failure.
/// It implements no canister behavior and qualifies neither dispatch nor IC effects.
struct NativeProvider {
    failure: Option<IcMutationProviderError>,
    calls: Vec<(String, String, String, Vec<u8>, u32)>,
}
impl IcMutationProvider for NativeProvider {
    fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        let payload = request.payload();
        self.calls.push((
            payload.receiver().into(),
            payload.target().into(),
            payload.method().name().into(),
            payload.arguments().into(),
            request.mutation_attempt(),
        ));
        if let Some(error) = self.failure {
            return Err(error);
        }
        let reply = if payload.method() == Method::TakeCanisterSnapshot {
            candid::encode_one(ic_management_canister_types::Snapshot {
                id: vec![0, 255, 128],
                taken_at_timestamp: 42,
                total_size: 128,
            })
            .unwrap()
        } else {
            b"DIDL\0\0".to_vec()
        };
        IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: request.plan().context().clone(),
            target: payload.target().into(),
            reply,
            evidence: hash("12"),
        })
        .map_err(|_| IcMutationProviderError::Indeterminate)
    }
}

#[test]
fn all_original_updates_and_provider_failures_preserve_exact_spent_evidence_through_reopen() {
    for method in [
        Method::TakeCanisterSnapshot,
        Method::LoadCanisterSnapshot,
        Method::StopCanister,
        Method::StartCanister,
    ] {
        for failure in [
            None,
            Some(IcMutationProviderError::Unavailable),
            Some(IcMutationProviderError::Unsupported),
            Some(IcMutationProviderError::Indeterminate),
        ] {
            retained_case(method, failure);
        }
    }
}
fn retained_case(method: Method, failure: Option<IcMutationProviderError>) {
    let originals = Originals::new(method);
    let root = root();
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_operation_plan(&layout, &originals.plan).unwrap();
    fs::write(
        root.join("original.arguments"),
        originals.payload.arguments(),
    )
    .unwrap();
    fs::write(
        root.join("retained.obligation"),
        b"native obligation evidence",
    )
    .unwrap();
    layout
        .retain_restore(
            &root.join("restore-reference.json"),
            originals.plan.digest().hash(),
        )
        .unwrap();
    let references = layout.restore_references().unwrap();
    let mut journal =
        AttemptJournalGuard::create(&layout, originals.plan.attempt_authority(42).unwrap())
            .unwrap();
    let attempt = journal.reserve_mutation().unwrap();
    let bytes = fs::read(journal.path()).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let evidence = RetainedEvidence {
        journal: bytes,
        plan: plan_bytes,
        references,
    };
    let request = IcMutationRequest::new(
        &originals.plan,
        42,
        journal.record().unwrap(),
        &originals.payload,
    )
    .unwrap();
    let mut provider = NativeProvider {
        failure,
        calls: vec![],
    };
    let acknowledgement = match provider.submit_mutation(&request) {
        Ok(acknowledgement) => {
            validate_acknowledgement(&request, journal.record().unwrap(), &acknowledgement)
                .unwrap();
            fs::write(root.join("reply.candid"), &acknowledgement.input().reply).unwrap();
            Some(acknowledgement)
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
            originals.payload.arguments().into(),
            attempt
        )]
    );
    assert_eq!(fs::read(journal.path()).unwrap(), evidence.journal);
    assert_eq!(layout.restore_references().unwrap(), evidence.references);
    assert!(journal.reserve_mutation().is_err());
    drop(journal);
    drop(layout);
    assert_reopened(
        &root,
        &originals,
        &evidence,
        acknowledgement.as_ref(),
        &provider,
    );
    fs::remove_dir_all(root).unwrap();
}
fn assert_reopened(
    root: &Path,
    originals: &Originals,
    evidence: &RetainedEvidence,
    acknowledgement: Option<&IcMutationAcknowledgement>,
    provider: &NativeProvider,
) {
    let layout = BackupLayoutGuard::acquire(root).unwrap();
    let mut journal =
        AttemptJournalGuard::open(&layout, &originals.plan.attempt_authority(42).unwrap()).unwrap();
    let request = IcMutationRequest::new(
        &originals.plan,
        42,
        journal.record().unwrap(),
        &originals.payload,
    )
    .unwrap();
    if let Some(acknowledgement) = acknowledgement {
        // A retained late reply is associated locally; it never invokes the provider again.
        assert_eq!(
            fs::read(root.join("reply.candid")).unwrap(),
            acknowledgement.input().reply
        );
        validate_acknowledgement(&request, journal.record().unwrap(), acknowledgement).unwrap();
    }
    assert_eq!(fs::read(journal.path()).unwrap(), evidence.journal);
    assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
    assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
    assert!(!journal.record().unwrap().view().applied);
    journal.reserve_observation(1, hash("34").hash()).unwrap();
    if let Some(acknowledgement) = acknowledgement {
        assert!(matches!(
            validate_acknowledgement(&request, journal.record().unwrap(), acknowledgement),
            Err(IcMutationAssociationError::Reservation(_))
        ));
    }
    assert_eq!(
        journal.record().unwrap().view().pending_observation,
        Some(2)
    );
    assert_eq!(provider.calls.len(), 1);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        evidence.plan
    );
    assert_eq!(
        fs::read(root.join("original.arguments")).unwrap(),
        originals.payload.arguments()
    );
    assert_eq!(
        fs::read(root.join("retained.obligation")).unwrap(),
        b"native obligation evidence"
    );
    assert_eq!(layout.restore_references().unwrap(), evidence.references);
    drop(journal);
    drop(layout);
}
