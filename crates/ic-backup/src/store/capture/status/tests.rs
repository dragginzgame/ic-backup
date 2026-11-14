//! Re-authored legacy controller/status cases against the current plan boundary.
use super::*;
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::{
    Principal,
    management_canister_types::{
        DefiniteCanisterSettings, MemoryMetrics, QueryStats, StatusVisibility,
    },
};

pub(in crate::store::capture) fn plan() -> CapturePlan {
    CapturePlan::decode(include_bytes!("../../tests/fixtures/capture-plan.json").to_vec()).unwrap()
}
fn sender(plan: &CapturePlan) -> Principal {
    match plan.request().execution {
        CaptureExecution::Direct { sender } => sender,
        CaptureExecution::Canic { executor, .. } => executor,
    }
}
pub(in crate::store::capture) fn status(plan: &CapturePlan) -> CanisterStatusResult {
    CanisterStatusResult {
        status: CanisterStatusType::Stopped,
        ready_for_migration: false,
        version: 23,
        settings: DefiniteCanisterSettings {
            controllers: vec![sender(plan)],
            ..DefiniteCanisterSettings::default()
        },
        module_hash: Some(plan.request().module_sha256.as_bytes().to_vec()),
        memory_size: 0_u64.into(),
        memory_metrics: MemoryMetrics {
            wasm_memory_size: 0_u64.into(),
            stable_memory_size: 0_u64.into(),
            global_memory_size: 0_u64.into(),
            wasm_binary_size: 0_u64.into(),
            custom_sections_size: 0_u64.into(),
            canister_history_size: 0_u64.into(),
            wasm_chunk_store_size: 0_u64.into(),
            snapshots_size: 0_u64.into(),
            log_memory_store_size: 0_u64.into(),
        },
        cycles: 0_u64.into(),
        reserved_cycles: 0_u64.into(),
        idle_cycles_burned_per_day: 0_u64.into(),
        query_stats: QueryStats {
            num_calls_total: 0_u64.into(),
            num_instructions_total: 0_u64.into(),
            request_payload_bytes_total: 0_u64.into(),
            response_payload_bytes_total: 0_u64.into(),
        },
    }
}

#[test]
fn stopped_matching_status_preserves_input_order_and_original_plan() {
    let p = plan();
    let bytes = p.encoded().to_vec();
    let commitment = p.digest();
    let mut observed = status(&p);
    observed.settings.controllers = vec![
        p.request().initiator,
        sender(&p),
        p.request().target.canister,
    ];
    let original = observed.clone();
    assert_eq!(p.validate_status(&observed), Ok(()));
    assert_eq!(observed, original);
    assert_eq!(p.encoded(), bytes);
    assert_eq!(p.digest(), commitment);
    observed.settings.controllers.reverse();
    assert_eq!(p.validate_status(&observed), Ok(()));
}

#[test]
fn running_and_stopping_are_refused_without_implicit_stop() {
    let p = plan();
    for state in [CanisterStatusType::Running, CanisterStatusType::Stopping] {
        let mut observed = status(&p);
        observed.status = state;
        assert_eq!(
            p.validate_status(&observed),
            Err(CaptureStatusError::NotStopped(state))
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
        observed.settings.status_visibility = StatusVisibility::AllowedViewers(vec![sender(&p)]);
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
    request.execution = CaptureExecution::Canic {
        executor,
        protocol: Sha256Digest::compute(b"protocol"),
        generation: 7,
    };
    let p = CapturePlan::new(request).unwrap();
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

#[test]
fn resource_statistics_and_version_do_not_become_extra_capture_policy() {
    let p = plan();
    let mut observed = status(&p);
    observed.version = u64::MAX;
    observed.cycles = u128::MAX.into();
    observed.ready_for_migration = true;
    observed.settings.minimum_incoming_canister_call_cycles = u128::MAX.into();
    let original = observed.clone();
    assert_eq!(p.validate_status(&observed), Ok(()));
    assert_eq!(observed, original);
}
