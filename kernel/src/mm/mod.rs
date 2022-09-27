mod heap;
mod pagemanage;
mod kernel_space;



pub use pagemanage::Sv39Manager;
pub use kernel_space::kernel_space;
pub use heap::{PAGE, init, test};