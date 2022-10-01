use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use alloc::alloc::handle_alloc_error;
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;

/// HEAP 指向的是用户进程的 HEAP
static mut HEAP: Option<&mut MutAllocator<32>> = None;

pub fn init(heap: &'static mut MutAllocator<32>) {
    // 将用户进程堆的指针传递给共享库的堆，从而使得可以在用户进程的堆中分配数据
    unsafe { HEAP = Some(heap) };
}

struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok((ptr, _)) = HEAP.as_mut().unwrap().allocate_layout::<u8>(layout) {
            ptr.as_ptr()
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.as_mut().unwrap().deallocate_layout(NonNull::new(ptr).unwrap(), layout)
    }
}
