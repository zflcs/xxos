
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::task::{Poll, Context};
use alloc::sync::Arc;
use crate::executor::EXECUTOR;
use runtime::Coroutine;
use spin::Mutex;

#[no_mangle]
#[inline(never)]
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    let task = Arc::new(Coroutine::spawn(Mutex::new(future), prio));
    unsafe{ EXECUTOR.as_mut().unwrap().add_coroutine(task); }
    printlib::log::debug!("add task ok");
}

#[no_mangle]
pub fn run(){
    loop {
        let ex = unsafe { EXECUTOR.as_mut().unwrap() };
        let task = ex.fetch();
        if task.is_none() { break; }
        let waker = ex.get_waker(task.clone().unwrap().cid, task.clone().unwrap().prio);
 
        // creat Context
        let mut context = Context::from_waker(&*waker);
        match task.clone().unwrap().future.lock().as_mut().poll(&mut context) {
            Poll::Pending => {  }
            Poll::Ready(()) => {
                // remove task
                
            }
        }; 
        //if check_bitmap_should_yield() { sys_yield(); }
    }
}