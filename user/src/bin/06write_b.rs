#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::sched_yield;

const WIDTH: usize = 10;
const HEIGHT: usize = 2;

#[no_mangle]
fn main() -> i32 {
    for i in 0..HEIGHT {
        for _ in 0..WIDTH {
            print!("B");
        }
        println!(" [{}/{}]", i + 1, HEIGHT);
        sched_yield();
    }
    println!("Test write_b OK!");
    0
}