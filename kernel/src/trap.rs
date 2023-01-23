use fast_trap::FlowContext;


/// 内核进程
#[allow(unused)]
pub fn kern_process() {
    log::info!("into kernel");
    
        let ctx = unsafe { *((usize::MAX - core::mem::size_of::<FlowContext>() + 1) as *mut usize as *mut FlowContext) };
        log::info!("{:#x?}", ctx);
    
}