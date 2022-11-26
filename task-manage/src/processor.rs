use alloc::{boxed::Box, sync::Arc};
use super::Task;
use super::Schedule;
use spin::Mutex;

/// Processor
pub struct Processor{
    // 任务对象调度器
    scheduler: Arc<Mutex<dyn Schedule<Item = Arc<Box<dyn Task>>>>>,
    // 当前正在运行的任务
    current: Option<Arc<Box<dyn Task>>>,
}

impl Processor {
    /// new
    pub const fn new(scheduler: Arc<Mutex<dyn Schedule<Item = Arc<Box<dyn Task>>>>>) -> Self {
        Self {
            scheduler,
            current: None
        }
    }

    /// 添加任务
    pub fn add_task(&mut self, task: Arc<Box<dyn Task>>) {
        self.scheduler.lock().add(task);
    }
    
    /// 取出任务
    pub fn next_task(&mut self) -> Option<Arc<Box<dyn Task>>> {
        let task = self.scheduler.lock().fetch();
        self.current = task.clone();
        task
    }

    /// 当前任务
    pub fn current_task(&self) -> Option<Arc<Box<dyn Task>>> {
        self.current.as_ref().map(Arc::clone)
    }
}
