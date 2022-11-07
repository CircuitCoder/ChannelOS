use crate::trap::wfi;
use align_data::include_aligned;
use align_data::Align64;

#[no_mangle]
pub fn idle() -> ! {
    loop {
        wfi();
    }
}

#[link_section = ".rodata"]
pub static TEST: &'static [u8] = include_aligned!(Align64, "../user/test.elf");

#[link_section = ".rodata"]
pub static PUTCHAR: &'static [u8] = include_aligned!(Align64, "../user/putchar.elf");