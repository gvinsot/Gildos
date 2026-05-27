//! `dmload` — DirectModelLoad (§4.4.1).
//!
//! Zero-copy NVMe → VRAM streaming of signed model weights. Bypasses
//! the page cache; hashes on the fly against the signed manifest.
//!
//! Phase-1 status: USERSPACE shim only. The kernel-side module will land
//! once Rust-for-Linux upstream exposes the IO-uring and GPUDirect
//! Storage hooks we need.
#![no_std]

/// Opaque handle into a pinned VRAM region (see `vramctl`).
#[derive(Clone, Copy, Debug)]
pub struct VramHandle(pub u64);

/// Result of a `dmload` operation: number of bytes streamed.
pub type BytesLoaded = u64;

/// Errors that `dmload` may return.
#[derive(Debug)]
pub enum Error {
    /// The source file is not on a signed read-only mount.
    UnsignedSource,
    /// On-the-fly hash does not match the signed manifest.
    HashMismatch,
    /// The VRAM region is too small for the requested transfer.
    VramTooSmall,
    /// Underlying I/O failure.
    Io,
}

/// Stream `len` bytes from `fd` at `offset` directly into `vram`.
///
/// Phase-1 stub: returns `Err(Error::Io)`. The MVG (§1.5) requires this
/// path to saturate ≥ 80 % of NVMe Gen4 read bandwidth on the reference
/// workstation; the conformance suite is in §4.7.
pub fn dmload(
    _fd: i32,
    _offset: u64,
    _len: u64,
    _vram: VramHandle,
) -> Result<BytesLoaded, Error> {
    Err(Error::Io)
}
