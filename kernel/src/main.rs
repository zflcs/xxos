#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![deny(warnings)]

mod console;
// mod sync;

use sbi_rt::*;
use printlib::*;
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, Ordering},
    hint
};


/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
    unreachable!()
}

/// Supervisor 汇编入口。
///
/// 设置栈并跳转到 Rust。
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start(hart_id: usize) -> ! {
    const STACK_SIZE: usize = 4096 * 16 * 4;

    #[link_section = ".bss.uninit"]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    core::arch::asm!(
        "mv tp, a0",                // a0 表示 hart_id
        "la sp, {stack} + {stack_size}",
        "addi t0, a0, 1",           // t0 = a0 + 1
        "slli t0, t0, 16",          // t0 左移 16 位
        "add sp, sp, t0",           // sp = sp + t0
        "bne x0, a1, 1f",
        "j  {secondary_main}",
        "1: j  {primary_main}",
        stack_size      = const STACK_SIZE,
        stack           = sym STACK,
        primary_main    = sym primary_main,
        secondary_main  = sym secondary_main,
        options(noreturn),
    )
}

/// bss 段清零。
///
/// 需要定义 sbss 和 ebss 全局符号才能定位 bss。
#[inline]
fn zero_bss() {
    extern "C" {
        static mut sbss: u64;
        static mut ebss: u64;
    }
    unsafe { r0::zero_bss(&mut sbss, &mut ebss) };
}

/// hart_id
/// 
/// 获取硬件线程 id
#[inline]
pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}


/// send_ipi
/// 
/// 需要使用 rustsbi
#[inline]
pub fn hart_start() {
    let hart_id = hart_id();
    for i in 0..4 {
        if i != hart_id {
            sbi_rt::hart_start(i, 0x80200000, 0);
        }
    }
}

static AP_CAN_INIT: AtomicBool = AtomicBool::new(false);

/// 主核初始化
extern "C" fn primary_main() -> ! {
    zero_bss();
    console::init_console();
    log::error!("hart_id {}", hart_id());
    hart_start();
    AP_CAN_INIT.store(true, Ordering::Relaxed);
    loop {
        
    }
    // system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    // unreachable!()
}

/// init_other_hart
/// 
/// 初始化其他的硬件核，需要等待主核初始化之后，发送中断之后才可以初始化，在这之前一直自旋等待
extern "C" fn secondary_main() {
    while !AP_CAN_INIT.load(Ordering::Relaxed) {
        hint::spin_loop();
    }
    log::error!("hart_id {}", hart_id());
}

