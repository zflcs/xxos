mod io;
mod clock;
mod process;
mod schedule;


pub fn init_syscall() {
    syscall::init_io(&SyscallContext);
    syscall::init_process(&SyscallContext);
    syscall::init_scheduling(&SyscallContext);
    syscall::init_clock(&SyscallContext);
}

struct SyscallContext;

use kernel_vm::{
    page_table::{Sv39, VmFlags},
};
const READABLE: VmFlags<Sv39> = VmFlags::build_from_str("RV");
const WRITEABLE: VmFlags<Sv39> = VmFlags::build_from_str("W_V");