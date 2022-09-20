
use super::TaskManager;

/// 处理器
pub struct Processor<T, I: Copy + Ord> {
    // 进程管理调度
    manager: TaskManager<T, I>,
    // 当前正在运行的进程 ID
    current: Option<I>,

}

impl <T, I: Copy + Ord> Processor<T, I> {
    pub fn new() -> Self {
        Self { 
            manager: TaskManager::new(), 
            current: None,
        }
    }

    pub fn run_next(&mut self) {
        let task = self.manager.fetch();
        
    }

    pub fn make_current_suspend(&mut self) {
        let id = self.current.unwrap();
        self.manager.add(id);
        self.current = None;
    }

    pub fn make_current_exited(&mut self) {
        self.current = None;
    }


}