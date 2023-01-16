#![no_std]
#![feature(asm_const)]

extern crate alloc;

mod process;
mod id;

use id::ProcId;
pub use process::Process;