#![no_std]

mod coroutine;
mod task_waker;
mod executor;
mod config;

extern crate alloc;

pub use executor::Executor;
pub use coroutine::Coroutine;
pub use coroutine::CoroutineId;
use spin::Once;
use core::future::Future;
use core::pin::Pin;
use alloc::boxed::Box;

// 共享模块在各个层面的实现
pub struct ModuleFun(usize);

impl ModuleFun {
    pub fn add_coroutine(&self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
        unsafe {
            let add_coroutine_true: fn(Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, usize) = 
                core::mem::transmute((self.0 as *mut usize).add(0) as usize);
            add_coroutine_true(future, prio);
        }
    }
}


static UNFI_SCHE: Once<&'static ModuleFun> = Once::new();

pub fn init_unfi_sche(unfi_sche: &'static ModuleFun) {
    UNFI_SCHE.call_once(|| unfi_sche);
}

pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    UNFI_SCHE.get().unwrap().add_coroutine(future, prio);
}

