use crate::{task::process::{Process, TaskId}, hart_id};
use alloc::collections::{BTreeMap, VecDeque};
use task_manage::{Manage, Processor};
use alloc::sync::Arc;
use spin::Mutex;
use crate::{config::MAX_HART};


const PROCESSOR: Processor<Process, TaskId, ProcManager> = Processor::new();

pub static mut PROCESSORS: [Processor<Process, TaskId, ProcManager>; MAX_HART] = [PROCESSOR; MAX_HART];
pub fn init_processor() {
    static mut MANAGER: BTreeMap::<TaskId, Process> = BTreeMap::new();
    let ready_queue = Arc::new(Mutex::new(VecDeque::<TaskId>::new()));
    // 初始化所有的处理器
    for i in 0..MAX_HART {
        unsafe {
            PROCESSORS[i].set_manager(ProcManager::new(&mut MANAGER, ready_queue.clone()));
        }
    }
}

pub fn processor() -> &'static mut Processor<Process, TaskId, ProcManager<'static>>{
    unsafe{ &mut PROCESSORS[hart_id()] }
}

/// 任务管理器
/// `tasks` 中保存所有的任务实体
/// `ready_queue` 删除任务的实体
pub struct ProcManager<'a> {
    tasks: &'a mut BTreeMap<TaskId, Process>,
    ready_queue: Arc<Mutex<VecDeque<TaskId>>>,
}

impl<'a> ProcManager<'a> {
    /// 新建任务管理器
    pub fn new(tasks: &'a mut BTreeMap<TaskId, Process>, ready_queue: Arc<Mutex<VecDeque<TaskId>>>) -> Self {
        Self {
            tasks,
            ready_queue,
        }
    }
}

impl<'a> Manage<Process, TaskId> for ProcManager<'a> {
    /// 插入一个新任务
    #[inline]
    fn insert(&mut self, id: TaskId, task: Process) {
        self.tasks.insert(id, task);
    }
    /// 根据 id 获取对应的任务
    #[inline]
    fn get_mut(&mut self, id: TaskId) -> Option<&mut Process> {
        self.tasks.get_mut(&id)
    }
    /// 删除任务实体
    #[inline]
    fn delete(&mut self, id: TaskId) {
        self.tasks.remove(&id);
    }
    /// 添加 id 进入调度队列
    fn add(&mut self, id: TaskId) {
        self.ready_queue.lock().push_back(id);
    }
    /// 从调度队列中取出 id
    fn fetch(&mut self) -> Option<TaskId> {
        self.ready_queue.lock().pop_front()
    }
}
