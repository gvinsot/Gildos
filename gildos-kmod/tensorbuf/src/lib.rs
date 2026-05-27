//! `tensorbuf` — tensor handles (§4.4.2).
//!
//! DMA-buf file descriptor + shape/dtype sidecar for zero-copy tool
//! dispatch.
#![no_std]

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Dtype { F32, F16, Bf16, I8, U8 }

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Handle {
    pub shape_rank: u32,
    pub shape:      [u32; 8],
    pub dtype:      Dtype,
    pub layout:     u32,
    pub byte_len:   u64,
    pub device_mask:u64,
    pub provenance: [u8; 32],
}

#[derive(Debug)]
pub enum Error { BadShape, NoMemory, NotPermitted }

pub fn create(_shape: &[u32], _dtype: Dtype, _device_mask: u64) -> Result<Handle, Error> {
    Err(Error::NoMemory)
}
/// Share `h` with `target_pid`; returns a dma-buf-class fd as i32.
pub fn share(_h: Handle, _target_pid: i32) -> Result<i32, Error> { Err(Error::NotPermitted) }
