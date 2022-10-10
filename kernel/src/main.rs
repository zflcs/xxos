#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![feature(default_alloc_error_handler)]
#![deny(warnings)]

mod fsimpl;
mod processorimpl;
mod drivers;
mod consoleimpl;
mod syscallimpl;
mod task;
mod mmimpl;

#[macro_use]
extern crate printlib;

#[macro_use]
extern crate alloc;

use crate::{
    fsimpl::{read_all, FS},
    task::process::Process, consoleimpl::init_console,
};
use kernel_vm::page_table::{VmFlags, Sv39, PPN, MmuMeta, VPN};
use printlib::log;
use easy_fs::{FSManager, OpenFlags};

use processorimpl::{init_processor, PROCESSOR};
use riscv::register::*;
use sbi_rt::*;
use xmas_elf::ElfFile;

/// Supervisor 汇编入口。
///
/// 设置栈并跳转到 Rust。
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 16 * 4096;

    #[link_section = ".bss.uninit"]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    core::arch::asm!(
        "la sp, {stack} + {stack_size}",
        "j  {main}",
        stack_size = const STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
        options(noreturn),
    )
}


extern "C" fn rust_main() -> ! {
    let layout = linker::KernelLayout::locate();
    // bss 段清零
    unsafe { layout.zero_bss() };
    // 初始化 `console`
    init_console();
    // 初始化 syscall
    syscallimpl::init_syscall();
    // 初始化内核堆
    mmimpl::heap_init();
    mmimpl::heap_test();

    // 初始化处理器，同时设置好异界传送门
    init_processor();
    
    // 建立内核地址空间
    mmimpl::init_kern_space();
    
    // 传送门映射到内核地址空间
    unsafe { mmimpl::KERNEL_SPACE.get_mut().unwrap().map_portal(
        VPN::MAX, 
        PPN::<Sv39>::new( &PROCESSOR.portal as *const _ as usize >> Sv39::PAGE_BITS),
        VmFlags::build_from_str("XWRV"),
    )};
    // 加载应用程序
    // TODO!
    println!("/**** APPS ****");
    for app in FS.readdir("").unwrap() {
        println!("{}", app);
    }
    println!("**************/");
    {
        let initproc = read_all(FS.open("initproc", OpenFlags::RDONLY).unwrap());
        if let Some(mut process) = Process::from_elf(ElfFile::new(initproc.as_slice()).unwrap()) {
            process.address_space.map_portal(
                VPN::MAX, 
                PPN::<Sv39>::new(unsafe { &PROCESSOR.portal } as *const _ as usize >> Sv39::PAGE_BITS),
                VmFlags::build_from_str("XWRV"),
            );
            unsafe { PROCESSOR.add(process.pid, process) };
        }
    }

    const PROTAL_TRANSIT: usize = VPN::<Sv39>::MAX.base().val();
    loop {
        if let Some(task) = unsafe { PROCESSOR.find_next() } {
            task.execute(unsafe { &mut PROCESSOR.portal }, PROTAL_TRANSIT);
            match scause::read().cause() {
                scause::Trap::Exception(scause::Exception::UserEnvCall) => {
                    use syscall::{SyscallId as Id, SyscallResult as Ret};
                    let ctx = &mut task.context.context;
                    ctx.move_next();
                    let id: Id = ctx.a(7).into();
                    let args = [ctx.a(0), ctx.a(1), ctx.a(2), ctx.a(3), ctx.a(4), ctx.a(5)];
                    match syscall::handle(id, args) {
                        Ret::Done(ret) => match id {
                            Id::EXIT => unsafe { PROCESSOR.make_current_exited() },
                            _ => {
                                let ctx = &mut task.context.context;
                                *ctx.a_mut(0) = ret as _;
                                unsafe { PROCESSOR.make_current_suspend() };
                            }
                        },
                        Ret::Unsupported(_) => {
                            log::info!("id = {id:?}");
                            unsafe { PROCESSOR.make_current_exited() };
                        }
                    }
                }
                e => {
                    log::error!("unsupported trap: {e:?}");
                    unsafe { PROCESSOR.make_current_exited() };
                }
            }
        } else {
            println!("no task");
            break;
        }
    }

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

