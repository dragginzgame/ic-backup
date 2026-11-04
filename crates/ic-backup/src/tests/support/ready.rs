//! Poll immediate fixture providers without nesting a runtime around blocking Testkit.

use std::{
    future::Future,
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
};

/// Immediate fixtures must finish on their first poll; this is not an async executor.
pub fn ready<F: Future>(future: F) -> F::Output {
    match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture unexpectedly requires an async executor"),
    }
}

/// Poll one borrowed fixture future; the caller retains ownership and chooses when to cancel.
pub fn assert_pending<F: Future + ?Sized>(future: Pin<&mut F>) {
    assert!(
        future
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending(),
        "fixture unexpectedly completed"
    );
}
