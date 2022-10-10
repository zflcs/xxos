﻿#![allow(unused_variables)]

use crate::{ClockId, SyscallId};
use spin::Once;


pub trait Process: Sync {
    fn exit(&self, status: usize) -> isize {
        unimplemented!()
    }
    fn fork(&self) -> isize {
        unimplemented!()
    }
    fn exec(&self, path: usize, count: usize) -> isize {
        unimplemented!()
    }
    fn wait(&self, pid: isize, exit_code_ptr: usize) -> isize {
        unimplemented!()
    }
}

pub trait IO: Sync {
    fn read(&self, fd: usize, buf: usize, count: usize) -> isize {
        unimplemented!()
    }
    fn write(&self, fd: usize, buf: usize, count: usize) -> isize {
        unimplemented!()
    }
    fn open(&self, path: usize, flags: usize) -> isize {
        unimplemented!()
    }
    fn close(&self, fd: usize) -> isize {
        unimplemented!()
    }
}

pub trait Memory: Sync {
    fn mmap(
        &self,
        addr: usize,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: usize,
    ) -> isize {
        unimplemented!()
    }

    fn munmap(&self, addr: usize, length: usize) -> isize {
        unimplemented!()
    }
}

pub trait Scheduling: Sync {
    fn sched_yield(&self) -> isize {
        unimplemented!()
    }
}

pub trait Clock: Sync {
    fn clock_gettime(&self, clock_id: ClockId, tp: usize) -> isize {
        unimplemented!()
    }
}

static PROCESS: Container<dyn Process> = Container::new();
static IO: Container<dyn IO> = Container::new();
static MEMORY: Container<dyn Memory> = Container::new();
static SCHEDULING: Container<dyn Scheduling> = Container::new();
static CLOCK: Container<dyn Clock> = Container::new();

#[inline]
pub fn init_process(process: &'static dyn Process) {
    PROCESS.init(process);
}

#[inline]
pub fn init_io(io: &'static dyn IO) {
    IO.init(io);
}

#[inline]
pub fn init_memory(memory: &'static dyn Memory) {
    MEMORY.init(memory);
}

#[inline]
pub fn init_scheduling(scheduling: &'static dyn Scheduling) {
    SCHEDULING.init(scheduling);
}

#[inline]
pub fn init_clock(clock: &'static dyn Clock) {
    CLOCK.init(clock);
}

pub enum SyscallResult {
    Done(isize),
    Unsupported(SyscallId),
}

pub fn handle(id: SyscallId, args: [usize; 6]) -> SyscallResult {
    use SyscallId as Id;
    match id {
        Id::WRITE => IO.call(id, |io| io.write(args[0], args[1], args[2])),
        Id::READ => IO.call(id, |io| io.read(args[0], args[1], args[2])),
        Id::OPENAT => IO.call(id, |io| io.open(args[0], args[1])),
        Id::CLOSE => IO.call(id, |io| io.close(args[0])),
        Id::EXIT => PROCESS.call(id, |proc| proc.exit(args[0])),
        Id::CLONE => PROCESS.call(id, |proc| proc.fork()),
        Id::EXECVE => PROCESS.call(id, |proc| proc.exec(args[0], args[1])),
        Id::WAIT4 => PROCESS.call(id, |proc| proc.wait(args[0] as _, args[1])),
        Id::CLOCK_GETTIME => CLOCK.call(id, |clock| {
            clock.clock_gettime(ClockId(args[0]), args[1])
        }),
        Id::SCHED_YIELD => SCHEDULING.call(id, |sched| sched.sched_yield()),
        Id::MUNMAP => MEMORY.call(id, |memory| memory.munmap(args[0], args[1])),
        Id::MMAP => MEMORY.call(id, |memory| {
            let [addr, length, prot, flags, fd, offset] = args;
            memory.mmap(addr, length, prot as _, flags as _, fd as _, offset)
        }),
        _ => SyscallResult::Unsupported(id),
    }
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
