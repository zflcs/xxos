#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod heap;

extern crate alloc;

use core::alloc::Layout;
use core::future::Future;
use alloc::boxed::Box;
use core::pin::Pin;
pub use printlib::{print, println};
pub use syscall::*;


static mut ADD_COROUTINE_PTR: usize = 0usize;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(add_coroutine_ptr: usize) -> usize {
    printlib::init_console(&Console);
    printlib::set_log_level(option_env!("LOG"));
    heap::init();
    unsafe{
        printlib::log::warn!("HEAP size {:#x}", core::mem::size_of::<heap::MutAllocator<32>>());
        printlib::log::warn!("EXECUTOR ptr {:#x}", &mut heap::EXECUTOR as *mut runtime::Executor as usize);
        ADD_COROUTINE_PTR = add_coroutine_ptr;
    }
    main as usize
    // exit(main());
    // unreachable!()
}

#[no_mangle]
#[inline(never)]
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    unsafe {
        let add_coroutine_true: fn(Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, usize) = 
            core::mem::transmute(ADD_COROUTINE_PTR);
        add_coroutine_true(future, prio);
    }
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message().unwrap();
    if let Some(location) = panic_info.location() {
        printlib::log::error!("Panicked at {}:{}, {err}", location.file(), location.line());
    } else {
        printlib::log::error!("Panicked: {err}");
    }
    exit(1);
    unreachable!()
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Failed to alloc {layout:?}")
}

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    read(1, &mut c);
    c[0]
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
