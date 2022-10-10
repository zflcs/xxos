use super::SyscallContext;
use syscall::Scheduling;

impl Scheduling for SyscallContext {
    #[inline]
    fn sched_yield(&self) -> isize {
        0
    }
}