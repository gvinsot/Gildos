//! `promptpin` — Persistent Prompt Objects (§4.4.8).
#![no_std]

#[derive(Clone, Copy, Debug)]
pub struct PromptId(pub u64);

#[derive(Clone, Copy, Debug)]
pub struct ContextHandle(pub u64);

#[derive(Debug)]
pub enum Error { UnknownPrompt, BadContext, ModelMismatch }

pub fn pin(_p: PromptId, _ctx: ContextHandle) -> Result<(), Error> { Ok(()) }
pub fn unpin(_p: PromptId, _ctx: ContextHandle) -> Result<(), Error> { Ok(()) }
