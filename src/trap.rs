use riscv::register::scause::Scause;
use riscv::register::{sscratch, stvec, sstatus, sie};
use riscv::register::sstatus::Sstatus;

use crate::mprintln;

#[repr(C)]
#[derive(Clone)]
pub struct TrapFrame {
    pub x: [usize; 32],   // General registers
    pub sstatus: Sstatus, // Supervisor Status Register
    pub sepc: usize,      // Supervisor exception program counter
    pub stval: usize,     // Supervisor trap value
    pub scause: Scause,   // Scause register: record the cause of exception/interrupt/trap
}

macro_rules! save_reg {
    ($x:ident, $shift:literal) => {
        concat!("sd ", stringify!($x), ", ", stringify!($shift * 8),"(sp)")
    };
}

macro_rules! restore_reg {
    ($x:ident, $shift:literal) => {
        concat!("ld ", stringify!($x), ", ", stringify!($shift * 8),"(sp)")
    };
}


#[no_mangle]
#[naked]
pub unsafe extern "C" fn trap_entry() -> ! {
    core::arch::asm!(
        "csrrw sp, sscratch, sp",
        "bnez sp, 0", // from user
        "csrr sp, sscratch",

        "0:",
        "addi sp, sp, {}",
        restore_reg!(x1, 1),
        restore_reg!(x3, 3),
        restore_reg!(x4, 4),
        restore_reg!(x5, 5),
        restore_reg!(x6, 6),
        restore_reg!(x7, 7),
        restore_reg!(x8, 8),
        restore_reg!(x9, 9),
        restore_reg!(x10, 10),
        restore_reg!(x11, 11),
        restore_reg!(x12, 12),
        restore_reg!(x13, 13),
        restore_reg!(x14, 14),
        restore_reg!(x15, 15),
        restore_reg!(x16, 16),
        restore_reg!(x17, 17),
        restore_reg!(x18, 18),
        restore_reg!(x19, 19),
        restore_reg!(x20, 20),
        restore_reg!(x21, 21),
        restore_reg!(x22, 22),
        restore_reg!(x23, 23),
        restore_reg!(x24, 24),
        restore_reg!(x25, 25),
        restore_reg!(x26, 26),
        restore_reg!(x27, 27),
        restore_reg!(x28, 28),
        restore_reg!(x29, 29),
        restore_reg!(x30, 30),
        restore_reg!(x31, 31),

        "csrrw s0, sscratch, x0",
        "csrr s1, sstatus",
        "csrr s2, sepc",
        "csrr s3, stval",
        "csrr s4, scause",

        save_reg!(s0, 2),
        save_reg!(s1, 32),
        save_reg!(s2, 33),
        save_reg!(s3, 34),
        save_reg!(s4, 35),

        "mv a0, sp",
        "j trap_impl",

        const core::mem::size_of::<TrapFrame>(),
        options(noreturn),
    )
}

#[no_mangle]
#[naked]
pub unsafe extern "C" fn trap_exit() -> ! {
    core::arch::asm!(
        restore_reg!(s1, 32),
        restore_reg!(s2, 33),
        "andi s0, s1, 1 << 8",
        "bnez s0, 0", // to kernel

        "addi s0, sp, {}",
        "csrw sscratch, s0",

        "0:",
        "csrw sstatus, s1",
        "csrw sepc, s2",

        restore_reg!(x1, 1),
        restore_reg!(x3, 3),
        restore_reg!(x4, 4),
        restore_reg!(x5, 5),
        restore_reg!(x6, 6),
        restore_reg!(x7, 7),
        restore_reg!(x8, 8),
        restore_reg!(x9, 9),
        restore_reg!(x10, 10),
        restore_reg!(x11, 11),
        restore_reg!(x12, 12),
        restore_reg!(x13, 13),
        restore_reg!(x14, 14),
        restore_reg!(x15, 15),
        restore_reg!(x16, 16),
        restore_reg!(x17, 17),
        restore_reg!(x18, 18),
        restore_reg!(x19, 19),
        restore_reg!(x20, 20),
        restore_reg!(x21, 21),
        restore_reg!(x22, 22),
        restore_reg!(x23, 23),
        restore_reg!(x24, 24),
        restore_reg!(x25, 25),
        restore_reg!(x26, 26),
        restore_reg!(x27, 27),
        restore_reg!(x28, 28),
        restore_reg!(x29, 29),
        restore_reg!(x30, 30),
        restore_reg!(x31, 31),
        restore_reg!(sp, 2),
        "sret",

        const core::mem::size_of::<TrapFrame>(),
        options(noreturn),
    )
}

#[no_mangle]
unsafe fn trap_impl(tf: *mut TrapFrame) -> ! {
    mprintln!("Trap!").unwrap();
    trap_exit()
}

pub unsafe fn init() {
    sscratch::write(0);
    stvec::write(trap_entry as usize, stvec::TrapMode::Direct);

    sstatus::set_sie();
    sie::set_sext();
}