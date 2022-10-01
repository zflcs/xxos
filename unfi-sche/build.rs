fn main() {
    use std::{env, fs, path::PathBuf};

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=BASE_ADDRESS");

    let ld = &PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("linker.ld");
    // let base = 0x86000000usize;
    // let text = format!("BASE_ADDRESS = {base:#x};{LINKER}",);
    let text = format!("{LINKER}");
    fs::write(ld, text).unwrap();
    println!("cargo:rustc-link-arg=-T{}", ld.display());
}

const LINKER: &str = "
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x86000000;
SECTIONS {
    . = BASE_ADDRESS;
    .text : ALIGN(4K) {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    .data : ALIGN(4K) {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : ALIGN(4K) {
        *(.bss.uninit)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
}";
