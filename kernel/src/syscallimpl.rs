use crate::{
    fsimpl::{read_all, FS},
    task::process::ProcId,
    PROCESSOR,
};
use alloc::{vec::Vec, string::String};
use printlib::log;
use easy_fs::UserBuffer;
use easy_fs::{FSManager, OpenFlags};
use kernel_vm::{
    page_table::{Sv39, VAddr, VmFlags},
};
use spin::Mutex;
use syscall::*;
use xmas_elf::ElfFile;

pub fn init_syscall() {
    syscall::init_io(&SyscallContext);
    syscall::init_process(&SyscallContext);
    syscall::init_scheduling(&SyscallContext);
    syscall::init_clock(&SyscallContext);
}

struct SyscallContext;
const READABLE: VmFlags<Sv39> = VmFlags::build_from_str("RV");
const WRITEABLE: VmFlags<Sv39> = VmFlags::build_from_str("W_V");

impl IO for SyscallContext {
    #[inline]
    fn write(&self, _caller: Caller, fd: usize, buf: usize, count: usize) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        if let Some(ptr) = current.address_space.translate(VAddr::new(buf), READABLE) {
            if fd == 0 {
                print!("{}", unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        ptr.as_ptr(),
                        count,
                    ))
                });
                count as _
            } else if fd > 1 && fd < current.fd_table.len() {
                if let Some(file) = &current.fd_table[fd] {
                    let mut _file = file.lock();
                    if !_file.writable() {
                        return -1;
                    }
                    let mut v: Vec<&'static mut [u8]> = Vec::new();
                    unsafe {
                        let raw_buf: &'static mut [u8] =
                            core::slice::from_raw_parts_mut(ptr.as_ptr(), count);
                        v.push(raw_buf);
                    }
                    _file.write(UserBuffer::new(v)) as _
                } else {
                    log::error!("unsupported fd: {fd}");
                    -1
                }
            } else {
                log::error!("unsupported fd: {fd}");
                -1
            }
        } else {
            log::error!("ptr not readable");
            -1
        }
    }

    #[inline]
    #[allow(deprecated)]
    fn read(&self, _caller: Caller, fd: usize, buf: usize, count: usize) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        if fd == 0 || fd >= current.fd_table.len() {
            return -1;
        }
        if let Some(ptr) = current.address_space.translate(VAddr::new(buf), WRITEABLE) {
            if fd == 1 {
                let mut ptr = ptr.as_ptr();
                for _ in 0..count {
                    let c = sbi_rt::legacy::console_getchar() as u8;
                    unsafe {
                        *ptr = c;
                        ptr = ptr.add(1);
                    }
                }
                count as _
            } else if fd != 0 && fd < current.fd_table.len() {
                if let Some(file) = &current.fd_table[fd] {
                    let mut _file = file.lock();
                    if !_file.readable() {
                        return -1;
                    }
                    let mut v: Vec<&'static mut [u8]> = Vec::new();
                    unsafe {
                        let raw_buf: &'static mut [u8] =
                            core::slice::from_raw_parts_mut(ptr.as_ptr(), count);
                        v.push(raw_buf);
                    }
                    _file.read(UserBuffer::new(v)) as _
                } else {
                    log::error!("unsupported fd: {fd}");
                    -1
                }
            } else {
                log::error!("unsupported fd: {fd}");
                -1
            }
        } else {
            log::error!("ptr not writeable");
            -1
        }
    }

    #[inline]
    fn open(&self, _caller: Caller, path: usize, flags: usize) -> isize {
        // FS.open(, flags)
        let current = unsafe { PROCESSOR.current().unwrap() };
        if let Some(ptr) = current.address_space.translate(VAddr::new(path), READABLE) {
            let mut string = String::new();
            let mut raw_ptr: *mut u8 = ptr.as_ptr();
            loop {
                unsafe {
                    let ch = *raw_ptr;
                    if ch == 0 {
                        break;
                    }
                    string.push(ch as char);
                    raw_ptr = (raw_ptr as usize + 1) as *mut u8;
                }
            }

            if let Some(fd) =
                FS.open(string.as_str(), OpenFlags::from_bits(flags as u32).unwrap())
            {
                let new_fd = current.fd_table.len();
                current.fd_table.push(Some(Mutex::new(fd.as_ref().clone())));
                new_fd as isize
            } else {
                -1
            }
        } else {
            log::error!("ptr not writeable");
            -1
        }
    }

    #[inline]
    fn close(&self, _caller: Caller, fd: usize) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        if fd >= current.fd_table.len() || current.fd_table[fd].is_none() {
            return -1;
        }
        current.fd_table[fd].take();
        0
    }
}

