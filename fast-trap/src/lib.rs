//! 陷入模块
//! 

#![no_std]
#![deny(warnings, missing_docs)]
#![feature(naked_functions, asm_const)]

mod hal;
mod stack;

pub use stack::Stack;
pub use hal::{FlowContext, skip_context};