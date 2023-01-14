mod riscv;

use core::alloc::Layout;
pub use riscv::*;

/// 在栈顶保留上下文的空间
#[naked]
pub unsafe extern "C" fn skip_context() {
    const LAYOUT: Layout = Layout::new::<FlowContext>();
    core::arch::asm!(
        "   addi sp, sp, {size}
            andi sp, sp, {mask}
            ret
        ",
        size = const -(LAYOUT.size() as isize),
        mask = const !(LAYOUT.align() as isize - 1) ,
        options(noreturn)
    )
}