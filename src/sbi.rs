use core::result::Result;
use Result::*;

enum SPIFunc {
    SetTimer,
    ConsolePutchar,
}

impl SPIFunc {
    fn id(&self) -> (usize, usize) {
        match self {
            SPIFunc::SetTimer => (0x54494D45, 0),
            SPIFunc::ConsolePutchar => (1, 0),
        }
    }
}

fn send(func: SPIFunc, mut param0: usize, mut param1: usize) -> Result<usize, usize> {
    let (eid, fid) = func.id();
    let mut err: usize;
    let mut val : usize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inout("a0") param0,
            inout("a1") param1,
        );
    }

    if param0 == 0 {
        Ok(param1)
    } else {
        Err(param0)
    }
}

pub fn console_putchar(c: u8) {
    send(SPIFunc::ConsolePutchar, c as usize, 0);
}