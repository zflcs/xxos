use alloc::alloc::handle_alloc_error;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
use runtime::Executor;


pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;
#[no_mangle]
#[link_section = ".data.heap"]
pub static mut HEAP: MutAllocator<32> = MutAllocator::new();

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Executor = Executor::new();

// 托管空间 16 KiB
const MEMORY_SIZE: usize = 16 << 10;
#[no_mangle]
#[link_section = ".data.memory"]
static mut MEMORY: [u8; MEMORY_SIZE] = [0u8; MEMORY_SIZE];




/// 初始化全局分配器和内核堆分配器。
pub fn init() {
    // printlib::log::debug!("heap {:#x}", unsafe{ &mut HEAP as *mut MutAllocator<32> as usize });
    // printlib::log::debug!("heap {:#x}", core::mem::size_of::<MutAllocator<32>>());
    // printlib::log::debug!("memory {:#x}", unsafe{ &mut MEMORY as *mut u8 as usize });
    
    unsafe {
        HEAP.init(
            core::mem::size_of::<usize>().trailing_zeros() as _,
            NonNull::new(MEMORY.as_mut_ptr()).unwrap(),
        );
        HEAP.transfer(NonNull::new_unchecked(MEMORY.as_mut_ptr()), MEMORY.len());
    }
}


struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok((ptr, _)) = HEAP.allocate_layout::<u8>(layout) {
            ptr.as_ptr()
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.deallocate_layout(NonNull::new(ptr).unwrap(), layout)
    }
}
