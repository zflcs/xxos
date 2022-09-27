/// 页表的内存管理

use super::heap::PAGE;
use alloc::alloc::handle_alloc_error;

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