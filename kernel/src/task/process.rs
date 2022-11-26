use crate::mmimpl::{PAGE, Sv39Manager, from_elf, elf_entry, PAGE_SIZE};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::alloc::Layout;
use kernel_context::{foreign::ForeignContext, LocalContext};
use kernel_vm::{
    page_table::{MmuMeta, Sv39, VmFlags, PPN, VPN},
    AddressSpace,
};
use task_manage::Task;
use xmas_elf::ElfFile;
use spin::Mutex;
use crate::{PORTAL, PROTAL_TRANSIT};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct ProcId(usize);

impl ProcId {
    pub(crate) fn generate() -> ProcId {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        ProcId(id)
    }

    // pub fn from(v: usize) -> Self {
    //     Self(v)
    // }

    // pub fn get_val(&self) -> usize {
    //     self.0
    // }
}

/// 进程内部可变数据
pub struct ProcessInner {
    pub context: ForeignContext,
    pub address_space: AddressSpace<Sv39, Sv39Manager>,
}

/// 进程。
pub struct Process {
    /// 不可变
    pub pid: ProcId,
    /// 可变
    pub inner: Mutex<ProcessInner>,
}

impl Process {

    // 默认将共享的主线程代码链接进来，创建时需要从符号表查找堆的位置
    pub fn from_elf(elf: ElfFile) -> Option<Self> {
        // elf 入口地址
        let entry = elf_entry(&elf)?;
        // 根据 elf 生成地址空间
        let mut address_space = from_elf(&elf);
        // 分配两个页当作用户态栈
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
            inner: Mutex::new(
                ProcessInner {
                    context: ForeignContext { context, satp },
                    address_space,
                }
            )
        })
    }

}

// unsafe impl Sync for Process {}
// unsafe impl Send for Process {}

unsafe impl Sync for ProcessInner {}
unsafe impl Send for ProcessInner {}


impl Task for Process {
    fn execute(&self) {
        unsafe {
            self.inner.lock().context.execute(&mut PORTAL, PROTAL_TRANSIT);
        }
    }
}


