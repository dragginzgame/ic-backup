//! Re-authored legacy controller/status cases against the current plan boundary.

use super::{super::CapturePolicy, *};
use ic_platform_protocol::management_snapshots::{CaptureStatus, Settings};
use ic_platform_types::Principal;

impl CapturePolicy {
    fn status_requirements(&self) -> StatusRequirements {
        StatusRequirements {
            module_sha256: self.request().module_sha256,
            sender: self.request().sender,
            sole_controller: false,
        }
    }

    fn validate_status(&self, observed: &CaptureStatus) -> Result<(), CaptureStatusError> {
        self.status_requirements().validate_status(observed)
    }

    pub(in crate::store) fn validate_status_reply(
        &self,
        reply: &[u8],
    ) -> Result<(), CaptureStatusReplyError> {
        self.status_requirements().validate_status_reply(reply)
    }
}

impl StatusRequirements {
    pub(in crate::store) fn validate_status(
        &self,
        observed: &CaptureStatus,
    ) -> Result<(), CaptureStatusError> {
        self.validate_status_fields(
            observed.status.clone(),
            observed.module_hash.as_deref(),
            &observed.settings.controllers,
        )
    }
}

pub(in crate::store) fn plan() -> CapturePolicy {
    CapturePolicy::decode(include_bytes!("../../tests/fixtures/capture-plan.json")).unwrap()
}

fn sender(plan: &CapturePolicy) -> Principal {
    plan.request().sender
}

pub(in crate::store) fn status(plan: &CapturePolicy) -> CaptureStatus {
    CaptureStatus {
        status: Status::Stopped,
        settings: Settings {
            controllers: vec![sender(plan)],
        },
        module_hash: Some(plan.request().module_sha256.as_bytes().to_vec()),
    }
}

#[test]
fn stopped_matching_status_preserves_input_order_and_original_plan() {
    let p = plan();
    let mut observed = status(&p);
    observed.settings.controllers = vec![
        p.request().initiator,
        sender(&p),
        p.request().target.canister,
    ];
    let original = observed.clone();
    assert_eq!(p.validate_status(&observed), Ok(()));
    assert_eq!(observed, original);
    observed.settings.controllers.reverse();
    assert_eq!(p.validate_status(&observed), Ok(()));
}

#[test]
fn running_and_stopping_are_refused_without_implicit_stop() {
    let p = plan();

    for state in [Status::Running, Status::Stopping] {
        let mut observed = status(&p);
        observed.status = state.clone();
        assert_eq!(
            p.validate_status(&observed),
            Err(CaptureStatusError::NotStopped(state.clone()))
        );
        assert_eq!(observed.status, state);
    }
}

#[test]
fn module_hash_must_be_present_exact_length_and_exact_original_value() {
    let p = plan();
    for hash in [
        None,
        Some(vec![]),
        Some(vec![0x44; 31]),
        Some(vec![0x44; 33]),
        Some(vec![0x45; 32]),
    ] {
        let mut observed = status(&p);
        observed.module_hash = hash;
        assert_eq!(
            p.validate_status(&observed),
            Err(CaptureStatusError::ModuleMismatch)
        );
    }
    assert_eq!(p.validate_status(&status(&p)), Ok(()));
}

#[test]
fn empty_initiator_target_and_visible_status_do_not_grant_sender_authority() {
    let p = plan();

    for controllers in [
        vec![],
        vec![p.request().initiator],
        vec![p.request().target.canister],
    ] {
        let mut observed = status(&p);
        observed.settings.controllers = controllers;
        assert_eq!(
            p.validate_status(&observed),
            Err(CaptureStatusError::NotController)
        );
    }
}

#[test]
fn canic_requires_executor_membership_not_host_sender_or_initiator() {
    let original = plan();
    let host = sender(&original);
    let mut request = original.request().clone();
    let executor = Principal::from_slice(&[17, 1]);
    request.sender = executor;
    let p = CapturePolicy::new(request).unwrap();
    let mut observed = status(&p);
    assert_eq!(p.validate_status(&observed), Ok(()));
    observed.settings.controllers = vec![host, p.request().initiator];
    assert_eq!(
        p.validate_status(&observed),
        Err(CaptureStatusError::NotController)
    );
    observed.settings.controllers.push(executor);
    assert_eq!(p.validate_status(&observed), Ok(()));
}

#[test]
fn canonical_duplicates_and_excessive_sets_refuse_even_with_the_sender_present() {
    let p = plan();
    let mut observed = status(&p);
    let alias = Principal::from_text(sender(&p).to_text().to_uppercase()).unwrap();
    observed.settings.controllers.push(alias);
    assert_eq!(
        p.validate_status(&observed),
        Err(CaptureStatusError::DuplicateController)
    );
    observed.settings.controllers = (0_u32..9)
        .map(|n| Principal::from_slice(&n.to_be_bytes()))
        .collect();
    observed.settings.controllers.push(sender(&p));
    assert_eq!(p.validate_status(&observed), Ok(()));
    observed.settings.controllers.push(p.request().initiator);
    assert_eq!(
        p.validate_status(&observed),
        Err(CaptureStatusError::ControllerLimit)
    );
}
