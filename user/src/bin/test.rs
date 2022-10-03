#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::add_coroutine;
extern crate alloc;
use alloc::boxed::Box;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    add_coroutine(Box::pin(te()), 1);
    0
}

async fn te() {
    println!("Hello, world! add by coroutine");
}
