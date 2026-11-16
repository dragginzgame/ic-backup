use crate::store::{OperationLimits, RestoreRequest};
pub(in crate::store) fn declaration(
    source: &crate::store::CaptureSource<'_, '_>,
) -> RestoreRequest {
    RestoreRequest {
        operation_id: "cd".repeat(16),
        target: source.summary().target,
        trust_root: source.original_plan().request().trust_root,
        initiator: ic_platform::Principal::from_text("2vxsx-fae").unwrap(),
        release: source.summary().release,
        sender: ic_platform::Principal::from_text("aaaaa-aa").unwrap(),
        limits: OperationLimits {
            payload_bytes: 10,
            extent_bytes: 3,
            network_requests: 15,
            observations: 0,
            signing_requests: 15,
            evidence_bytes: 1024 * 1024,
        },
    }
}
