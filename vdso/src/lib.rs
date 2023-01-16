#![no_std]

#[link_section = ".text.vdso"]
pub unsafe extern "C" fn user_entry() {
    core::arch::asm!(
        "
            ecall
        "
    )
}