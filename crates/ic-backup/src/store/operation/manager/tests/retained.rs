//! Replay the lost-take slot from the installed Root/child host-restart witness.
//! Canic independently prepared the binding and decoded the exact original inner
//! reply. This fixture codec recognizes only that historical exchange.

use super::super::*;
use crate::store::{
    CapturePlan, StatusEvidence,
    operation::{Call as Slot, Invocation, ReferenceRecord, custody::Role, intent::Intent, replay},
};
use ic_platform::ingress::{Content, decode_envelope};
use ic_platform_protocol::management_snapshots::TakeSnapshotArgs;

const EXECUTE: &[u8] = include_bytes!("fixtures/take-envelope.bin");
const LOOKUP: &[u8] = include_bytes!("fixtures/lookup-envelope.bin");
const RESULT: &[u8] = include_bytes!("fixtures/lookup-result.bin");
const INNER: &[u8] = include_bytes!("fixtures/inner-reply.bin");
const EXECUTE_REFERENCE: u64 = 1_794_996_924_874_121_711;
const LOOKUP_REFERENCE: u64 = 1_794_996_925_316_189_214;

#[derive(Debug)]
struct RetainedCodec;
struct Bound(Vec<u8>, Vec<u8>);
fn frame(bytes: &[u8]) -> Vec<u8> {
    let Content::Call { arg, .. } = decode_envelope(bytes, bytes.len()).unwrap().value.content
    else {
        panic!("fixture update")
    };
    arg
}
impl RootCodec for RetainedCodec {
    fn protocol(&self) -> [u8; 32] {
        [84; 32]
    }

    fn prepare(&self, request: RootRequest<'_>, _: u32) -> Option<Box<dyn RootBinding>> {
        if request.id != [1; 16]
            || request.step != Step::TakeSnapshot
            || request.method != "take_canister_snapshot"
            || request.arguments_bytes != 57
            || request.expires_at_ns != EXECUTE_REFERENCE + 120_000_000_000
            || request.arguments_hash.to_string()
                != "e6138a1846a19b40def26cea3f7f685aa11d2e37c015f835630257ea73d14dad"
        {
            return None;
        }
        Some(Box::new(Bound(frame(EXECUTE), frame(LOOKUP))))
    }
}
impl RootBinding for Bound {
    fn execute(&self) -> &[u8] {
        &self.0
    }

    fn lookup(&self) -> &[u8] {
        &self.1
    }

    fn decode(&self, exchange: RootExchange, payload: &[u8]) -> Option<RootOutcome> {
        (exchange == RootExchange::Lookup
            && payload == StatusEvidence::decode(RESULT).unwrap().response)
            .then(|| RootOutcome::Reply(INNER.to_vec()))
    }
}

#[test]
fn cold_lookup_authenticates_both_real_ingresses_and_applies_only_the_original_inner_reply() {
    let mut intent = CapturePlan::decode(include_bytes!("fixtures/capture-intent.json")).unwrap();
    intent.attach_codec(Some(&RetainedCodec));
    let effect = Effect::Take(TakeSnapshotArgs {
        canister_id: intent.request().target.canister,
        replace_snapshot: None,
        uninstall_code: Some(false),
        sender_canister_version: None,
    });
    let slot = Slot {
        step: Step::TakeSnapshot,
        update: Invocation {
            reference_ns: EXECUTE_REFERENCE,
            request: Some(ReferenceRecord::new(EXECUTE).unwrap()),
            submitted: true,
            result: None,
        },
        manager_lookup: Some(Invocation {
            reference_ns: LOOKUP_REFERENCE,
            request: Some(ReferenceRecord::new(LOOKUP).unwrap()),
            submitted: true,
            result: Some(ReferenceRecord::new(RESULT).unwrap()),
        }),
        observation: None,
    };
    let effect = (effect, 512);
    for fault in [
        None,
        Some(Role::Request),
        Some(Role::ManagerLookupRequest),
        Some(Role::ManagerLookupResult),
    ] {
        let resolved = replay::resolve_call(intent.view(), &slot, &effect, &mut |_, role| {
            let mut bytes = match role {
                Role::Request => EXECUTE,
                Role::ManagerLookupRequest => LOOKUP,
                Role::ManagerLookupResult => RESULT,
                _ => panic!("unexpected evidence role"),
            }
            .to_vec();
            if fault.is_some_and(|f| std::mem::discriminant(&f) == std::mem::discriminant(&role)) {
                *bytes.last_mut().unwrap() ^= 1;
            }
            Ok(bytes)
        });
        if fault.is_some() {
            assert!(resolved.is_err());
            continue;
        }
        let result = resolved.unwrap().unwrap();
        assert!(!result.rejected);
        assert_eq!(result.response, INNER);
        let request_id = |bytes: &[u8]| {
            decode_envelope(bytes, bytes.len())
                .unwrap()
                .value
                .content
                .request_id()
                .unwrap()
        };
        assert_eq!(result.request_id, request_id(EXECUTE));
        assert_ne!(result.request_id, request_id(LOOKUP));
    }
}
