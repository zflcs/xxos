use alloc::{collections::VecDeque, boxed::Box};
use task_manage::{Schedule, Processor, Task};
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROCESSOR: Arc<Mutex<Processor>> = 
        Arc::new(Mutex::new(Processor::new(Arc::new(Mutex::new(Scheduler::new())))));
}

/// 任务调度器
pub struct Scheduler {
    ready_queue: VecDeque<Arc<Box<dyn Task>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { ready_queue: VecDeque::new() }
    }
}

impl Schedule for Scheduler {
    type Item = Arc<Box<dyn Task>>;
    fn add(&mut self, task: Self::Item) {
        self.ready_queue.push_back(task)
    }

    fn fetch(&mut self) -> Option<Self::Item> {
        self.ready_queue.pop_front()
    }
}

