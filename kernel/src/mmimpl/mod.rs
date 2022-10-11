/// 物理内存管理
/// 

// 内核堆管理
mod kernel_heap;

// 物理页管理
mod page_manager;

// 内核地址空间
mod kernel_space;

pub use kernel_heap::heap_init;
pub use kernel_heap::heap_test;
pub use kernel_heap::PAGE;

pub use page_manager::Sv39Manager;

pub use kernel_space::KERNEL_SPACE;
pub use kernel_space::init_kern_space;