impl Process for SyscallContext {
    #[inline]
    fn exit(&self, _caller: Caller, _status: usize) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        if let Some(parent) = unsafe { PROCESSOR.get_task(current.parent) } {
            let pair = parent
                .children
                .iter()
                .enumerate()
                .find(|(_, &id)| id == current.pid);
            if let Some((idx, _)) = pair {
                parent.children.remove(idx);
                // log::debug!("parent remove child {}", parent.children.remove(idx));
            }
            for (_, &id) in current.children.iter().enumerate() {
                // log::warn!("parent insert child {}", id);
                parent.children.push(id);
            }
        }
        0
    }

    fn fork(&self, _caller: Caller) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        let mut child_proc = current.fork().unwrap();
        let pid = child_proc.pid;
        let context = &mut child_proc.context.context;
        *context.a_mut(0) = 0 as _;
        unsafe {
            PROCESSOR.add(pid, child_proc);
        }
        pid.get_val() as isize
    }

    fn exec(&self, _caller: Caller, path: usize, count: usize) -> isize {
        const READABLE: VmFlags<Sv39> = VmFlags::build_from_str("RV");
        let current = unsafe { PROCESSOR.current().unwrap() };
        if let Some(ptr) = current.address_space.translate(VAddr::new(path), READABLE) {
            let name = unsafe {
                core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr.as_ptr(), count))
            };
            current.exec(
                ElfFile::new(read_all(FS.open(name, OpenFlags::RDONLY).unwrap()).as_slice())
                    .unwrap(),
            );
            0
        } else {
            -1
        }
    }

    // 简化的 wait 系统调用，pid == -1，则需要等待所有子进程结束，若当前进程有子进程，则返回 -1，否则返回 0
    // pid 为具体的某个值，表示需要等待某个子进程结束，因此只需要在 TASK_MANAGER 中查找是否有任务
    // 简化了进程的状态模型
    fn wait(&self, _caller: Caller, pid: isize, exit_code_ptr: usize) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        const WRITABLE: VmFlags<Sv39> = VmFlags::build_from_str("W_V");
        if let Some(mut ptr) = current
            .address_space
            .translate(VAddr::new(exit_code_ptr), WRITABLE)
        {
            unsafe { *ptr.as_mut() = 333 as i32 };
        }
        if pid == -1 {
            if current.children.is_empty() {
                return 0;
            } else {
                return -1;
            }
        } else {
            if unsafe { PROCESSOR.get_task(ProcId::from(pid as usize)).is_none() } {
                return pid;
            } else {
                return -1;
            }
        }
    }
}

impl Scheduling for SyscallContext {
    #[inline]
    fn sched_yield(&self, _caller: Caller) -> isize {
        0
    }
}

impl Clock for SyscallContext {
    #[inline]
    fn clock_gettime(&self, _caller: Caller, clock_id: ClockId, tp: usize) -> isize {
        const WRITABLE: VmFlags<Sv39> = VmFlags::build_from_str("W_V");
        match clock_id {
            ClockId::CLOCK_MONOTONIC => {
                if let Some(mut ptr) = unsafe { PROCESSOR.current().unwrap() }
                    .address_space
                    .translate(VAddr::new(tp), WRITABLE)
                {
                    let time = riscv::register::time::read() * 10000 / 125;
                    *unsafe { ptr.as_mut() } = TimeSpec {
                        tv_sec: time / 1_000_000_000,
                        tv_nsec: time % 1_000_000_000,
                    };
                    0
                } else {
                    log::error!("ptr not readable");
                    -1
                }
            }
            _ => -1,
        }
    }
}