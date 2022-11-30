use fast_trap::TrapStackBlock;
use crate::config::STACK_SIZE;
use printlib::log;

#[repr(C, align(4096))]
pub struct Stack(pub [u8; STACK_SIZE]);

pub struct StackRef(pub &'static mut Stack);

impl AsRef<[u8]> for StackRef {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0 .0
    }
}

impl AsMut<[u8]> for StackRef {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0 .0
    }
}

impl TrapStackBlock for StackRef {}

impl Drop for StackRef {
    fn drop(&mut self) {
        log::info!("Stack Dropped!")
    }
}