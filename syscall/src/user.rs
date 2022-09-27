﻿use crate::{ClockId, SyscallId, TimeSpec};
use bitflags::*;
use native::*;

/// see <https://man7.org/linux/man-pages/man2/write.2.html>.
#[inline]
pub fn write(fd: usize, buffer: &[u8]) -> isize {
    unsafe { syscall3(SyscallId::WRITE, fd, buffer.as_ptr() as _, buffer.len()) }
}

#[inline]
pub fn read(fd: usize, buffer: &[u8]) -> isize {
    unsafe { syscall3(SyscallId::READ, fd, buffer.as_ptr() as _, buffer.len()) }
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

#[inline]
pub fn open(path: &str, flags: OpenFlags) -> isize {
    unsafe {
        syscall2(
            SyscallId::OPENAT,
            path.as_ptr() as usize,
            flags.bits as usize,
        )
    }
}

#[inline]
pub fn close(fd: usize) -> isize {
    unsafe { syscall1(SyscallId::CLOSE, fd) }
}

/// see <https://man7.org/linux/man-pages/man2/exit.2.html>.
#[inline]
pub fn exit(exit_code: i32) -> isize {
    unsafe { syscall1(SyscallId::EXIT, exit_code as _) }
}

/// see <https://man7.org/linux/man-pages/man2/sched_yield.2.html>.
#[inline]
pub fn sched_yield() -> isize {
    unsafe { syscall0(SyscallId::SCHED_YIELD) }
}

/// see <https://man7.org/linux/man-pages/man2/clock_gettime.2.html>.
#[inline]
pub fn clock_gettime(clockid: ClockId, tp: *mut TimeSpec) -> isize {
    unsafe { syscall2(SyscallId::CLOCK_GETTIME, clockid.0, tp as _) }
}

pub fn fork() -> isize {
    unsafe { syscall0(SyscallId::CLONE) }
}

pub fn exec(path: &str) -> isize {
    unsafe { syscall2(SyscallId::EXECVE, path.as_ptr() as usize, path.len()) }
}

pub fn wait(exit_code_ptr: *mut i32) -> isize {
    loop {
        let pid = unsafe { syscall2(SyscallId::WAIT4, usize::MAX, exit_code_ptr as usize) };
        if pid == -1 {
            sched_yield();
            continue;
        } else {
            return pid;
        }
    }
}

pub fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    loop {
        let pid = unsafe { syscall2(SyscallId::WAIT4, pid as usize, exit_code_ptr as usize) };
        if pid == -1 {
            sched_yield();
        } else {
            return pid;
        }
    }
}

/// 这个模块包含调用系统调用的最小封装，用户可以直接使用这些函数调用自定义的系统调用。
pub mod native {
    use crate::SyscallId;
    use core::arch::asm;

    #[inline(always)]
    pub unsafe fn syscall0(id: SyscallId) -> isize {
        let ret: isize;
        asm!("ecall",
            in("a7") id.0,
            out("a0") ret,
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall1(id: SyscallId, a0: usize) -> isize {
        let ret: isize;
        asm!("ecall",
            inlateout("a0") a0 => ret,
            in("a7") id.0,
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall2(id: SyscallId, a0: usize, a1: usize) -> isize {
        let ret: isize;
        asm!("ecall",
            in("a7") id.0,
            inlateout("a0") a0 => ret,
            in("a1") a1,
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall3(id: SyscallId, a0: usize, a1: usize, a2: usize) -> isize {
        let ret: isize;
        asm!("ecall",
            in("a7") id.0,
            inlateout("a0") a0 => ret,
            in("a1") a1,
            in("a2") a2,
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall4(id: SyscallId, a0: usize, a1: usize, a2: usize, a3: usize) -> isize {
        let ret: isize;
        asm!("ecall",
            in("a7") id.0,
            inlateout("a0") a0 => ret,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall5(
        id: SyscallId,
        a0: usize,
        a1: usize,
        a2: usize,
        a3: usize,
        a4: usize,
    ) -> isize {
        let ret: isize;
        asm!("ecall",
            in("a7") id.0,
            inlateout("a0") a0 => ret,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
            in("a4") a4,
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall6(
        id: SyscallId,
        a0: usize,
        a1: usize,
        a2: usize,
        a3: usize,
        a4: usize,
        a5: usize,
    ) -> isize {
        let ret: isize;
        asm!("ecall",
            in("a7") id.0,
            inlateout("a0") a0 => ret,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
            in("a4") a4,
            in("a5") a5,
        );
        ret
    }
}
