#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
#![feature(default_alloc_error_handler)]
#![deny(warnings)]


mod consoleimpl;
mod trapstack;
mod config;

#[macro_use]
extern crate printlib;

use sbi_rt::*;
use fast_trap::{
    reuse_stack_for_trap, 
    FastResult, FlowContext, FreeTrapStack, FastContext
};
use riscv::register::*;
use core::ptr::NonNull;
use config::STACK_SIZE;
use trapstack::{Stack, StackRef};


#[link_section = ".bss.uninit"]
static mut ROOT_STACK: Stack = Stack([0; STACK_SIZE]);
// static mut FREE_STACK: Stack = Stack([0; STACK_SIZE]);
static mut ROOT_CONTEXT: FlowContext = FlowContext::ZERO;

/// 设置栈并跳转到 Rust。
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "   la   sp, {stack} + {stack_size}
            call {move_stack}
            j    {main}
        ",
        stack_size = const STACK_SIZE,
        stack      =   sym ROOT_STACK,
        move_stack =   sym reuse_stack_for_trap,
        main       =   sym rust_main,
        options(noreturn),
    )
}

extern "C" fn rust_main() -> ! {
    let layout = linker::KernelLayout::locate();
    // bss 段清零
    unsafe { layout.zero_bss() };
    // 初始化 `console`
    consoleimpl::init_console();
    let context_ptr = unsafe { NonNull::new_unchecked(&mut ROOT_CONTEXT) };
    // 测试构造和释放
    let _ = FreeTrapStack::new(
        StackRef(unsafe { &mut ROOT_STACK }),
        context_ptr,
        fast_handler,
    )
    .unwrap();

    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    unreachable!()
}

/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
    unreachable!()
}


extern "C" fn fast_handler(
    mut ctx: FastContext,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
) -> FastResult {
    // use {scause::Exception as E, scause::Trap as T};
    let cause = scause::read();
    log::debug!("fast trap: {:?}({})", cause.cause(), cause.bits());
    ctx.regs().a = [ctx.a0(), a1, a2, a3, a4, a5, a6, a7];
    ctx.restore()
}



