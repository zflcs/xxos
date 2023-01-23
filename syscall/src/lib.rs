//! 实现内核的系统调用以及暴露给用户态的系统调用

#![no_std]
#![feature(concat_idents)]
#![allow(warnings)]

extern crate syscall_macro;


mod kernel;
mod user;

use syscall_macro::SyscallMacro;

#[repr(usize)]
#[derive(Debug)]
#[derive(SyscallMacro)]
pub enum SyscallId{
    #[arguments(args = "fd, buffer_ptr, buffer_len")]
	read = 4,
    #[arguments(args = "ffff")]
    write = 5,
}

macro_rules! syscall {
    ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, $($g:ident)?)?)?)?)?)?);)+) => {
        $(
            pub unsafe fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize, $($g: usize)?)?)?)?)?)?) -> isize {
                let ret: isize;
                core::arch::asm!(
                    "ecall",
                    in("a7") $a,
                    $(
                        in("a0") $b,
                        $(
                            in("a1") $c,
                            $(
                                in("a2") $d,
                                $(
                                    in("a3") $e,
                                    $(
                                        in("a4") $f,
                                        $(
                                            in("a5") $g,
                                        )?
                                    )?
                                )?
                            )?
                        )?
                    )?
                    lateout("a0") ret,
                    options(nostack),
                );
                ret
            }
        )+
    };
}

syscall! {
    syscall0(a,);
    syscall1(a, b,);
    syscall2(a, b, c,);
    syscall3(a, b, c, d,);
    syscall4(a, b, c, d, e,);
    syscall5(a, b, c, d, e, f,);
    syscall6(a, b, c, d, e, f, g);
}


