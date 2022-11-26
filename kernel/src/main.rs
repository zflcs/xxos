#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
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
use alloc::{sync::Arc, boxed::Box};
use kernel_vm::page_table::{VmFlags, Sv39, PPN, MmuMeta, VPN};
// use printlib::log;
use easy_fs::{FSManager, OpenFlags};

use processorimpl::PROCESSOR;
// use riscv::register::*;
use sbi_rt::*;
use xmas_elf::ElfFile;
use kernel_context::foreign::ForeignPortal;
use task_manage::Task;


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

static mut PORTAL: ForeignPortal = ForeignPortal::EMPTY;
const PROTAL_TRANSIT: usize = VPN::<Sv39>::MAX.base().val();


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
    mmimpl::init_kern_space();

    
    // 建立内核地址空间
    mmimpl::init_kern_space();
    // 异界传送门
    unsafe { PORTAL = ForeignPortal::new(); }
    // 传送门映射到内核地址空间
    unsafe { mmimpl::KERNEL_SPACE.get_mut().unwrap().map_portal(
        VPN::MAX, 
        PPN::<Sv39>::new( &PORTAL as *const _ as usize >> Sv39::PAGE_BITS),
        VmFlags::build_from_str("XWRV"),
    )};
    // 加载应用程序
    println!("/**** APPS ****");
    for app in FS.readdir("").unwrap() {
        let elf = read_all(FS.open(&app, OpenFlags::RDONLY).unwrap());
        if let Some(process) = Process::from_elf(ElfFile::new(&elf.as_slice()).unwrap()) {
            // 添加异界传送门映射            
            unsafe { 
                process.inner.lock().address_space.map_portal(
                    VPN::MAX,
                    PPN::<Sv39>::new(&PORTAL as *const _ as usize >> Sv39::PAGE_BITS),
                    VmFlags::build_from_str("XWRV"),
                );
                PROCESSOR.lock().add_task(Arc::new(Box::new(process) as Box<dyn Task>));
            };
        }
        println!("{}", app);
    }
    println!("**************/");
    // const PROTAL_TRANSIT: usize = VPN::<Sv39>::MAX.base().val();
    // loop {
    //     if let Some(task) = unsafe { PROCESSOR.get_mut().unwrap().next_task() } {
    //         match task {
    //             Arc<Task::Proc(process) => {
    //                 process.execute();
    //                 match scause::read().cause() {
    //                     scause::Trap::Exception(scause::Exception::UserEnvCall) => {
    //                         use syscall::{SyscallId as Id, SyscallResult as Ret};
    //                         let ctx = &mut process.inner.lock().context.context;
    //                         ctx.move_next();
    //                         let id: Id = ctx.a(7).into();
    //                         let args = [ctx.a(0), ctx.a(1), ctx.a(2), ctx.a(3), ctx.a(4), ctx.a(5)];
    //                         match syscall::handle(id, args) {
    //                             Ret::Done(ret) => match id {
    //                                 Id::EXIT => continue,
    //                                 _ => {
    //                                     let ctx = &mut process.inner.lock().context.context;
    //                                     *ctx.a_mut(0) = ret as _;
    //                                     unsafe { PROCESSOR.get_mut().unwrap().add_task(task) };
    //                                 }
    //                             },
    //                             Ret::Unsupported(_) => {
    //                                 log::info!("id = {id:?}");
    //                             }
    //                         }
    //                     }
    //                     e => {
    //                         log::error!("unsupported trap: {e:?} stval = {:#x}", stval::read());
    //                         log::error!("sepc = {:#x}", sepc::read());
    //                     }
    //                 }
    //             },
    //             _ => continue,

    //         }
            
    //     } else {
    //         println!("no task");
    //         break;
    //     }
    // }

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

