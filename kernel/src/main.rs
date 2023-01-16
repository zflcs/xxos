#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
#![deny(warnings)]

mod console;
mod trap;

#[macro_use]
extern crate rcore_console;


use sbi_rt::*;
use config::STACK_SIZE;
use fast_trap::{Stack, skip_context, FlowContext};
use trap::kern_process;

#[link_section = ".bss.stack"]
static mut STACK: Stack = Stack([0; STACK_SIZE]);

/// 设置栈并跳转到 Rust。
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    // 在栈顶已经预留上下文的空间，sscratch 指向上下文的起始地址
    
    core::arch::asm!(
        "   la   sp, {stack} + {stack_size}
            call {skip_context}
            j    {main}
        ",
        stack_size      = const STACK_SIZE,
        stack           = sym STACK,
        skip_context    = sym skip_context,
        main            = sym rust_main,
        options(noreturn),
    )
}


extern "C" fn rust_main() -> ! {
    // 初始化内存布局，bss 段清零
    unsafe { linker::zero_bss(); }
    // 初始化 `console`
    console::init_console();
    vmm::init();
    println!("vmm init done");
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
    fast_trap::trap_init();
    let init_proc = task::Process::new();
    init_proc.execute();
    system_reset(Shutdown, NoReason);
    unreachable!()
}


/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    system_reset(Shutdown, SystemFailure);
    unreachable!()
}





