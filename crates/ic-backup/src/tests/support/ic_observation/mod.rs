//! Retained original observation and passive wire fixtures; no IC simulation.

use crate::{
    model::{
        attempt_journal::AttemptJournalRecord,
        ic_observation::{IcObservationRequest, IcObservationResponseInput},
        ic_request::{
            IcManagementMethodRecord as Method, IcManagementRequest, IcManagementRequestRecord,
        },
        operation_plan::OperationPlanRecord,
    },
    tests::support::{ic_mutation, operation_plan::hash},
};

pub struct Original {
    pub mutation: IcManagementRequestRecord,
    pub payload: IcManagementRequestRecord,
    pub plan: OperationPlanRecord,
    pub journal: AttemptJournalRecord,
}

impl Original {
    pub fn request(&self) -> IcObservationRequest<'_> {
        IcObservationRequest::new(&self.plan, 7, &self.journal, &self.mutation, &self.payload)
            .unwrap()
    }
}

pub fn original_for_mutation(mutation: Method, observation: Method) -> Original {
    let mut original = ic_mutation::original(mutation);
    let payload = IcManagementRequestRecord::new(IcManagementRequest {
        method: observation,
        target: original.payload.target().into(),
        snapshot_id: None,
    })
    .unwrap();
    original
        .journal
        .reserve_observation(1, payload.digest().hash())
        .unwrap();
    Original {
        mutation: original.payload,
        payload,
        plan: original.plan,
        journal: original.journal,
    }
}

pub fn snapshot(id: &[u8]) -> ic_platform::management_canister_types::Snapshot {
    ic_platform::management_canister_types::Snapshot {
        id: id.into(),
        taken_at_timestamp: 42,
        total_size: 123,
    }
}

pub fn original(method: Method) -> Original {
    let mutation = ic_mutation::original(Method::TakeCanisterSnapshot);
    let mut plan = serde_json::to_value(&mutation.plan).unwrap();
    plan["operations"][1]["budget"]["observations"] = serde_json::json!(2);
    plan["budget"]["observations"] = serde_json::json!(3);
    let plan: OperationPlanRecord = serde_json::from_value(plan).unwrap();
    let payload = IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: mutation.payload.target().into(),
        snapshot_id: None,
    })
    .unwrap();
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    let attempt = journal.reserve_mutation().unwrap();
    journal
        .reserve_observation(attempt, payload.digest().hash())
        .unwrap();
    Original {
        mutation: mutation.payload,
        payload,
        plan,
        journal,
    }
}
pub fn reply(method: Method) -> Vec<u8> {
    let file = if method == Method::CanisterStatus {
        include_str!("../../../model/ic_lifecycle_reply/tests/golden.json")
    } else {
        include_str!("../../../model/ic_snapshot_reply/tests/golden.json")
    };
    let cases: Vec<serde_json::Value> = serde_json::from_str(file).unwrap();
    let case = cases
        .iter()
        .find(|case| case["method"] == serde_json::to_value(method).unwrap())
        .unwrap();
    let hex = case["reply_hex"].as_str().unwrap();
    crate::tests::support::hex_bytes(hex)
}
pub fn input(request: &IcObservationRequest<'_>) -> IcObservationResponseInput {
    IcObservationResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        request: request.payload().digest(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply: reply(request.payload().method()),
        evidence: hash("34"),
    }
}
