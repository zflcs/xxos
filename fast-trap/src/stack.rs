use config::STACK_SIZE;


/// 栈
#[repr(C, align(4096))]
pub struct Stack(pub [u8; STACK_SIZE]);
