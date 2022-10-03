use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};

use spin::Mutex;




#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct CoroutineId(pub usize);

impl CoroutineId {
    pub(crate) fn generate() -> CoroutineId {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id > usize::MAX / 2 {
            // TODO: 不让系统 Panic
            panic!("too many tasks!")
        }
        CoroutineId(id)
    }

    pub fn get_tid_by_usize(v: usize) -> Self {
        Self(v)
    }

    pub fn get_val(&self) -> usize {
        self.0
    } 
}



//Task包装协程
pub struct Coroutine{
    // 任务编号
    pub cid: CoroutineId,
    // future
    pub future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, 

    pub prio: usize,
}

impl Coroutine{
    //创建一个协程
    pub fn spawn(future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, prio: usize) -> Self{
        Coroutine{
            cid: CoroutineId::generate(),
            future,
            prio,
        }
    }
}
