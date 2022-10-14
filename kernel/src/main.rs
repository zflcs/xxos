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
mod config;

#[macro_use]
extern crate printlib;

#[macro_use]
extern crate alloc;

use crate::{
    fsimpl::{read_all, FS},
    task::process::Process, consoleimpl::init_console,
    mmimpl::{KERNEL_SPACE, activate_space},
};
use kernel_vm::page_table::{VmFlags, Sv39, PPN, MmuMeta, VPN};
use printlib::log;
use easy_fs::{FSManager, OpenFlags};

use processorimpl::{init_processor, processor, PROCESSORS};
use riscv::register::*;
use sbi_rt::*;
use xmas_elf::ElfFile;
use config::MAX_HART;
/// Supervisor 汇编入口。
///
/// 设置栈并跳转到 Rust。
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start(hartid: usize, opaque: usize) -> ! {
    // todo：目前假设是只有 4 个核启动，之后需要动态的实现
    const STACK_SIZE_PER_HART: usize = 32 * 4096;
    const TOTAL_STACK_SIZE: usize = STACK_SIZE_PER_HART * config::MAX_HART;

    #[link_section = ".bss.uninit"]
    static mut STACK: [u8; TOTAL_STACK_SIZE] = [0u8; TOTAL_STACK_SIZE];

    core::arch::asm!(
        "   mv tp, a0",                        // 将 a0 寄存器中保存的 cpuid 用 tp 寄存器保存
        "   la sp, {stack}",                   // 将 sp 指向整个栈的最下方
        "   li t0, {stack_size_per_hart}",     // t0 保存每个 cpu 使用的栈大小
        "   addi t1, a0, 1",                   // t1 = hartid + 1

        "1: add sp, sp, t0",                   // 循环加法，sp 指向正确的栈顶
        "   addi t1, t1, -1",                  // 
        "   bnez t1, 1b",                      //

        "   beq a1, x0, 2f",                   // 根据 a1 寄存器中的参数，进入副 cpu 初始化入口，副 cpu 进入内核时，a1 = opaque = 0
        "   j  {primary_main}",
        "2: j {secondary_main}",
        stack_size_per_hart = const STACK_SIZE_PER_HART,
        stack               = sym STACK,
        primary_main        = sym primary_main,
        secondary_main      = sym secondary_main,
        options(noreturn),
    )
}

/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
    unreachable!()
}

pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) hart_id, options(nomem, nostack));
    }
    hart_id
}

extern "C" fn primary_main() -> ! {
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
    for i in 0..MAX_HART {
        unsafe{ KERNEL_SPACE.get_mut().unwrap().map_portal(
            VPN::<Sv39>::new( PROCESSORS[i].portal_vpn ),
            PPN::<Sv39>::new( &PROCESSORS[i].portal as *const _ as usize >> Sv39::PAGE_BITS),
            VmFlags::build_from_str("XWRV"),
        ) }
    }
    // log::debug!("{:?}", unsafe { KERNEL_SPACE.get().unwrap() });

    // 初始化完毕，通过 hsm 启动副 cpu
    for i in 0..config::MAX_HART{
        if i != hart_id() {
            sbi_rt::hart_start(i, _start as usize, 0);
        }
    }
    // 加载应用程序
    // TODO!
    println!("/**** APPS ****");
    for app in FS.readdir("").unwrap() {
        println!("{}", app);
    }
    println!("**************/");
    {
        let initproc = read_all(FS.open("initproc", OpenFlags::RDONLY).unwrap());
        if let Some(process) = Process::from_elf(ElfFile::new(initproc.as_slice()).unwrap()) {
            processor().add(process.pid, process);
        }
    }
    run_task();
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    unreachable!()
}

fn secondary_main() {
    // 初始化 satp 寄存器
    unsafe{ activate_space(KERNEL_SPACE.get_mut().unwrap()); }
    run_task();
    // sbi_rt::hart_stop();
}

fn run_task() {
    loop {
        if let Some(task) = processor().find_next() {
            task.execute(&mut processor().portal, processor().portal_transit);
            match scause::read().cause() {
                scause::Trap::Exception(scause::Exception::UserEnvCall) => {
                    use syscall::{SyscallId as Id, SyscallResult as Ret};
                    let ctx = &mut task.context.context;
                    ctx.move_next();
                    let id: Id = ctx.a(7).into();
                    let args = [ctx.a(0), ctx.a(1), ctx.a(2), ctx.a(3), ctx.a(4), ctx.a(5)];
                    match syscall::handle(id, args) {
                        Ret::Done(ret) => match id {
                            Id::EXIT => processor().make_current_exited(),
                            _ => {
                                let ctx = &mut task.context.context;
                                *ctx.a_mut(0) = ret as _;
                                processor().make_current_suspend();
                            }
                        },
                        Ret::Unsupported(_) => {
                            log::info!("id = {id:?}");
                            processor().make_current_exited();
                        }
                    }
                }
                e => {
                    log::error!("unsupported trap: {e:?}");
                    processor().make_current_exited();
                }
            }
        } else {
            println!("no task");
            break;
        }
    }
}



