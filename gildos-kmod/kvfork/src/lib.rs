//! `kvfork` — Cognitive Fork (§4.4.7).
//!
//! Copy-on-write clone of a paged-attention KV cache. Enables
//! tree-of-thoughts and shadow evaluation without re-prefill.
#![no_std]

#[derive(Clone, Copy, Debug)]
pub struct ContextHandle(pub u64);

#[derive(Debug)]
pub enum MergePolicy { TakeBest, MajorityVote, Concat }

#[derive(Debug)]
pub enum Error { BadHandle, OutOfPages }

pub fn fork(_parent: ContextHandle) -> Result<ContextHandle, Error> { Err(Error::BadHandle) }
pub fn merge(_children: &[ContextHandle], _policy: MergePolicy) -> Result<ContextHandle, Error> {
    Err(Error::BadHandle)
}
pub fn discard(_c: ContextHandle) -> Result<(), Error> { Ok(()) }
