
use runtime::Executor;
/// HEAP 指向的是用户进程的 HEAP
static mut EXECUTOR: Option<&mut Executor> = None;

pub fn init(executor: &'static mut Executor) {
    // 将用户进程堆的指针传递给共享库的堆，从而使得可以在用户进程的堆中分配数据
    unsafe { EXECUTOR = Some(executor) };
}