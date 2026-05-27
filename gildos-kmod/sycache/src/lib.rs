//! `sycache` — speculative syscall prefetch (§4.4.5).
#![no_std]

#[derive(Clone, Copy, Debug)]
pub struct SpecToken(pub u64);

#[derive(Debug)]
pub enum Error { NotSpeculatable, AlreadyCommitted, BadToken }

/// Issue a speculative syscall. The call descriptor MUST be one whose
/// capability manifest declares `speculative: idempotent` or
/// `speculative: deferred_commit` (see SPECIFICATION.md §8).
pub fn speculate(_call_descriptor: &[u8]) -> Result<SpecToken, Error> {
    Err(Error::NotSpeculatable)
}
pub fn commit(_t: SpecToken) -> Result<(), Error> { Ok(()) }
pub fn cancel(_t: SpecToken) -> Result<(), Error> { Ok(()) }
