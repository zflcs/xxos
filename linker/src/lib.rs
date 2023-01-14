//! 在 kernel 的 build.rs 和 src 之间共享常量。

#![no_std]
#![deny(warnings, missing_docs)]

use core::{fmt, fmt::{Formatter, Debug}};

/// 链接脚本。
/// 
pub const SCRIPT: &[u8] = b"\
OUTPUT_ARCH(riscv)
ENTRY(_start)
SECTIONS {
    . = 0x80200000;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        svdso = .;
        *(.text.vdso);
        . = ALIGN(4K);
        evdso = .;
        . = ALIGN(4K);
        strampoline = .;
        *(.text.fast_handler);
        . = ALIGN(4K);
        etrampoline = .;
        *(.text .text.*)
    }
    .rodata : ALIGN(4K) {
        rodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    .data : ALIGN(4K) {
        data = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : ALIGN(4K) {
        sstack = .;
        *(.bss.stack)
        . = ALIGN(4K);
        bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    end = .;
}";

/// 返回段落
pub struct Paragraph {
    /// 段起始地址
    pub start: usize,
    /// 段结束地址
    pub end: usize,
}

impl Debug for Paragraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("start:{:#x} end:{:#x}", self.start, self.end))
    }
}

/// 代码段
#[inline]
pub fn locate_text() -> Paragraph {
    extern "C" {
        fn _start();
        fn rodata();
    }
    Paragraph { start: _start as _, end: rodata as _ }
}

/// 只读数据段
#[inline]
pub fn locate_rodata() -> Paragraph {
    extern "C" {
        fn rodata();
        fn data();
    }
    Paragraph { start: rodata as _, end: data as _ }
}

/// 数据段
#[inline]
pub fn locate_data() -> Paragraph {
    extern "C" {
        fn data();
        fn sstack();
    }
    Paragraph { start: data as _, end: sstack as _ }
}

/// bss 段
#[inline]
pub fn locate_bss() -> Paragraph {
    extern "C" {
        fn bss();
        fn end();
    }
    Paragraph { start: bss as _, end: end as _ }
}

/// 返回 stack 段
#[inline]
pub fn locate_stack() -> Paragraph {
    extern "C" {
        fn sstack();
        fn bss();
    }
    Paragraph { start: sstack as _, end: bss as _ }
}

/// 返回 trampoline 段
#[inline]
pub fn locate_trampoline() -> Paragraph {
    extern "C" {
        fn strampoline();
        fn etrampoline();
    }
    Paragraph { start: strampoline as _, end: etrampoline as _ }
}

/// vdso 段
#[inline]
pub fn locate_vdso() -> Paragraph {
    extern "C" {
        fn svdso();
        fn evdso();
    }
    Paragraph { start: svdso as _, end: evdso as _ }
}

/// 清零 .bss 段。
#[inline]
pub unsafe fn zero_bss() {
    extern "C" {
        fn bss();
        fn end();
    }
    r0::zero_bss::<u64>(bss as _, end as _);
}

