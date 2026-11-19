//! Retained signed native fixtures, not IC-effect or live-adapter qualification.

use super::*;
use crate::store::capture::{
    CaptureOperation, CapturePlan,
    intent::tests::signed_reply,
    procedure::{Next, Procedure},
    tests::reply,
};

pub(in crate::store) fn settle(operation: &mut CaptureOperation, mixed: bool) {
    let intent = CapturePlan::decode(operation.plan().encoded()).unwrap();
    let mut flow = Procedure::new(&intent).unwrap();
    let mut ordinal = 0;
    loop {
        match flow.next().unwrap() {
            Next::Call(step) => {
                let (method, arguments) = flow.request().unwrap();
                let (request, evidence) = signed_reply(
                    &intent,
                    ordinal,
                    method,
                    arguments,
                    reply(&intent, step, mixed),
                );
                ordinal += 1;
                let bytes = evidence.encode(intent.profile().result_bytes).unwrap();
                operation
                    .append_fixture(
                        FactRecord::CallReserved {
                            step,
                            reference_ns: 100,
                        },
                        None,
                    )
                    .unwrap();
                operation
                    .append_fixture(
                        FactRecord::CallPrepared {
                            step,
                            request: ReferenceRecord::new(&request.envelope).unwrap(),
                        },
                        Some(&request.envelope),
                    )
                    .unwrap();
                operation
                    .append_fixture(FactRecord::CallSubmitted { step }, None)
                    .unwrap();
                operation
                    .append_fixture(
                        FactRecord::CallResult {
                            step,
                            result: ReferenceRecord::new(&bytes).unwrap(),
                        },
                        Some(&bytes),
                    )
                    .unwrap();
                flow.accept_authenticated(step, 100, &request, &evidence)
                    .unwrap();
            }
            Next::Materialize => {
                operation
                    .append_fixture(FactRecord::MaterializationStarted, None)
                    .unwrap();
                operation
                    .materialize(&operation.check().unwrap(), &flow, true)
                    .unwrap();
                flow.after_materialization().unwrap();
            }
            Next::Publish => break,
        }
    }
}
