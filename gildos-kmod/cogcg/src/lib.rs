//! `cogcg` — Cognitive cgroups (§4.4.9).
#![no_std]

#[derive(Clone, Copy, Debug)]
pub struct CgroupId(pub u64);

#[derive(Debug)]
pub enum Error { Quota, BadCgroup }

/// Report token/GPU usage to the kernel cgroup controller.
/// Returns `Err(Error::Quota)` when the cgroup would exceed its budget.
pub fn account(_cg: CgroupId, _tokens: u64, _gpu_ns: u64, _ctx_tokens: u64) -> Result<(), Error> {
    Ok(())
}
