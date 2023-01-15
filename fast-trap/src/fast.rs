
/// 快速路径处理结果。
#[allow(unused)]
#[repr(usize)]
pub enum FastResult {
    /// 处理完成，直接返回
    Restore = 0,
    /// 完整陷入，处理异常继续返回用户态或者进程切换
    Continue = 1,
    /// 恶意操作，进入内核直接杀死进程
    Kill = 2,
}