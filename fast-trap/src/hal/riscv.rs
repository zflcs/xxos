/// 陷入上下文。
/// 保存在一个栈的顶部
/// 保存了陷入时的寄存器状态。包括所有通用寄存器和 `pc`。
#[repr(C)]
#[allow(missing_docs)]
pub struct FlowContext {
    pub sp: usize,      // 0，
    pub ra: usize,      // 1..
    pub t: [usize; 7],  // 2..
    pub a: [usize; 8],  // 9..
    pub s: [usize; 12], // 17..
    pub gp: usize,      // 29..
    pub tp: usize,      // 30..
    pub pc: usize,      // 31..
}

impl FlowContext {
    /// 零初始化。
    #[allow(unused)]
    pub const ZERO: Self = Self {
        ra: 0,
        t: [0; 7],
        a: [0; 8],
        s: [0; 12],
        gp: 0,
        tp: 0,
        sp: 0,
        pc: 0,
    };
}