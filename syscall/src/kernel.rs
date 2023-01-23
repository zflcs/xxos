use super::{SyscallTrait, SyscallId};
use spin::Once;


static SYSCALL: Container<dyn SyscallTrait> = Container::new();

#[inline]
pub fn init_syscall(syscall_trait: &'static dyn SyscallTrait) {
    SYSCALL.init(syscall_trait);
}

pub fn syscall_handler(id: SyscallId, args: [usize; 6]) -> SyscallResult {
    match id {
        SyscallId::Write => SYSCALL.call(id, |sys| sys.write(args[0])),
        SyscallId::Read => SYSCALL.call(id, |sys| sys.read(args[0], args[1], args[2])),
    }
}

#[derive(Debug)]
pub enum SyscallResult {
    Done(isize),
    Unsupported(SyscallId),
}
struct Container<T: 'static + ?Sized>(spin::Once<&'static T>);

impl<T: 'static + ?Sized> Container<T> {
    #[inline]
    const fn new() -> Self {
        Self(Once::new())
    }

    #[inline]
    fn init(&self, val: &'static T) {
        self.0.call_once(|| val);
    }

    #[inline]
    fn call(&self, id: SyscallId, f: impl FnOnce(&T) -> isize) -> SyscallResult {
        self.0
            .get()
            .map_or(SyscallResult::Unsupported(id), |clock| {
                SyscallResult::Done(f(clock))
            })
    }
}
