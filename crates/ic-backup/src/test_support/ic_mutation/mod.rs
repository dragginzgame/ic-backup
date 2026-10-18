//! Exact original IC reservations and passive bounded wire fixtures; no IC backend.

use crate::model::ic_update_response::IcUpdateResponseInput;
use crate::{
    model::{
        attempt_journal::AttemptJournalRecord,
        ic_mutation::IcMutationRequest,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        operation_plan::OperationPlanRecord,
    },
    test_support::{
        control_authority,
        membership::{APP, hash},
    },
};
use ic_platform::candid::contract;

pub struct Original {
    pub payload: IcManagementRequestRecord,
    pub plan: OperationPlanRecord,
    pub journal: AttemptJournalRecord,
}
pub fn original(method: IcManagementMethodRecord) -> Original {
    let payload = IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: APP.into(),
        snapshot_id: (method == IcManagementMethodRecord::LoadCanisterSnapshot)
            .then(|| vec![0, 255, 128]),
    })
    .unwrap();
    let plan = control_authority::plan(&payload);
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    journal.reserve_mutation().unwrap();
    Original {
        payload,
        plan,
        journal,
    }
}
pub fn reply(method: IcManagementMethodRecord) -> Vec<u8> {
    if method == IcManagementMethodRecord::TakeCanisterSnapshot {
        contract::encode_one(ic_platform::management_canister_types::Snapshot {
            id: vec![0, 255, 128],
            taken_at_timestamp: u64::MAX,
            total_size: 42,
        })
        .unwrap()
    } else {
        b"DIDL\0\0".to_vec()
    }
}
pub fn input(request: &IcMutationRequest<'_>) -> IcUpdateResponseInput {
    IcUpdateResponseInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply: reply(request.payload().method()),
        evidence: hash("34"),
    }
}
