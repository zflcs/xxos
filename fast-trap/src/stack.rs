use core::ptr::NonNull;
use vmm::stack_alloc;
use config::STACK_SIZE;

use crate::FlowContext;


/// 栈
#[repr(C, align(4096))]
pub struct Stack(pub [u8; STACK_SIZE]);

impl Stack {
    /// 找到上下文
    pub fn context(&self) -> NonNull<FlowContext> {
        let ctx = self.0.as_ptr() as usize + STACK_SIZE - core::mem::size_of::<FlowContext>();

        unsafe { NonNull::new_unchecked(ctx as *mut usize as *mut FlowContext) }
    }
}

/// 分配栈
pub fn alloc_stack() -> Option<NonNull<Stack>> {
    stack_alloc().map(|p| unsafe { 
        NonNull::new_unchecked(p as *mut usize as *mut Stack)
    })
}
