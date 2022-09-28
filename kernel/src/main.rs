#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const, const_btree_new)]
#![feature(default_alloc_error_handler)]
#![deny(warnings)]

mod fsimpl;
mod heap_alloc;
mod processorimpl;
mod drivers;
mod consoleimpl;
mod syscallimpl;
mod task;

#[macro_use]
extern crate printlib;

#[macro_use]
extern crate alloc;

use crate::{
    fsimpl::{read_all, FS},
    impls::{Sv39Manager},
    task::process::Process, consoleimpl::init_console,
};
use printlib::log;
use easy_fs::{FSManager, OpenFlags};
use kernel_vm::{
    page_table::{MmuMeta, Sv39, VAddr, VmFlags, PPN, VPN},
    AddressSpace,
};
use processorimpl::{init_processor, PROCESSOR};
use riscv::register::*;
use sbi_rt::*;
use spin::Once;
use xmas_elf::ElfFile;
use syscall::Caller;


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

static mut KERNEL_SPACE: Once<AddressSpace<Sv39, Sv39Manager>> = Once::new();

extern "C" fn rust_main() -> ! {
    let layout = linker::KernelLayout::locate();
    // bss 段清零
    unsafe { layout.zero_bss() };
    // 初始化 `console`
    init_console();

    // 初始化 syscall
    syscallimpl::init_syscall();
    // 初始化内核堆
    heap_alloc::init();
    heap_alloc::test();
    // 建立内核地址空间
    unsafe { KERNEL_SPACE.call_once(|| kernel_space(layout)) };
    // 异界传送门
    // 可以直接放在栈上
    init_processor();
    let tramp = (
        PPN::<Sv39>::new(unsafe { &PROCESSOR.portal } as *const _ as usize >> Sv39::PAGE_BITS),
        VmFlags::build_from_str("XWRV"),
    );
    // 传送门映射到所有地址空间
    unsafe { KERNEL_SPACE.get_mut().unwrap().map_portal(tramp) };
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
            process.address_space.map_portal(tramp);
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
                    match syscall::handle(Caller { entity: 0, flow: 0 }, id, args) {
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

pub const MMIO: &[(usize, usize)] = &[
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];

fn kernel_space(layout: linker::KernelLayout) -> AddressSpace<Sv39, Sv39Manager> {
    // 打印段位置
    let text = VAddr::<Sv39>::new(layout.text);
    let rodata = VAddr::<Sv39>::new(layout.rodata);
    let data = VAddr::<Sv39>::new(layout.data);
    let end = VAddr::<Sv39>::new(layout.end);
    log::info!("__text ----> {:#10x}", text.val());
    log::info!("__rodata --> {:#10x}", rodata.val());
    log::info!("__data ----> {:#10x}", data.val());
    log::info!("__end -----> {:#10x}", end.val());
    println!();

    // 内核地址空间
    let mut space = AddressSpace::<Sv39, Sv39Manager>::new();
    space.map_extern(
        text.floor()..rodata.ceil(),
        PPN::new(text.floor().val()),
        VmFlags::build_from_str("X_RV"),
    );
    space.map_extern(
        rodata.floor()..data.ceil(),
        PPN::new(rodata.floor().val()),
        VmFlags::build_from_str("__RV"),
    );
    space.map_extern(
        data.floor()..end.ceil(),
        PPN::new(data.floor().val()),
        VmFlags::build_from_str("_WRV"),
    );

    // MMIO
    for pair in MMIO {
        let _mmio_begin = VAddr::<Sv39>::new(pair.0);
        let _mmio_end = VAddr::<Sv39>::new(pair.0 + pair.1);
        log::info!(
            "MMIO range ---> {:#10x}, {:#10x} \n",
            _mmio_begin.val(),
            _mmio_end.val()
        );
        space.map_extern(
            _mmio_begin.floor().._mmio_end.ceil(),
            PPN::new(_mmio_begin.floor().val()),
            VmFlags::build_from_str("_WRV"),
        );
    }

    unsafe { satp::set(satp::Mode::Sv39, 0, space.root_ppn().val()) };
    space
}

/// 各种接口库的实现。
mod impls {
    use crate::{
        heap_alloc::PAGE,
    };
    use alloc::{alloc::handle_alloc_error};
    use core::{alloc::Layout, num::NonZeroUsize, ptr::NonNull};
    use kernel_vm::{
        page_table::{MmuMeta, Pte, Sv39, VAddr, VmFlags, PPN, VPN},
        PageManager,
    };

    #[repr(transparent)]
    pub struct Sv39Manager(NonNull<Pte<Sv39>>);

    impl Sv39Manager {
        const OWNED: VmFlags<Sv39> = unsafe { VmFlags::from_raw(1 << 8) };
    }

    impl PageManager<Sv39> for Sv39Manager {
        #[inline]
        fn new_root() -> Self {
            const SIZE: usize = 1 << Sv39::PAGE_BITS;
            unsafe {
                match PAGE.allocate(Sv39::PAGE_BITS, NonZeroUsize::new_unchecked(SIZE)) {
                    Ok((ptr, _)) => Self(ptr),
                    Err(_) => handle_alloc_error(Layout::from_size_align_unchecked(SIZE, SIZE)),
                }
            }
        }

        #[inline]
        fn root_ppn(&self) -> PPN<Sv39> {
            PPN::new(self.0.as_ptr() as usize >> Sv39::PAGE_BITS)
        }

        #[inline]
        fn root_ptr(&self) -> NonNull<Pte<Sv39>> {
            self.0
        }

        #[inline]
        fn p_to_v<T>(&self, ppn: PPN<Sv39>) -> NonNull<T> {
            unsafe { NonNull::new_unchecked(VPN::<Sv39>::new(ppn.val()).base().as_mut_ptr()) }
        }

        #[inline]
        fn v_to_p<T>(&self, ptr: NonNull<T>) -> PPN<Sv39> {
            PPN::new(VAddr::<Sv39>::new(ptr.as_ptr() as _).floor().val())
        }

        #[inline]
        fn check_owned(&self, pte: Pte<Sv39>) -> bool {
            pte.flags().contains(Self::OWNED)
        }

        fn allocate(&mut self, len: usize, flags: &mut VmFlags<Sv39>) -> NonNull<u8> {
            unsafe {
                match PAGE.allocate(
                    Sv39::PAGE_BITS,
                    NonZeroUsize::new_unchecked(len << Sv39::PAGE_BITS),
                ) {
                    Ok((ptr, size)) => {
                        assert_eq!(size, len << Sv39::PAGE_BITS);
                        *flags |= Self::OWNED;
                        ptr
                    }
                    Err(_) => handle_alloc_error(Layout::from_size_align_unchecked(
                        len << Sv39::PAGE_BITS,
                        1 << Sv39::PAGE_BITS,
                    )),
                }
            }
        }

        fn deallocate(&mut self, _pte: Pte<Sv39>, _len: usize) -> usize {
            todo!()
        }

        fn drop_root(&mut self) {
            todo!()
        }
    }



    
}
