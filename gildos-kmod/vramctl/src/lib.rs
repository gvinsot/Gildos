//! `vramctl` — VRAM as a first-class managed resource (§4.4.6).
#![no_std]

/// Opaque handle into a pinned VRAM region.
#[derive(Clone, Copy, Debug)]
pub struct VramHandle(pub u64);

bitflags::bitflags! {
    /// Allocation flags for `reserve`.
    pub struct Flags: u32 {
        /// Mark as `model-resident` — never evictable.
        const MODEL_RESIDENT = 0b0001;
        /// Mark as `kv-cache` — evictable only when no INT/RT cog work runs.
        const KV_CACHE       = 0b0010;
        /// Pin (do not move) for the lifetime of the handle.
        const PIN            = 0b0100;
    }
}

#[derive(Debug)]
pub enum Error { OutOfMemory, InvalidHandle, NotPermitted }

/// Reserve `bytes` of VRAM with the given flags.
pub fn reserve(_bytes: u64, _flags: Flags) -> Result<VramHandle, Error> {
    Err(Error::OutOfMemory)
}

/// Release a previously-reserved VRAM region.
pub fn release(_h: VramHandle) -> Result<(), Error> { Ok(()) }
