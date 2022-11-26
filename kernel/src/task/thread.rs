
use alloc::{vec, vec::Vec};

use crate::interface::poll_future;

pub const USER_STACK_SIZE: usize = 0x1000;

/// ThreadContext，用户线程上下文
/// 根据 ra 的值来判断线程是否执行结束
/// 协程执行结束，ra 设置为 0
/// 协程未执行完，进行线程切换，回到主线程，则会保存 ra，等待下次调度继续执行
#[repr(C)]
pub struct ThreadContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

impl ThreadContext {
    pub const EMPTY: Self = Self {
        ra: 0,
        sp: 0,
        s: [0; 12],
    };
}
/// 用户线程，只是提供一个栈，记录正在运行的协程
pub struct Thread {
    // current: Option<CoroutineId>,
    main_context: ThreadContext,
    context: ThreadContext,
    stack: Vec<u8>,
}

impl Thread {
    pub fn new() -> Self {
        let mut thread = Self { 
            // current: None, 
            main_context: ThreadContext::EMPTY,
            context: ThreadContext::EMPTY, 
            stack: vec![0u8; USER_STACK_SIZE], 
        };
        thread.context.sp = thread.stack.as_ptr() as usize + USER_STACK_SIZE;
        thread.context.ra = poll_future as usize;
        thread
    }
    // 根据传递进来的协程，进行初始化，并且切换到该线程执行协程
    pub fn execute(&mut self) {
        unsafe {
            core::arch::asm!(
                "mv a1, {next_ctx_ptr}",
                "mv a0, {main_ctx_ptr}",
                "call {execute_naked}",
                next_ctx_ptr = in(reg) &self.context as *const ThreadContext as usize,
                main_ctx_ptr = in(reg) &self.main_context as *const ThreadContext as usize,
                execute_naked = sym execute_naked
            )
        }
    }

}

pub fn yield_thread(ctx_addr: usize) {
    unsafe {
        core::arch::asm!(
        r"  .altmacro
            .macro LOAD_SN n
                ld s\n, (\n+2)*8(a0)
            .endm
            
            mv a0, {a0}
            ld ra, 0(a0)
            .set n, 0
            .rept 12
                LOAD_SN %n
                .set n, n + 1
            .endr
            ld sp, 8(a0)
            ret",
            a0  = in(reg) ctx_addr,
            options(noreturn)
        )
    }
}

#[naked]
unsafe extern "C" fn execute_naked() {
    core::arch::asm!(
    r"  .altmacro
        .macro SAVE_SN n
            sd s\n, (\n+2)*8(a0)
        .endm
        .macro LOAD_SN n
            ld s\n, (\n+2)*8(a1)
        .endm
        sd ra, 0(a0)
        .set n, 0
        .rept 12
            SAVE_SN %n
            .set n, n + 1
        .endr
        sd sp, 8(a0)

        ld ra, 0(a1)
        .set n, 0
        .rept 12
            LOAD_SN %n
            .set n, n + 1
        .endr
        ld sp, 8(a1)
        ret",
        options(noreturn),
    )
}




