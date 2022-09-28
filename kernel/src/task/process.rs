﻿use crate::{heap_alloc::PAGE, Sv39Manager};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{alloc::Layout, str::FromStr};
use easy_fs::FileHandle;
use kernel_context::{foreign::ForeignContext, foreign::ForeignPortal, LocalContext};
use kernel_vm::{
    page_table::{MmuMeta, Sv39, VAddr, VmFlags, PPN, VPN},
    AddressSpace,
};
use spin::Mutex;
use xmas_elf::{
    header::{self, HeaderPt2, Machine},
    program, ElfFile,
};
use super::{thread::Thread};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct ProcId(usize);

impl ProcId {
    pub(crate) fn generate() -> ProcId {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        ProcId(id)
    }

    pub fn from(v: usize) -> Self {
        Self(v)
    }

    pub fn get_val(&self) -> usize {
        self.0
    }
}

/// 进程。
pub struct Process {
    /// 不可变
    pub pid: ProcId,
    /// 可变
    pub parent: ProcId,
    pub children: Vec<ProcId>,
    pub context: ForeignContext,
    pub address_space: AddressSpace<Sv39, Sv39Manager>,

    // 文件描述符表
    pub fd_table: Vec<Option<Mutex<FileHandle>>>,

    // 线程
    pub threads: Vec<Thread>,
}

impl Process {
    pub fn exec(&mut self, elf: ElfFile) {
        let proc = Process::from_elf(elf).unwrap();
        let tramp = self.address_space.tramp;
        self.address_space = proc.address_space;
        self.address_space.map_portal(tramp);
        self.context = proc.context;
    }

    pub fn fork(&mut self) -> Option<Process> {
        // 子进程 pid
        let pid = ProcId::generate();
        // 复制父进程地址空间
        let parent_addr_space = &self.address_space;
        let mut address_space: AddressSpace<Sv39, Sv39Manager> = AddressSpace::new();
        parent_addr_space.cloneself(&mut address_space);
        // 复制父进程上下文
        let context = self.context.context.clone();
        let satp = (8 << 60) | address_space.root_ppn().val();
        let foreign_ctx = ForeignContext { context, satp };
        self.children.push(pid);
        // 复制父进程文件符描述表
        let mut new_fd_table: Vec<Option<Mutex<FileHandle>>> = Vec::new();
        for fd in self.fd_table.iter_mut() {
            if let Some(file) = fd {
                new_fd_table.push(Some(Mutex::new(file.get_mut().clone())));
            } else {
                new_fd_table.push(None);
            }
        }
        Some(Self {
            pid,
            parent: self.pid,
            children: Vec::new(),
            context: foreign_ctx,
            address_space,
            fd_table: new_fd_table,
            threads: Vec::new(),
        })
    }

    pub fn from_elf(elf: ElfFile) -> Option<Self> {
        let entry = match elf.header.pt2 {
            HeaderPt2::Header64(pt2)
                if pt2.type_.as_type() == header::Type::Executable
                    && pt2.machine.as_machine() == Machine::RISC_V =>
            {
                pt2.entry_point as usize
            }
            _ => None?,
        };

        const PAGE_SIZE: usize = 1 << Sv39::PAGE_BITS;
        const PAGE_MASK: usize = PAGE_SIZE - 1;

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
        unsafe {
            let (pages, size) = PAGE
                .allocate_layout::<u8>(Layout::from_size_align_unchecked(2 * PAGE_SIZE, PAGE_SIZE))
                .unwrap();
            assert_eq!(size, 2 * PAGE_SIZE);
            core::slice::from_raw_parts_mut(pages.as_ptr(), 2 * PAGE_SIZE).fill(0);
            address_space.map_extern(
                VPN::new((1 << 26) - 2)..VPN::new(1 << 26),
                PPN::new(pages.as_ptr() as usize >> Sv39::PAGE_BITS),
                VmFlags::build_from_str("U_WRV"),
            );
        }
        let mut context = LocalContext::user(entry);
        let satp = (8 << 60) | address_space.root_ppn().val();
        *context.sp_mut() = 1 << 38;
        Some(Self {
            pid: ProcId::generate(),
            parent: ProcId(usize::MAX),
            children: Vec::new(),
            context: ForeignContext { context, satp },
            address_space,
            fd_table: vec![
                // Stdin
                Some(Mutex::new(FileHandle::empty(true, false))),
                // Stdout
                Some(Mutex::new(FileHandle::empty(false, true))),
            ],
            threads: Vec::new(),

        })
    }

    pub fn execute(&mut self, portal: &mut ForeignPortal, portal_transit: usize) {
        unsafe { self.context.execute(portal, portal_transit) };
    }
}
