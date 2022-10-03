
use core::task::Waker;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::{
    coroutine::{Coroutine, CoroutineId},
    task_waker::TaskWaker
};

pub struct Executor {
    pub tasks: BTreeMap<CoroutineId, Arc<Coroutine>>,
    pub ready_queue: Vec<CoroutineId>,
    pub block_queue: Vec<CoroutineId>,
    pub waker_cache: BTreeMap<CoroutineId, Arc<Waker>>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            ready_queue: Vec::new(),
            block_queue: Vec::new(),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn add_coroutine(&mut self, task: Arc<Coroutine>) {
        let cid = task.cid;
        self.ready_queue.push(cid);
        self.tasks.insert(cid, task);
    }

    pub fn get_waker(&mut self, cid: CoroutineId, prio: usize) -> Arc<Waker> {
        self.waker_cache
                        .entry(cid)
                        .or_insert_with(|| Arc::new(TaskWaker::new(cid, prio)))
                        .clone()
    }

    pub fn fetch(&mut self) -> Option<Arc<Coroutine>> {
        if !self.ready_queue.is_empty() {
            let cid = self.ready_queue.remove(0);
            return self.get_task(&cid);
        } else {
            if !self.block_queue.is_empty() {
                let cid = self.block_queue.remove(0);
                return self.get_task(&cid);
            }
        }
        return None;
    }

    fn get_task(&self, cid: &CoroutineId) -> Option<Arc<Coroutine>> {
        if let Some(ret) = self.tasks.get(cid) {
            return Some(ret.clone());
        } else {
            return None;
        }
    }

    pub fn is_empty(&self) -> bool { self.ready_queue.is_empty() && self.block_queue.is_empty() }
}