// use core::ptr::NonNull;

// use kernel_context::LocalContext;
// use super::process::ProcId;

// /// 线程
// /// 每个线程有自己的 id，栈，上下文，并且能够知道进程
// /// 这里直接存进程的引用了，从生命周期看，线程的生命周期比进程短
// /// 但是从 rust 语言来看，这里记录了引用，然后其他的系统调用也会用到进程的引用，所以会导致冲突
// /// 但是可以利用 id，只保存进程的 id，这样可以直接获取到进程的信息
// pub struct Thread {
//     /// id
//     pub id: ThreadId,
//     /// 栈
//     pub stack: ThreadStack,
//     /// 上下文
//     pub context: LocalContext,
//     /// 所属进程 id
//     pub pid: ProcId,
//     /// 表示当前的线程栈是否被占用
//     pub is_used: bool,
// }

// impl Thread {
//     /// 运行
//     pub fn execute(&mut self) -> usize{
//         unsafe { self.context.execute() }
//     }
// }


// use core::sync::atomic::{AtomicUsize, Ordering};

// pub struct ThreadId(usize);

// impl ThreadId {
//     pub(crate) fn generate() -> ThreadId {
//         // 线程编号计数器
//         static CC: AtomicUsize = AtomicUsize::new(0);
//         let id = CC.fetch_add(1, Ordering::Relaxed);
//         ThreadId(id)
//     }

//     pub fn from(v: usize) -> Self {
//         Self(v)
//     }

//     pub fn get_val(&self) -> usize {
//         self.0
//     }
// }

// #[repr(C)]
// pub struct ThreadStack{
//     /// 物理页的起始地址
//     base: NonNull<u8>,
//     /// 栈大小
//     size: usize,
//     /// 虚拟地址的起始地址
//     vbase: NonNull<u8>,
// }