
use core::ptr::NonNull;

use alloc::{sync::Arc};
use config::TRAMPOLINE;
use fast_trap::{Stack, FlowContext, alloc_stack, restore, trap_entry};
use spin::Mutex;
use vmm::{MemorySet};
use super::ProcId;

pub struct Process {
    pub pid: ProcId,
    pub inner: Mutex<ProcessInner>,
}

pub struct ProcessInner {
    pub space: MemorySet,
    pub stack: NonNull<Stack>,
    pub ctx: NonNull<FlowContext>,
}

impl Process {

    pub fn execute(self: Arc<Self>) {
        let satp = self.inner.lock().space.token();
        let restore = restore as usize - trap_entry as usize + TRAMPOLINE;
        log::info!("{:#x}", restore);
        unsafe { 
            core::arch::asm!(
                "fence.i",
                "jr a1",
                in("a0") satp,
                in("a1") restore,
            );
        }
    }

    pub fn new() -> Arc<Self> {
        let pid = ProcId::new();
        let mut space = MemorySet::new_bare();
        let stack;
        if let Some(s) = alloc_stack() {
            stack = s;
        } else {
            panic!("alloc stack failed");
        }
        space.map_vdso();
        space.map_trampoline();
        space.map_stack(stack.as_ptr() as *mut usize as usize);
        let mut ctx = unsafe { stack.as_ref().context() };
        unsafe { 
            ctx.as_mut().pc = vdso::user_entry as usize;
            ctx.as_mut().sp = usize::MAX - core::mem::size_of::<FlowContext>() + 1;
        };
        Arc::new(Self {
            pid,
            inner: Mutex::new(ProcessInner {
                space,
                stack,
                ctx,
            }),
        })
    } 
}