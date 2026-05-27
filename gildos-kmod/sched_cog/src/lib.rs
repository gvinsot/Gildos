//! `sched_cog` — cognitive scheduling class (§4.4.3).
#![no_std]

/// Cognitive task priority. See §4.4.3 / §5.5.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class { Rt = 0, Int = 1, Bg = 2, Opt = 3 }

#[derive(Debug)]
pub enum Error { NoSuchPid, NotPermitted }

/// Move `pid` into the given cognitive class.
pub fn set_class(_pid: i32, _class: Class, _flags: u32) -> Result<(), Error> { Ok(()) }
