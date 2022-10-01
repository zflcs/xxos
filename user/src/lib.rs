#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod heap;

extern crate alloc;

use core::alloc::Layout;

pub use printlib::{print, println};
pub use syscall::*;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> usize {
    printlib::init_console(&Console);
    printlib::set_log_level(option_env!("LOG"));
    heap::init();
    main as usize
    // exit(main());
    // unreachable!()
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
