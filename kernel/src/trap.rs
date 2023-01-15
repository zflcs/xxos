use fast_trap::FlowContext;


/// 内核进程
pub fn kern_process() {
    loop {
        let sp = usize::MAX - core::mem::size_of::<FlowContext>() + 1;
        let ra = kern_process as usize;
        unsafe {
            core::arch::asm!(
                "sd {sp}, -1*8(x0)",
                "sd {ra}, -2*8(x0)",
                sp = in(reg) sp,
                ra = in(reg) ra,
            );
        }
        let ctx = unsafe { *((usize::MAX - core::mem::size_of::<FlowContext>() + 1) as *mut usize as *mut FlowContext) };
        log::info!("{:#x?}", ctx);
    }
}