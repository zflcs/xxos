#![no_std]
pub const STACK_SIZE: usize = 0x2000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;

pub const MEMORY_END: usize = 0x88000000;

pub const SP: usize = 0;
pub const STACK_START: usize = usize::MAX - STACK_SIZE + 1;
pub const TRAMPOLINE: usize = STACK_START - PAGE_SIZE;

