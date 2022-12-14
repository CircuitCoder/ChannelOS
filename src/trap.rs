use riscv::register::scause::{self, Exception, Interrupt, Scause, Trap};
use riscv::register::sstatus::Sstatus;
use riscv::register::{sie, sscratch, sstatus, stvec};

use crate::mem::addr::{PhysAddr, VirtAddr};
use crate::mem::set::{MapArea, MapPermission};
use crate::{mprintln, sched, service, uprint};

#[repr(C)]
#[derive(Clone)]
pub struct TrapFrame {
    pub x: [usize; 32],   // General registers
    pub sstatus: Sstatus, // Supervisor Status Register
    pub sepc: usize,      // Supervisor exception program counter
    pub stval: usize,     // Supervisor trap value
    pub scause: Scause,   // Scause register: record the cause of exception/interrupt/trap
}

impl TrapFrame {
    pub fn with_process(is_user: bool, entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();

        sstatus.set_spie(true);
        if is_user {
            sstatus.set_spp(sstatus::SPP::User);
        } else {
            sstatus.set_spp(sstatus::SPP::Supervisor);
        }

        let mut result = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            stval: 0,
            scause: scause::read(),
        };
        result.x[2] = sp;
        result
    }
}

macro_rules! save_reg {
    ($x:ident, $shift:literal) => {
        concat!("sd ", stringify!($x), ", ", stringify!($shift * 8), "(sp)")
    };
}

macro_rules! restore_reg {
    ($x:ident, $shift:literal) => {
        concat!("ld ", stringify!($x), ", ", stringify!($shift * 8), "(sp)")
    };
}

#[no_mangle]
#[naked]
pub unsafe extern "C" fn trap_entry() -> ! {
    core::arch::asm!(
        ".align 4",
        "csrrw sp, sscratch, sp",

        "addi sp, sp, -{}",
        save_reg!(x1, 1),
        save_reg!(x3, 3),
        save_reg!(x4, 4),
        save_reg!(x5, 5),
        save_reg!(x6, 6),
        save_reg!(x7, 7),
        save_reg!(x8, 8),
        save_reg!(x9, 9),
        save_reg!(x10, 10),
        save_reg!(x11, 11),
        save_reg!(x12, 12),
        save_reg!(x13, 13),
        save_reg!(x14, 14),
        save_reg!(x15, 15),
        save_reg!(x16, 16),
        save_reg!(x17, 17),
        save_reg!(x18, 18),
        save_reg!(x19, 19),
        save_reg!(x20, 20),
        save_reg!(x21, 21),
        save_reg!(x22, 22),
        save_reg!(x23, 23),
        save_reg!(x24, 24),
        save_reg!(x25, 25),
        save_reg!(x26, 26),
        save_reg!(x27, 27),
        save_reg!(x28, 28),
        save_reg!(x29, 29),
        save_reg!(x30, 30),
        save_reg!(x31, 31),

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
        "call trap_impl",
        "j trap_exit",

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

        "addi s0, sp, {}",
        "csrw sscratch, s0",

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
unsafe fn trap_impl(tf: *mut TrapFrame) {
    let tf = &mut *tf;
    mprintln!("[Trap] enter <- {:#x}", tf.sepc);
    match tf.scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::timer::tick(tf);
        }
        Trap::Exception(Exception::UserEnvCall) => {
            syscall(tf);
        }
        x => {
            panic!(
                "Unimplemented trap: {:?} at {:#x}, tval = {:#x}",
                x, tf.sepc, tf.stval
            );
        }
    }
    mprintln!("[Trap] exit -> {:#x}", tf.sepc);
}

pub fn init() {
    unsafe {
        sscratch::write(0);
        stvec::write(trap_entry as usize, stvec::TrapMode::Direct);

        // sstatus::set_sie();
        sie::set_sext();
    }
}

pub fn wfi() {
    unsafe {
        sstatus::set_sie();
        riscv::asm::wfi();
    }
}

fn syscall(tf: &mut TrapFrame) {
    mprintln!("[SyncSyscall] num: {}", tf.x[10]);
    match tf.x[10] {
        0x3 => {
            let srv = tf.x[11];
            mprintln!("[SyncSyscall] requesting service {}", srv);
            if srv >= service::SERVICE_LIST.len() {
                panic!("[SyncSyscall] invalid service {}", srv);
            }

            let (req, resp) = service::SERVICE_LIST[srv]();
            let mut sch = sched::SCHEDULER.lock();
            let proc = sch.running_process();
            let req_area = MapArea::linear(
                req.floor()..PhysAddr(req.0 + 1).ceil(),
                VirtAddr(0x64000000).into(),
                MapPermission::U | MapPermission::W | MapPermission::R,
            );
            let resp_area = MapArea::linear(
                resp.floor()..PhysAddr(resp.0 + 1).ceil(),
                VirtAddr(0x64001000).into(),
                MapPermission::U | MapPermission::W | MapPermission::R,
            );
            proc.mset.push(req_area, None);
            proc.mset.push(resp_area, None);
            unsafe {
                riscv::asm::sfence_vma_all();
            }
            tf.x[10] = 0x64000000;
            tf.x[11] = 0x64001000;
        }
        0x100 => {
            // Sync call putchar
            uprint!("{}", tf.x[11] as u8 as char);
        }
        _ => {}
    }

    tf.sepc += 4;
}
