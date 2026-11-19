//! Adapter-boundary policy only. No simulated transport or IC effects.

mod retained;

use super::*;

struct Binding;
impl RootBinding for Binding {
    fn execute(&self) -> &[u8] {
        &[80]
    }

    fn lookup(&self) -> &[u8] {
        &[81]
    }

    fn decode(&self, exchange: RootExchange, bytes: &[u8]) -> Option<RootOutcome> {
        assert_eq!(exchange, RootExchange::Lookup);
        match bytes {
            [0] => Some(RootOutcome::Pending),
            [1] => Some(RootOutcome::Reply(vec![3, 4])),
            [2] => Some(RootOutcome::Rejected(5, "no".into())),
            [3] => Some(RootOutcome::Refused),
            [4] => Some(RootOutcome::Reply(vec![1; 3])),
            _ => None,
        }
    }
}

#[test]
fn outer_rejection_is_unresolved_and_never_reaches_inner_decoder() {
    let call = Call::Root {
        binding: Box::new(Binding),
        reply_bytes: 2,
    };
    assert_eq!(
        call.result(RootExchange::Execute, true, vec![2]).unwrap(),
        None
    );
    for bytes in [vec![0], vec![9]] {
        assert_eq!(
            call.result(RootExchange::Lookup, false, bytes).unwrap(),
            None
        );
    }
    assert_eq!(
        call.result(RootExchange::Lookup, false, vec![1]).unwrap(),
        Some((false, vec![3, 4]))
    );
    assert_eq!(
        call.result(RootExchange::Lookup, false, vec![2]).unwrap(),
        Some((true, b"no".to_vec()))
    );
    assert!(matches!(
        call.result(RootExchange::Lookup, false, vec![3]),
        Err(Error::RootRefused)
    ));
    assert_eq!(
        call.result(RootExchange::Lookup, false, vec![4]).unwrap(),
        None
    );
}
