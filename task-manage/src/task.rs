

/// Task trait
pub trait Task: Send + Sync {
    /// 执行任务
    fn execute(&self);
}



#[allow(unused)]
pub enum TaskResult {
    Syscall,        // 任务执行系统调用
    End,            // 任务结束
    Exception,      // 任务异常
    Pending,        // 任务暂停
}