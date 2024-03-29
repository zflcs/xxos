use core::alloc::Layout;

/// 上下文
#[repr(C)]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct FlowContext {
    pub satp: usize,
    pub t: [usize; 7],  // 0..
    pub a: [usize; 8],  // 7..
    pub s: [usize; 12], // 15..
    pub gp: usize,      // 27..
    pub tp: usize,      // 28..
    pub pc: usize,      // 29..
    pub ra: usize,      // 30..
    pub sp: usize,      // 31，
}

/// 在栈顶上保留出上下文的空间
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

/// 陷入处理函数
#[naked]
#[link_section = ".text.trampoline"]
pub unsafe extern "C" fn trap_entry() {
    core::arch::asm!(
        ".align 2",
        // 保存进程上下文
        "
            sd sp, -1*8(x0)
            sd ra, -2*8(x0) 
            sd tp, -4*8(x0)
            sd gp, -5*8(x0)
            sd t0, -32*8(x0)
            sd t1, -31*8(x0)
            sd t2, -30*8(x0)
            sd t3, -29*8(x0)
            sd t4, -28*8(x0)
            sd t5, -27*8(x0)
            sd t6, -26*8(x0)
            sd a0, -25*8(x0)
            sd a1, -24*8(x0)
            sd a2, -23*8(x0)
            sd a3, -22*8(x0)
            sd a4, -21*8(x0)
            sd a5, -20*8(x0)
            sd a6, -19*8(x0)
            sd a7, -18*8(x0)
            sd s0, -17*8(x0)
            sd s1, -16*8(x0)
            sd s2, -15*8(x0)
            sd s3, -14*8(x0)
            sd s4, -13*8(x0)
            sd s5, -12*8(x0)
            sd s6, -11*8(x0)
            sd s7, -10*8(x0)
            sd s8, -9*8(x0)
            sd s9, -8*8(x0)
            sd s10, -7*8(x0)
            sd s11, -6*8(x0)
            csrr t1 , sepc
            sd t1, -3*8(x0)
            csrr t0, satp
            sd t0, -33*8(x0)
        ",
        // 切换地址空间
        "
            csrr t0, sscratch
            csrw satp, t0
            sfence.vma
        ",
        // 恢复内核上下文
        "
            ld sp, -1*8(x0)
            ld ra, -2*8(x0) 
            ret
        ",
        options(noreturn)
    )
}

/// 内核切换到用户程序地址空间，默认 a0 寄存器中保存着目标进程的地址空间
/// 先切换地址空间，之后直接从高位虚拟地址恢复上下文
#[naked]
#[link_section = ".text.trampoline"]
pub unsafe extern "C" fn restore() {
    core::arch::asm!(
        ".align 2",
        // 不需要保存内核的上下文，直接切换地址空间
        "
            csrw satp, a0
            sfence.vma
        ",
        "
            ld t1, -3*8(x0)
            csrw sepc, t1
            ld sp, -1*8(x0)
            ld ra, -2*8(x0) 
            ld tp, -4*8(x0)
            ld gp, -5*8(x0)
            ld t0, -32*8(x0)
            ld t1, -31*8(x0)
            ld t2, -30*8(x0)
            ld t3, -29*8(x0)
            ld t4, -28*8(x0)
            ld t5, -27*8(x0)
            ld t6, -26*8(x0)
            ld a0, -25*8(x0)
            ld a1, -24*8(x0)
            ld a2, -23*8(x0)
            ld a3, -22*8(x0)
            ld a4, -21*8(x0)
            ld a5, -20*8(x0)
            ld a6, -19*8(x0)
            ld a7, -18*8(x0)
            ld s0, -17*8(x0)
            ld s1, -16*8(x0)
            ld s2, -15*8(x0)
            ld s3, -14*8(x0)
            ld s4, -13*8(x0)
            ld s5, -12*8(x0)
            ld s6, -11*8(x0)
            ld s7, -10*8(x0)
            ld s8, -9*8(x0)
            ld s9, -8*8(x0)
            ld s10, -7*8(x0)
            ld s11, -6*8(x0)
            sret
        ",
        options(noreturn)
    )
}