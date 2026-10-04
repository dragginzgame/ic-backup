//! Passive native snapshot-list fixtures; no management backend.

use crate::model::{
    ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
    operation_plan::{PlanContextRecord, PlanContextRequest},
    snapshot_read::{SnapshotReadObservationInput, SnapshotReadRequest, SnapshotVisibility},
};
use crate::test_support::membership::{APP, hash};

pub fn list() -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::ListCanisterSnapshots,
        target: APP.into(),
        snapshot_id: None,
    })
    .unwrap()
}
pub fn input(request: &SnapshotReadRequest<'_>) -> SnapshotReadObservationInput {
    SnapshotReadObservationInput {
        request: request.digest(),
        context: PlanContextRecord::new(&PlanContextRequest {
            network: request.binding().network().into(),
            caller: request.binding().caller().into(),
            release: request.binding().release().into(),
        })
        .unwrap(),
        target: request.binding().target().into(),
        visibility: SnapshotVisibility::Public,
        controllers: None,
        evidence: hash("34"),
        remote_observations: 1,
    }
}
