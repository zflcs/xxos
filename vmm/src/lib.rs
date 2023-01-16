#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate rcore_console;
#[macro_use]
extern crate lazy_static;
use core::arch::asm;

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker, stack_alloc};
use linker::locate_stack;
pub use memory_set::{kernel_token, MapPermission, MemorySet, KERNEL_SPACE};
use page_table::PTEFlags;
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, PageTable,
    PageTableEntry, UserBuffer, UserBufferIterator,
};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate();
    // 将 sp 寄存器移动到高位虚拟地址，取消掉 stack 段的对等映射
    let mut sp: usize;
    unsafe { asm!("mv {sp}, sp", sp = out(reg) sp); }
    sp = sp | 0xfffffffffffff000;
    unsafe { 
        asm!(
            "mv sp, {sp}", 
            sp = in(reg) sp,         
        );
    }
    KERNEL_SPACE.lock().remove_area_with_start_vpn(locate_stack().start.into());
}
