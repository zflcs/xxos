﻿use super::{trap_entry, FlowContext};
use core::arch::asm;

macro_rules! exchange {
    () => {
        "csrrw sp, mscratch, sp"
    };
}

macro_rules! r#return {
    () => {
        "mret"
    };
}

pub(super) use {exchange, r#return};

impl FlowContext {
    /// 从上下文向硬件加载非调用规范约定的寄存器。
    #[inline]
    pub(crate) unsafe fn load_others(&self) {
        asm!(
            "   mv         gp, {gp}
                mv         tp, {tp}
                csrw mscratch, {sp}
                csrw     mepc, {pc}
            ",
            gp = in(reg) self.gp,
            tp = in(reg) self.tp,
            sp = in(reg) self.sp,
            pc = in(reg) self.pc,
        );
    }
}

/// 交换突发寄存器。
#[inline]
pub(crate) fn exchange_scratch(mut val: usize) -> usize {
    unsafe { asm!("csrrw {0}, mscratch, {0}", inlateout(reg) val) };
    val
}

/// 模拟一个 `cause` 类的陷入。
///
/// # Safety
///
/// 如同发生一个陷入。
#[inline]
pub unsafe fn soft_trap(cause: usize) {
    asm!(
        "   la   {0},    1f
            csrw mepc,   {0}
            csrw mcause, {cause}
            j    {trap}
         1:
        ",
        out(reg) _,
        cause = in(reg) cause,
        trap  = sym trap_entry,
    );
}

/// 设置全局陷入入口。
///
/// # Safety
///
/// 这个函数操作硬件寄存器，寄存器里原本的值将丢弃。
#[inline]
pub unsafe fn load_direct_trap_entry() {
    asm!("csrw mtvec, {0}", in(reg) trap_entry, options(nomem))
}
