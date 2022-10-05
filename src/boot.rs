macro_rules! clear_reg {
    ($x:ident) => {
        concat!("li ", stringify!($x), ", 0")
    };
}

#[no_mangle]
#[link_section = ".text.entry"]
#[naked]
pub unsafe extern "C" fn entry() -> ! {
    // TODO: relocate self
    // Setup registers
    core::arch::asm!(
        "fence.i",

        // Clear everything except ra, a0(hartid) and a1(fdt addr)
        clear_reg!(sp),
        clear_reg!(gp),
        clear_reg!(tp),
        clear_reg!(t0),
        clear_reg!(t1),
        clear_reg!(t2),
        clear_reg!(s0),
        clear_reg!(s1),
        // a0 is hartid,
        // a1 is fdt addr,
        clear_reg!(a2),
        clear_reg!(a3),
        clear_reg!(a4),
        clear_reg!(a5),
        clear_reg!(a6),
        clear_reg!(a7),
        clear_reg!(s2),
        clear_reg!(s3),
        clear_reg!(s4),
        clear_reg!(s5),
        clear_reg!(s6),
        clear_reg!(s7),
        clear_reg!(s8),
        clear_reg!(s9),
        clear_reg!(s10),
        clear_reg!(s11),
        clear_reg!(t3),
        clear_reg!(t4),
        clear_reg!(t5),
        clear_reg!(t6),

        // Setup stack
        "li sp, 0x80400000",

        // Jump to boot
        "j boot",
        options(noreturn),
    )
}