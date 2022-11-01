#[link_section = ".text.vdso"]
pub extern "C" fn kernel_meow() -> usize {
    loop {}
}
