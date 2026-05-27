//! Trivial compile-time test: confirm the public surface is usable.
use gildos_dmload::{dmload, Error, VramHandle};

#[test]
fn stub_returns_io_error() {
    let h = VramHandle(0);
    match dmload(0, 0, 0, h) {
        Err(Error::Io) => {}
        _ => panic!("expected Phase-1 stub to return Err(Error::Io)"),
    }
}
