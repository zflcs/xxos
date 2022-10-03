use crate::coroutine::CoroutineId;
use core::task::Waker;
use alloc::{
    sync::Arc,
    task::Wake,
};


pub struct TaskWaker {
    _cid: CoroutineId,
    _prio: usize,
}

impl TaskWaker {
    pub fn new(_cid: CoroutineId, _prio: usize) -> Waker {
        Waker::from(
            Arc::new(TaskWaker {
                    _cid,
                    _prio,
                }
            )
        )
    }

    fn wake_task(&self) {
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}