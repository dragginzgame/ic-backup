//! Test-only actual artifact reader accounting, scoped to one thread and invocation.

use std::{
    cell::Cell,
    io::{self, Read},
};

thread_local! {
    static BYTES: Cell<Option<u64>> = const { Cell::new(None) };
}

pub(super) fn measure<T>(run: impl FnOnce() -> T) -> (T, u64) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            BYTES.with(|bytes| bytes.set(None));
        }
    }
    BYTES.with(|bytes| {
        assert!(bytes.get().is_none());
        bytes.set(Some(0));
    });
    let reset = Reset;
    let result = run();
    let count = BYTES.with(|bytes| bytes.get().unwrap());
    drop(reset);
    (result, count)
}

pub(super) struct Reader<R>(pub R);
impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.0.read(buffer)?;
        BYTES.with(|bytes| {
            if let Some(previous) = bytes.get() {
                bytes.set(Some(previous + count as u64));
            }
        });
        Ok(count)
    }
}
