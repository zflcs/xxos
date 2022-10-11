use super::{SyscallContext, READABLE, WRITEABLE};
use crate::{
    PROCESSOR, read_all, FS,
    task::process::ProcId,
};
use syscall::Process;
use easy_fs::{OpenFlags, FSManager};
use xmas_elf::ElfFile;
use kernel_vm::page_table::VAddr;


impl Process for SyscallContext {
    #[inline]
    fn exit(&self, _status: usize) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        if let Some(parent) = unsafe { PROCESSOR.get_task(current.parent) } {
            let pair = parent
                .children
                .iter()
                .enumerate()
                .find(|(_, &id)| id == current.pid);
            if let Some((idx, _)) = pair {
                parent.children.remove(idx);
            }
            for (_, &id) in current.children.iter().enumerate() {
                parent.children.push(id);
            }
        }
        0
    }

    fn fork(&self) -> isize {
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

    fn exec(&self, path: usize, count: usize) -> isize {
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
    fn wait(&self, pid: isize, exit_code_ptr: usize) -> isize {
        let current = unsafe { PROCESSOR.current().unwrap() };
        if let Some(mut ptr) = current
            .address_space
            .translate(VAddr::new(exit_code_ptr), WRITEABLE)
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