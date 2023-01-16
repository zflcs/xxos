use super::{PhysAddr, PhysPageNum};
use core::{fmt, fmt::{Debug, Formatter}};
use config::{MEMORY_END, STACK_SIZE, PAGE_SIZE};
use alloc::vec::Vec;
use buddy_system_allocator::LockedFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: LockedFrameAllocator = LockedFrameAllocator::new();
}
pub fn init_frame_allocator() {
    extern "C" {
        fn end();
    }
    // println!("{:#x}-{:#x}", PhysAddr::from(end as usize).0, PhysAddr::from(MEMORY_END).0);
    FRAME_ALLOCATOR.lock().add_frame(
        PhysAddr::from(end as usize).ceil().0,
        PhysAddr::from(MEMORY_END).floor().0,
    );
}

pub fn stack_alloc() -> Option<usize> {
    FRAME_ALLOCATOR
        .lock()
        .alloc(STACK_SIZE / PAGE_SIZE)
        .map(|ppn| ppn * PAGE_SIZE)
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .lock()
        .alloc(1)
        .map(|p| FrameTracker::new(PhysPageNum(p)))
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn.0, 1);
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}


pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}