//! Restore consumes the shared policy with its own approved management sender.
use crate::store::{
    CaptureStatusError, CaptureStatusReplyError, RestorePlan,
    capture::test_fixtures::{
        completion::{complete, settled},
        http::runtime,
        source::maximum,
    },
    restore::tests::fixtures::declaration,
};
use ic_platform::{Principal, candid_codec, management_canister_types::CanisterStatusType};
use std::fs;

#[test]
fn restore_status_requires_its_sender_and_source_module_without_writing_intent() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let ledger_path = f.ledger();
    let before = fs::read(&ledger_path).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    let plan = RestorePlan::new(&source, declaration(&source)).unwrap();
    let mut observed = crate::store::capture::test_fixtures::source::status(&f.p);
    assert!(
        !observed
            .settings
            .controllers
            .contains(&plan.request().sender)
    );
    assert_eq!(
        plan.validate_status(&observed),
        Err(CaptureStatusError::NotController)
    );
    observed.settings.controllers = vec![plan.request().initiator];
    assert_eq!(
        plan.validate_status(&observed),
        Err(CaptureStatusError::NotController)
    );
    observed.settings.controllers.push(plan.request().sender);
    let original = observed.clone();
    assert_eq!(plan.validate_status(&observed), Ok(()));
    assert_eq!(
        plan.validate_status_reply(&candid_codec::encode_one(&observed, 4096).unwrap()),
        Ok(())
    );
    assert_eq!(observed, original);
    for state in [CanisterStatusType::Running, CanisterStatusType::Stopping] {
        observed.status = state;
        assert_eq!(
            plan.validate_status(&observed),
            Err(CaptureStatusError::NotStopped(state))
        );
    }
    observed.status = CanisterStatusType::Stopped;
    for hash in [
        None,
        Some(vec![]),
        Some(vec![0; 31]),
        Some(vec![0; 32]),
        Some(vec![0; 33]),
    ] {
        observed.module_hash = hash;
        assert_eq!(
            plan.validate_status(&observed),
            Err(CaptureStatusError::ModuleMismatch)
        );
        assert!(
            plan.validate_status_reply(&candid_codec::encode_one(&observed, 4096).unwrap())
                .is_err()
        );
    }
    assert_eq!(fs::read(ledger_path).unwrap(), before);
    assert_eq!(server.count(), 1);
    assert!(
        !f.root
            .join("operations")
            .join(&plan.request().operation_id)
            .exists()
    );
}

#[test]
fn restore_status_shares_controller_and_exact_wire_bounds() {
    let (mut f, _, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    let plan = RestorePlan::new(&source, declaration(&source)).unwrap();
    let mut observed = crate::store::capture::test_fixtures::source::status(&f.p);
    observed.settings.controllers = vec![plan.request().sender; 2];
    assert_eq!(
        plan.validate_status(&observed),
        Err(CaptureStatusError::DuplicateController)
    );
    assert_eq!(
        plan.validate_status_reply(&candid_codec::encode_one(&observed, 4096).unwrap()),
        Err(CaptureStatusReplyError::Status(
            CaptureStatusError::DuplicateController
        ))
    );
    observed.settings.controllers = (0_u32..10)
        .map(|n| Principal::from_slice(&n.to_be_bytes()))
        .collect();
    observed.settings.controllers[0] = plan.request().sender;
    let raw = candid_codec::encode_one(&observed, 4096).unwrap();
    assert_eq!(plan.validate_status_reply(&raw), Ok(()));
    observed
        .settings
        .controllers
        .push(Principal::from_slice(&[99, 1]));
    assert_eq!(
        plan.validate_status(&observed),
        Err(CaptureStatusError::ControllerLimit)
    );
    assert!(matches!(
        plan.validate_status_reply(&candid_codec::encode_one(&observed, 4096).unwrap()),
        Err(CaptureStatusReplyError::Decode(_))
    ));
    assert_eq!(
        plan.validate_status_reply(&vec![0; 1024 * 1024 + 1]),
        Err(CaptureStatusReplyError::ReplyTooLarge)
    );
    for bytes in [
        vec![],
        b"invalid".to_vec(),
        raw[..raw.len() - 1].to_vec(),
        [raw.as_slice(), &[0]].concat(),
    ] {
        assert!(matches!(
            plan.validate_status_reply(&bytes),
            Err(CaptureStatusReplyError::Decode(_))
        ));
    }
}
