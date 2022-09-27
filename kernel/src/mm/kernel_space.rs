
use printlib::*;
use kernel_vm::{
    page_table::{Sv39, VAddr, VmFlags, PPN},
    AddressSpace,
};
use super::pagemanage::Sv39Manager;
use riscv::register::*;


pub fn kernel_space(layout: linker::KernelLayout) -> AddressSpace<Sv39, Sv39Manager> {
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
    unsafe { satp::set(satp::Mode::Sv39, 0, space.root_ppn().val()) };
    space
}

