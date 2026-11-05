//! Poll immediate in-memory fixtures or pre-dispatch refusals; never drive simulator IO.

use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

/// Immediate fixtures must finish on their first poll; this is not an async executor.
pub fn ready<F: Future>(future: F) -> F::Output {
    match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture unexpectedly requires an async executor"),
    }
}
