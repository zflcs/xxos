//! 陷入模块
//! 

#![no_std]
#![deny(warnings, missing_docs)]
#![feature(naked_functions, asm_const)]

mod context;
mod fast;

mod stack;

pub use stack::Stack;
pub use context::{FlowContext, skip_context, trap_entry};


use riscv::register::{
    scause::{self, Exception, Trap},
    stval,
};

/// 内核处理中断异常函数
pub fn trap_handler() {
    let scause = scause::read();
    let stval = stval::read();
    //println!("into {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            log::info!("user syscall");
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
}