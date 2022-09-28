use kernel_context::LocalContext;
use super::process::ProcId;

/// 线程
/// 每个线程有自己的 id，栈，上下文，并且能够知道进程
/// 这里直接存进程的引用了，从生命周期看，线程的生命周期比进程短
/// 但是从 rust 语言来看，这里记录了引用，然后其他的系统调用也会用到进程的引用，所以会导致冲突
/// 但是可以利用 id，只保存进程的 id，这样可以直接获取到进程的信息
pub struct Thread {
    /// id
    pub id: ThreadId,
    /// 栈
    pub stack: ThreadStack,
    /// 上下文
    pub context: LocalContext,
    /// 所属进程 id
    pub pid: ProcId, 
}

pub struct ThreadId(usize);

impl ThreadId {
    pub(crate) fn generate() -> ThreadId {
        // 线程编号计数器
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        ThreadId(id)
    }

    pub fn from(v: usize) -> Self {
        Self(v)
    }

    pub fn get_val(&self) -> usize {
        self.0
    }
}

#[repr(C)]
pub struct ThreadStack{
    is_used: bool,
    content: [u8; 4096],
}