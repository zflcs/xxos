use core::any::Any;



/// Task trait
pub trait Task: Send + Sync {
    /// 执行任务
    fn execute(&self);
    /// 抽象成 Any
    fn as_any(&self) -> &dyn Any;
}



#[allow(unused)]
pub enum TaskResult {
    Syscall,        // 任务执行系统调用
    End,            // 任务结束
    Exception,      // 任务异常
    Pending,        // 任务暂停
}