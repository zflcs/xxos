#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::alloc::*;


#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    0
}