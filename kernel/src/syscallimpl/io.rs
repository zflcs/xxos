
// use super::{SyscallContext, READABLE};
// use crate::PROCESSOR;
// use syscall::IO;
// use printlib::log;
// use kernel_vm::page_table::VAddr;
// use task_manage::Task;



// impl IO for SyscallContext {
//     #[inline]
//     fn write(&self, fd: usize, buf: usize, count: usize) -> isize {
//         let current = unsafe { PROCESSOR.current_task().unwrap() };
//             if let Some(ptr) = process.inner.lock().address_space.translate(VAddr::new(buf), READABLE) {
//                 if fd == 0 {
//                     print!("{}", unsafe {
//                         core::str::from_utf8_unchecked(core::slice::from_raw_parts(
//                             ptr.as_ptr(),
//                             count,
//                         ))
//                     });
//                     count as _
//                 } else {
//                     log::error!("unsupported fd: {fd}");
//                     -1
//                 }
//             } else {
//                 log::error!("ptr not readable");
//                 -1
//             }
//         }
//     }

//     #[inline]
//     #[allow(deprecated)]
//     fn read(&self, fd: usize, buf: usize, count: usize) -> isize {
//         let current = unsafe { PROCESSOR.current().unwrap() };
//         if fd == 0 || fd >= current.fd_table.len() {
//             return -1;
//         }
//         if let Some(ptr) = current.address_space.translate(VAddr::new(buf), WRITEABLE) {
//             if fd == 1 {
//                 let mut ptr = ptr.as_ptr();
//                 for _ in 0..count {
//                     let c = sbi_rt::legacy::console_getchar() as u8;
//                     unsafe {
//                         *ptr = c;
//                         ptr = ptr.add(1);
//                     }
//                 }
//                 count as _
//             } else if fd != 0 && fd < current.fd_table.len() {
//                 if let Some(file) = &current.fd_table[fd] {
//                     let mut _file = file.lock();
//                     if !_file.readable() {
//                         return -1;
//                     }
//                     let mut v: Vec<&'static mut [u8]> = Vec::new();
//                     unsafe {
//                         let raw_buf: &'static mut [u8] =
//                             core::slice::from_raw_parts_mut(ptr.as_ptr(), count);
//                         v.push(raw_buf);
//                     }
//                     _file.read(UserBuffer::new(v)) as _
//                 } else {
//                     log::error!("unsupported fd: {fd}");
//                     -1
//                 }
//             } else {
//                 log::error!("unsupported fd: {fd}");
//                 -1
//             }
//         } else {
//             log::error!("ptr not writeable");
//             -1
//         }
//     }

//     #[inline]
//     fn open(&self, path: usize, flags: usize) -> isize {
//         // FS.open(, flags)
//         let current = unsafe { PROCESSOR.current().unwrap() };
//         if let Some(ptr) = current.address_space.translate(VAddr::new(path), READABLE) {
//             let mut string = String::new();
//             let mut raw_ptr: *mut u8 = ptr.as_ptr();
//             loop {
//                 unsafe {
//                     let ch = *raw_ptr;
//                     if ch == 0 {
//                         break;
//                     }
//                     string.push(ch as char);
//                     raw_ptr = (raw_ptr as usize + 1) as *mut u8;
//                 }
//             }

//             if let Some(fd) =
//                 FS.open(string.as_str(), OpenFlags::from_bits(flags as u32).unwrap())
//             {
//                 let new_fd = current.fd_table.len();
//                 current.fd_table.push(Some(Mutex::new(fd.as_ref().clone())));
//                 new_fd as isize
//             } else {
//                 -1
//             }
//         } else {
//             log::error!("ptr not writeable");
//             -1
//         }
//     }

//     #[inline]
//     fn close(&self, fd: usize) -> isize {
//         let current = unsafe { PROCESSOR.current().unwrap() };
//         if fd >= current.fd_table.len() || current.fd_table[fd].is_none() {
//             return -1;
//         }
//         current.fd_table[fd].take();
//         0
//     }
// }