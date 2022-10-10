
use super::{SyscallContext, WRITEABLE};
use crate::PROCESSOR;
use syscall::{ClockId, Clock, TimeSpec};
use printlib::log;
use kernel_vm::page_table::VAddr;

impl Clock for SyscallContext {
    #[inline]
    fn clock_gettime(&self, clock_id: ClockId, tp: usize) -> isize {
        match clock_id {
            ClockId::CLOCK_MONOTONIC => {
                if let Some(mut ptr) = unsafe { PROCESSOR.current().unwrap() }
                    .address_space
                    .translate(VAddr::new(tp), WRITEABLE)
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