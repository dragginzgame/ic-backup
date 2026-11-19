//! Exact protocol and HTTP regressions. Call lifecycle tests live with the recorder.

pub(super) mod call;
pub(in crate::store) mod certificate_tests;
mod http_lookup_tests;
mod http_tests;
mod lookup_tests;
mod observation_tests;
mod request_tests;

use super::*;
pub(in crate::store) use crate::store::capture::tests as fixtures;
use crate::store::{
    BackupDigest,
    capture::{
        CapturePolicy,
        status::tests::{plan, status},
    },
};
pub(in crate::store) use call::CaptureCall;
use ic_platform::ingress::Content as EnvelopeContent;
use ic_platform_protocol::management_snapshots::CanisterArgs;

impl PreparedStatus {
    // Inspect the original bytes; never normalize or re-encode a signed envelope.
    // Authentication metadata, routing, observation and freshness stay provider-owned.
    pub(in crate::store) fn validate_call(
        &self,
        plan: &CapturePolicy,
    ) -> Result<(), OperationStoreError> {
        self.validate_call_for(plan, CaptureCall::Status)
    }

    pub(in crate::store) fn validate_call_for(
        &self,
        plan: &CapturePolicy,
        call: CaptureCall,
    ) -> Result<(), OperationStoreError> {
        self.validate_expected_call(plan, call.method_name(), &call.arguments(plan)?)
    }

    pub(in crate::store) fn validate_expected_call(
        &self,
        plan: &CapturePolicy,
        method: &str,
        arguments: &[u8],
    ) -> Result<(), OperationStoreError> {
        let expected = plan.request().sender;
        self.validate_direct_call(
            expected,
            ic_platform_types::Principal::management_canister(),
            method,
            arguments,
        )
    }

    pub(in crate::store) fn decode(bytes: &[u8]) -> Result<Self, OperationStoreError> {
        if bytes.len() < 46 {
            return Err(OperationStoreError::Changed);
        }
        let (envelope, observation) = split(&bytes[40..])?;
        let expires_ns = u64::from_le_bytes(
            bytes[32..40]
                .try_into()
                .map_err(|_| OperationStoreError::Changed)?,
        );
        if expires_ns == 0 {
            return Err(OperationStoreError::Changed);
        }
        Ok(Self {
            request_id: bytes[..32]
                .try_into()
                .map_err(|_| OperationStoreError::Changed)?,
            expires_ns,
            envelope,
            observation,
        })
    }
}

pub(in crate::store) fn arguments(plan: &CapturePolicy) -> Result<Vec<u8>, OperationStoreError> {
    (CanisterArgs {
        canister_id: plan.request().target.canister,
    })
    .encode(256)
    .map_err(|_| OperationStoreError::StatusAdmission)
}

impl StatusAdmissionRecord {
    pub(in crate::store) fn validate(
        &self,
        plan: &CapturePolicy,
    ) -> Result<(), OperationStoreError> {
        let initial_bytes = 0;
        self.validate_limits(
            plan.request().limits.network_requests,
            plan.request().limits.evidence_bytes,
            initial_bytes,
        )
    }
}

pub(in crate::store) fn admission() -> StatusAdmissionRecord {
    StatusAdmissionRecord {
        provider: [1; 32],
        application: [2; 32],
        prepare_deadline_ns: 10,
        submit_deadline_ns: 20,
        prepare_requests: 1,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 4096,
    }
}

pub(in crate::store) fn prepared() -> PreparedStatus {
    prepared_content(call_content())
}

pub(in crate::store) fn call_content() -> EnvelopeContent {
    let p = plan();
    let sender = p.request().sender;
    EnvelopeContent::Call {
        nonce: Some(vec![0, 255, 7]),
        ingress_expiry: 21,
        sender: sender.as_slice().try_into().unwrap(),
        canister_id: [].as_slice().try_into().unwrap(),
        method_name: "canister_status".into(),
        arg: arguments(&p).unwrap(),
        sender_info: None,
    }
}

pub(in crate::store) fn prepared_content(content: EnvelopeContent) -> PreparedStatus {
    let request_id = content.request_id().unwrap();
    let expires_ns = match &content {
        EnvelopeContent::Call { ingress_expiry, .. }
        | EnvelopeContent::ReadState { ingress_expiry, .. } => *ingress_expiry,
    };
    // Correct wire shape, deliberately unauthenticated. No IC effect is simulated.
    let envelope = ic_platform::ingress::encode_envelope(
        &ic_platform::ingress::Envelope {
            content,
            sender_pubkey: Some(vec![1]),
            sender_sig: Some(vec![2]),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap();
    PreparedStatus {
        request_id,
        expires_ns,
        envelope,
        observation: b"original observation binding".to_vec(),
    }
}

pub(in crate::store) fn evidence() -> StatusEvidence {
    StatusEvidence {
        rejected: false,
        response: crate::store::capture::tests::wire::encode(&status(&plan()), 4096),
        proof: b"native ordering fixture; not a certificate".to_vec(),
    }
}
