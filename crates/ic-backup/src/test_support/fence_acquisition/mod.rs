//! Original application payload and pending acquisition fixtures; no authenticated backend.

use crate::{
    model::{
        attempt_journal::AttemptJournalRecord, consistency::*, fence_acquisition::*,
        fence_obligation::*, operation_plan::OperationPlanRecord,
        restore_safety::RestoreSafetyLaneRecord,
    },
    test_support::{
        membership::{self, APP, hash},
        restore_safety,
    },
};
use serde_json::json;

pub struct Original {
    pub payload: FenceAcquisitionPayload,
    pub plan: OperationPlanRecord,
    pub obligation: FenceObligationRecord,
    pub journal: AttemptJournalRecord,
}
pub fn original(restore: bool) -> Original {
    let payload =
        FenceAcquisitionPayload::new(APP, "acquire_fence", b"native opaque acquisition").unwrap();
    let mut plan = serde_json::to_value(membership::plan()).unwrap();
    plan["operations"][0]["request"] = json!(payload.digest().hash());
    let plan: OperationPlanRecord = serde_json::from_value(plan).unwrap();
    let obligation = if restore {
        let source = restore_safety::source();
        let requirement =
            restore_safety::requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
        FenceObligationRecord::for_restore(&plan, &source, &requirement, 0).unwrap()
    } else {
        FenceObligationRecord::for_capture(
            &plan,
            &ConsistencyRequirementRecord::new(
                &plan,
                ConsistencyGuaranteeRecord::ApplicationCoordinated,
            ),
            0,
            &ApplicationFenceBinding {
                identity: hash("56"),
                membership_revision: hash("78"),
            },
        )
        .unwrap()
    };
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(0).unwrap());
    journal.reserve_mutation().unwrap();
    Original {
        payload,
        plan,
        obligation,
        journal,
    }
}
