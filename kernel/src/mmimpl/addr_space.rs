
use super::Sv39Manager;
use alloc::{collections::BTreeMap, string::String};
use kernel_vm::{
    page_table::{Sv39, VAddr, VmFlags, PPN, MmuMeta},
    AddressSpace,
};
use spin::Once;
use printlib::log;
use riscv::register::satp;
use xmas_elf::{
    program, ElfFile,
    header::{self, HeaderPt2, Machine},
};
use core::{str::FromStr, slice::SlicePattern};
use crate::fsimpl::{FS, read_all};
use easy_fs::{OpenFlags, FSManager};


// 内核地址空间
pub static mut KERNEL_SPACE: Once<AddressSpace<Sv39, Sv39Manager>> = Once::new();
// 共享模块地址空间，根据模块名得到对应的模块地址空间
pub static mut SHARE_MODULE_SPACE: BTreeMap<String, AddressSpace<Sv39, Sv39Manager>> = BTreeMap::new();

pub fn init_kern_space() {
    let layout = linker::KernelLayout::locate();
    unsafe { KERNEL_SPACE.call_once(|| kernel_space(layout)) };
}

/// 外设映射到内存中的区域
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

pub const PAGE_SIZE: usize = 1 << Sv39::PAGE_BITS;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/// 根据 elf 文件生成地址空间
pub fn from_elf(elf: &ElfFile) -> AddressSpace<Sv39, Sv39Manager> {
    let mut address_space = AddressSpace::new();
    for program in elf.program_iter() {
        if !matches!(program.get_type(), Ok(program::Type::Load)) {
            continue;
        }
        let off_file = program.offset() as usize;
        let len_file = program.file_size() as usize;
        let off_mem = program.virtual_addr() as usize;
        let end_mem = off_mem + program.mem_size() as usize;
        assert_eq!(off_file & PAGE_MASK, off_mem & PAGE_MASK);

        let mut flags: [u8; 5] = *b"U___V";
        if program.flags().is_execute() {
            flags[1] = b'X';
        }
        if program.flags().is_write() {
            flags[2] = b'W';
        }
        if program.flags().is_read() {
            flags[3] = b'R';
        }
        address_space.map(
            VAddr::new(off_mem).floor()..VAddr::new(end_mem).ceil(),
            &elf.input[off_file..][..len_file],
            off_mem & PAGE_MASK,
            VmFlags::from_str(unsafe { core::str::from_utf8_unchecked(&flags) }).unwrap(),
        );
    }
    address_space
}

/// 读取共享模块
pub fn load_module(name: &str) {
    // 根据模块的 elf 文件得到地址空间以及入口
    let elf = ElfFile::new(read_all(FS.open(name, OpenFlags::RDONLY).unwrap()).as_slice()).unwrap();
    let addr_space = from_elf(&elf);
    let entry = elf_entry(&elf).unwrap();
    unsafe {
        SHARE_MODULE_SPACE.insert(String::from(name), addr_space);
    }
    let module_init: fn() -> usize = core::mem::transmute(entry);
    PROC_INIT = module_init();
}

/// 读取 elf 文件的入口
pub fn elf_entry(elf: &ElfFile) -> Option<usize> {
    // elf 入口地址
    match elf.header.pt2 {
        HeaderPt2::Header64(pt2)
            if pt2.type_.as_type() == header::Type::Executable
                && pt2.machine.as_machine() == Machine::RISC_V =>
        {
            Some(pt2.entry_point as usize)
        }
        _ => None?,
    }   
}

pub fn addrspace_add_module(addrspace: AddressSpace<Sv39, Sv39Manager>, module: AddressSpace<Sv39, Sv39Manager>, is_user: bool) {
    let sections = module.sections;
    for (_, addrmap) in sections.iter().enumerate() {
        let vpn_range = addrmap.vpn_range;
        let ppn_base = addrmap.ppn_range.start;
        let mut permission = addrmap.permission;
        if is_user {
            // permission.
        }
        addrspace.map_extern(vpn_range, addrmap.ppn_range.start, addrmap.permission);
    }
}

