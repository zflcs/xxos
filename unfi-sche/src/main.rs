#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions, asm_sym)]
mod heap;
mod thread;
mod executor;
mod interface;

extern crate printlib;
extern crate alloc;

use printlib::*;
use heap::MutAllocator;
use sbi_rt::*;
use runtime::Executor;
use interface::{add_coroutine, run};
use alloc::boxed::Box;


static mut SECONDARY_INIT: usize = 0usize;


/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
    unreachable!()
}

// #[link_section = ".bss.interface"]
// pub static mut INTERFACE: [usize; 0x1000 / core::mem::size_of::<usize>()] = [0usize; 0x1000 / core::mem::size_of::<usize>()];

/// _start() 函数由内核跳转执行，只由内核执行一次，设置 printlib，如果不初始化，似乎会出现一些奇怪的问题
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> usize {
    printlib::init_console(&Console);
    printlib::set_log_level(option_env!("LOG"));
    init_proc as usize
}

/// 每个进程的初始化函数，主要是设置用户堆，在内核调度用户进程之前执行
fn init_proc(secondary_init: usize, heapptr: usize) -> usize{
    let heap = heapptr as *mut usize as *mut MutAllocator<32>;
    let exe = (heapptr + core::mem::size_of::<MutAllocator<32>>()) as *mut usize as *mut Executor;
    unsafe {
        heap::init(&mut *heap);
        executor::init(&mut *exe);
        SECONDARY_INIT = secondary_init;
    }
    primary_thread as usize
}

/// 初始化进程时，主线程的入口，在这个函数中初始化进程堆的 MEMORY，printlib
fn primary_thread() {
    log::warn!("main thread init ");
    unsafe {
        log::debug!("SECONDARY_ENTER {:#x}", SECONDARY_INIT);
        let secondary_init: fn(usize) -> usize = core::mem::transmute(SECONDARY_INIT);
        let second_thread_entry =  secondary_init(add_coroutine as usize);
        add_coroutine(Box::pin(test(second_thread_entry)), 0);
    }
    run();
}

async fn test(entry: usize) {
    unsafe {
        let secondary_thread: fn() -> usize = core::mem::transmute(entry);
        secondary_thread();
    }
}



struct Console;

impl printlib::Console for Console {
    #[inline]
    fn put_char(&self, c: u8) {
        syscall::write(0, &[c]);
    }

    #[inline]
    fn put_str(&self, s: &str) {
        syscall::write(0, s.as_bytes());
    }
}




