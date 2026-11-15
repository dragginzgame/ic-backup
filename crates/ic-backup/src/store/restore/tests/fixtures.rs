use crate::store::{
    PreparedStatus, RestoreLimitsRecord, RestorePlan, RestoreRequest, StatusAdmissionRecord,
    capture::test_fixtures::signatures::{Scheme, change, fixture},
};
use ic_platform::{
    candid_codec, ic_protocol::EnvelopeContent, management_canister_types::CanisterIdRecord,
};

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
        limits: RestoreLimitsRecord {
            payload_bytes: 10,
            extent_bytes: 3,
            network_requests: 15,
            observations: 0,
            signing_requests: 15,
            evidence_bytes: 1024 * 1024,
        },
    }
}

pub(in crate::store) fn admission() -> StatusAdmissionRecord {
    StatusAdmissionRecord {
        provider: [1; 32],
        application: [2; 32],
        prepare_deadline_ns: 1,
        submit_deadline_ns: 2,
        prepare_requests: 0,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 4096,
    }
}

pub(in crate::store) fn prepared(plan: &RestorePlan, scheme: Scheme) -> PreparedStatus {
    let (_, mut r) = fixture(scheme);
    change(&mut r, |e| {
        if let EnvelopeContent::Call { arg, .. } = e.content.to_mut() {
            *arg = candid_codec::encode_one(
                &CanisterIdRecord {
                    canister_id: plan.request().target.canister,
                },
                512,
            )
            .unwrap();
        }
        e.sender_sig = Some(scheme.sign(&e.content.to_request_id().signable()));
    });
    r
}
