mod mapper;
mod visitor;

extern crate alloc;

use crate::PageManager;
use alloc::vec::Vec;
use core::{fmt, ops::Range, ptr::NonNull};
use mapper::Mapper;
use page_table::{PageTable, PageTableFormatter, Pos, VAddr, VmFlags, VmMeta, PPN, VPN};
use visitor::Visitor;



/// 地址空间。缺页则将对应的 PPN 改为 PPN::0
pub struct AddressSpace<Meta: VmMeta, M: PageManager<Meta>> {
    /// 段内存管理
    pub sections: Vec<AddrMap<Meta>>,
    page_manager: M,
    /// 异界传送门的属性
    pub tramps: Vec<AddrMap<Meta>>,
}

impl<Meta: VmMeta, M: PageManager<Meta>> AddressSpace<Meta, M> {
    /// 创建新地址空间。
    #[inline]
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            page_manager: M::new_root(),
            tramps: Vec::new(),
        }
    }

    /// 地址空间根页表的物理页号。
    #[inline]
    pub fn root_ppn(&self) -> PPN<Meta> {
        self.page_manager.root_ppn()
    }

    /// 地址空间根页表
    #[inline]
    pub fn root(&self) -> PageTable<Meta> {
        unsafe { PageTable::from_root(self.page_manager.root_ptr()) }
    }

    /// 向地址空间增加异界传送门映射关系。
    pub fn map_portal(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, permission: VmFlags<Meta>) {
        self.tramps.push(AddrMap::<Meta>::new(vpn, ppn, 1, permission));
        let mut root = self.root();
        let mut mapper = Mapper::new(self, ppn..ppn + 1, permission);
        root.walk_mut(Pos::new(vpn, 0), &mut mapper);
    }

    /// 向地址空间增加映射关系。
    pub fn map_extern(&mut self, range: Range<VPN<Meta>>, pbase: PPN<Meta>, permission: VmFlags<Meta>) {
        let count = range.end.val() - range.start.val();
        self.sections.push(AddrMap::<Meta>::new(range.start, pbase, count, permission));
        let mut root = self.root();
        let mut mapper = Mapper::new(self, pbase..pbase + count, permission);
        root.walk_mut(Pos::new(range.start, 0), &mut mapper);
        if !mapper.ans() {
            // 映射失败，需要回滚吗？
            todo!()
        }
    }

    /// 分配新的物理页，拷贝数据并建立映射。
    pub fn map(
        &mut self,
        range: Range<VPN<Meta>>,
        data: &[u8],
        offset: usize,
        mut permission: VmFlags<Meta>,
    ) {
        let count = range.end.val() - range.start.val();
        let size = count << Meta::PAGE_BITS;
        assert!(size >= data.len() + offset);
        let page = self.page_manager.allocate(count, &mut permission);
        unsafe {
            use core::slice::from_raw_parts_mut as slice;
            let mut ptr = page.as_ptr();
            slice(ptr, offset).fill(0);
            ptr = ptr.add(offset);
            slice(ptr, data.len()).copy_from_slice(data);
            ptr = ptr.add(data.len());
            slice(ptr, page.as_ptr().add(size).offset_from(ptr) as _).fill(0);
        }
        self.map_extern(range, self.page_manager.v_to_p(page), permission)
    }

    /// 检查 `flags` 的属性要求，然后将地址空间中的一个虚地址翻译成当前地址空间中的指针。
    pub fn translate<T>(&self, addr: VAddr<Meta>, flags: VmFlags<Meta>) -> Option<NonNull<T>> {
        let mut visitor = Visitor::new(self);
        self.root().walk(Pos::new(addr.floor(), 0), &mut visitor);
        visitor
            .ans()
            .filter(|pte| pte.flags().contains(flags))
            .map(|pte| unsafe {
                NonNull::new_unchecked(
                    self.page_manager
                        .p_to_v::<u8>(pte.ppn())
                        .as_ptr()
                        .add(addr.offset())
                        .cast(),
                )
            })
    }

    /// 遍历地址空间，将其中的地址映射添加进自己的地址空间中，重新分配物理页并拷贝所有数据及代码
    pub fn cloneself(&self, new_addrspace: &mut AddressSpace<Meta, M>) {
        let sections= &self.sections;
        for (_, addr_map) in sections.iter().enumerate() {
            // 段的范围
            let vpn_range = &addr_map.vpn_range;
            // 段的页面数量
            let count = vpn_range.end.val() - vpn_range.start.val();
            // 段的大小
            let size = count << Meta::PAGE_BITS;
            // 段的属性
            let mut permission = addr_map.permission;
            let data_ptr = (addr_map.ppn_range.start.val() << Meta::PAGE_BITS) as *mut u8;
            
            // 分配 count 个 flags 属性的物理页面
            let paddr = new_addrspace.page_manager.allocate(count, &mut permission);
            let ppn = new_addrspace.page_manager.v_to_p(paddr);
            unsafe {
                use core::slice::from_raw_parts_mut as slice;
                let data = slice(data_ptr, size);
                let ptr = paddr.as_ptr();
                slice(ptr, size).copy_from_slice(data);
            }
            new_addrspace.map_extern(vpn_range.start..vpn_range.end, ppn, permission);
        }
        let tramps = &self.tramps;
        for (_, addr_map) in tramps.iter().enumerate() {
            new_addrspace.map_portal(addr_map.vpn_range.start, addr_map.ppn_range.start, addr_map.permission);
        }
    }

}

impl<Meta: VmMeta, P: PageManager<Meta>> fmt::Debug for AddressSpace<Meta, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "root: {:#x}", self.root_ppn().val())?;
        write!(
            f,
            "{:?}",
            PageTableFormatter {
                pt: self.root(),
                f: |ppn| self.page_manager.p_to_v(ppn)
            }
        )
    }
}


/// 段内存管理
pub struct AddrMap<Meta: VmMeta> {
    /// 段地址范围
    pub vpn_range: Range<VPN<Meta>>,
    pub ppn_range: Range<PPN<Meta>>,
    /// 段属性
    pub permission: VmFlags<Meta>,
}

impl<Meta: VmMeta> AddrMap<Meta> {
    pub fn new(start_vpn: VPN<Meta>, start_ppn: PPN<Meta>, count: usize, permission: VmFlags<Meta>) -> Self {
        Self { 
            vpn_range: start_vpn..start_vpn + count, 
            ppn_range: start_ppn..start_ppn + count,
            permission, 
        }
    }
}
