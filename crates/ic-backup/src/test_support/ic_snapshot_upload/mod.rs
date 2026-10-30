//! Declared originals and independent wire vectors; no simulated IC effects.

use crate::model::{
    attempt_journal::AttemptBudgetRecord,
    effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    ic_snapshot_upload::IcSnapshotUploadRequest,
    operation_plan::{
        OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlannedOperationRecord,
        PlannedOperationRequest,
    },
};
pub use crate::test_support::hex_bytes as unhex;
use ic_platform::candid::contract;

pub const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
pub const SOURCE_ID: &[u8] = &[0, 255, 17];
pub const DESTINATION_ID: &[u8] = &[21, 0, 255];

pub fn raw(name: &str) -> Vec<u8> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    unhex(
        cases.iter().find(|case| case["name"] == name).unwrap()["reply_hex"]
            .as_str()
            .unwrap(),
    )
}

pub fn plan(request: &str) -> OperationPlanRecord {
    OperationPlanRecord::new(plan_request(request, 7, 1)).unwrap()
}

pub fn plan_request(request: &str, sequence: u64, observations: u32) -> OperationPlanRequest {
    use crate::model::inventory::{InventoryRecord, InventoryTargetRecord, InventoryTargetRequest};
    let mut plan = crate::test_support::operation_plan::request();
    plan.inventory = InventoryRecord::new(vec![
        InventoryTargetRecord::new(&InventoryTargetRequest {
            canister_id: TARGET.into(),
            parent_canister_id: None,
            role: None,
            module_hash: None,
        })
        .unwrap(),
    ])
    .unwrap();
    plan.graph = EffectGraphRecord::new(vec![
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: sequence,
            depends_on: vec![],
        })
        .unwrap(),
    ])
    .unwrap();
    plan.operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: sequence,
            target: TARGET.into(),
            request: request.into(),
            budget: AttemptBudgetRecord::new(1, observations).unwrap(),
        })
        .unwrap(),
    ];
    plan.budget = PlanBudgetRecord::new(1, observations).unwrap();
    plan
}

pub fn source_plan() -> OperationPlanRecord {
    plan(&"ef".repeat(32))
}
pub fn upload_plan(payload: &IcSnapshotUploadRequest<'_>) -> OperationPlanRecord {
    plan(payload.binding_digest().hash())
}

/// Borrow independent original declarations without returning self-referential fixtures.
pub fn with_observation(
    check: impl FnOnce(
        &OperationPlanRecord,
        &IcSnapshotUploadRequest<'_>,
        &crate::model::ic_request::IcManagementRequestRecord,
        &crate::model::attempt_journal::AttemptJournalRecord,
    ),
) {
    use crate::model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
    };
    let source = source_plan();
    let original = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let bytes = raw("upload-source");
    let metadata = IcSnapshotMetadataReply::decode(&original, &bytes).unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"retained declared source");
    let upload = IcSnapshotUploadRequest::metadata(&source, &metadata, &checksum).unwrap();
    let plan = upload_plan(&upload);
    let list = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::ListCanisterSnapshots,
        target: TARGET.into(),
        snapshot_id: None,
    })
    .unwrap();
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    journal.reserve_mutation().unwrap();
    journal
        .reserve_observation(1, list.digest().hash())
        .unwrap();
    check(&plan, &upload, &list, &journal);
}

pub fn observation_input(
    request: &crate::model::ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest<
        '_,
        '_,
    >,
) -> crate::model::ic_observation::IcObservationResponseInput {
    crate::model::ic_observation::IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: request.payload().digest(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply: contract::encode_one(Vec::<ic_platform::management_canister_types::Snapshot>::new())
            .unwrap(),
        evidence: crate::model::artifacts::ArtifactChecksumRecord::from_bytes(
            b"passive list evidence",
        ),
    }
}

pub fn data_workflow(
    payload: &IcSnapshotUploadRequest<'_>,
    mutations: u32,
) -> crate::model::execution_workflow::ExecutionWorkflowRecord {
    let original = upload_plan(payload);
    let operation = |sequence, mutations, observations| {
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: sequence,
            target: TARGET.into(),
            request: payload.binding_digest().hash().into(),
            budget: AttemptBudgetRecord::new(mutations, observations).unwrap(),
        })
        .unwrap()
    };
    let graph = EffectGraphRecord::new(
        [(0, vec![]), (7, vec![0])]
            .into_iter()
            .map(|(sequence, depends_on)| {
                EffectNodeRecord::new(EffectNodeRequest {
                    operation_sequence: sequence,
                    depends_on,
                })
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    crate::model::execution_workflow::ExecutionWorkflowRecord::new(
        OperationPlanRecord::new(OperationPlanRequest {
            context: original.context().clone(),
            inventory: original.inventory().clone(),
            selected_targets: original.selected_targets().to_vec(),
            graph,
            operations: vec![operation(0, 1, 0), operation(7, mutations, 1)],
            budget: PlanBudgetRecord::new(mutations + 10, 10).unwrap(),
        })
        .unwrap(),
    )
}
