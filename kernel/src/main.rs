#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const, default_alloc_error_handler)]
#![deny(warnings)]

mod console;
mod mm;
mod task;
mod syscall;
mod processor;

extern crate alloc;


use sbi_rt::*;
use printlib::*;
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, Ordering},
    hint
};
use spin::Lazy;
use alloc::collections::BTreeMap;
use core::ffi::CStr;
use processor::init_processor;
use kernel_vm::{
    page_table::{PPN, Sv39, VmFlags, MmuMeta},
};
use processor::PROCESSOR;
use mm::kernel_space;



/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
    unreachable!()
}


/// 加载用户进程。
static APPS: Lazy<BTreeMap<&'static str, &'static [u8]>> = Lazy::new(|| {
    extern "C" {
        static apps: utils::AppMeta;
        static app_names: u8;
    }
    unsafe {
        apps.iter_elf()
            .scan(&app_names as *const _ as usize, |addr, data| {
                let name = CStr::from_ptr(*addr as _).to_str().unwrap();
                *addr += name.as_bytes().len() + 1;
                Some((name, data))
            })
    }
    .collect()
});

// 应用程序内联进来。
core::arch::global_asm!(include_str!(env!("APP_ASM")));

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
    let layout = linker::KernelLayout::locate();
    // bss 段清零
    unsafe { layout.zero_bss() };
    // 初始化 console
    console::init_console();
    // 初始化 syscall
    syscall::init_syscall();
    // 初始化内核堆
    mm::init();
    mm::test();
    log::error!("hart_id {}", hart_id());
    hart_start();
    AP_CAN_INIT.store(true, Ordering::Relaxed);
    init_processor();
    // 建立内核地址空间
    let mut ks = kernel_space(layout);
    let tramp = (
        PPN::<Sv39>::new(unsafe { &PROCESSOR.portal } as *const _ as usize >> Sv39::PAGE_BITS),
        VmFlags::build_from_str("XWRV"),
    );
    // 传送门映射到所有地址空间
    ks.map_portal(tramp);
    // println!("{}", env!("APP_ASM"));
    // println!("{}", include_str!(env!("APP_ASM")));

    println!("/**** APPS ****");
    APPS.keys().for_each(|app| println!("{app}"));
    println!("**************/");
    loop {
        
    }
    // system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    // unreachable!()
}

/// init_other_hart
/// 
/// 初始化其他的核，需要等待主核初始化之后，通过 hsm start 才可以初始化，在这之前一直自旋等待
extern "C" fn secondary_main() {
    while !AP_CAN_INIT.load(Ordering::Relaxed) {
        hint::spin_loop();
    }
    log::error!("hart_id {}", hart_id());
}

