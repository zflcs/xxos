
use core::task::Waker;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::{
    coroutine::{Coroutine, CoroutineId},
    task_waker::TaskWaker,
    config::PRIO_NUM,
};

const fn vec_init() -> Vec<CoroutineId> {
    Vec::new()
}

const VAL: Vec<CoroutineId> = vec_init();

pub struct Executor {
    pub tasks: BTreeMap<CoroutineId, Arc<Coroutine>>,
    pub ready_queue: [Vec<CoroutineId>; PRIO_NUM],
    pub block_queue: Vec<CoroutineId>,
    pub waker_cache: BTreeMap<CoroutineId, Arc<Waker>>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            ready_queue: [VAL; PRIO_NUM],
            block_queue: Vec::new(),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn add_coroutine(&mut self, task: Arc<Coroutine>) {
        let cid = task.cid;
        let prio = task.prio;
        self.ready_queue[prio].push(cid);
        self.tasks.insert(cid, task);
    }

    pub fn get_waker(&mut self, cid: CoroutineId, prio: usize) -> Arc<Waker> {
        self.waker_cache
                        .entry(cid)
                        .or_insert_with(|| Arc::new(TaskWaker::new(cid, prio)))
                        .clone()
    }

    pub fn fetch(&mut self) -> Option<Arc<Coroutine>> {
        for i in 0..PRIO_NUM {
            if !self.ready_queue[i].is_empty() {
                let cid = self.ready_queue[i].remove(0);
                return self.get_task(&cid);
            }
        }
        if !self.block_queue.is_empty() {
            let cid = self.block_queue.remove(0);
            return self.get_task(&cid);
        }
        return None;
    }

    pub fn get_task(&self, cid: &CoroutineId) -> Option<Arc<Coroutine>> {
        if let Some(ret) = self.tasks.get(cid) {
            return Some(ret.clone());
        } else {
            return None;
        }
    }

    pub fn is_empty(&self) -> bool { self.ready_queue.is_empty() && self.block_queue.is_empty() }

    pub fn del_task(&mut self, cid: CoroutineId) {
        self.tasks.remove(&cid);
        self.waker_cache.remove(&cid);
    }

    pub fn block_task(&mut self, cid: CoroutineId) {
        self.block_queue.push(cid);
    }
}