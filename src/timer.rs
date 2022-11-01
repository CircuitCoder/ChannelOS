use crate::sbi::set_timer;
use riscv::register::{sie, time};

pub static mut TICKS: usize = 0;

// TODO: Allow changing TIMEBASE
static TIMEBASE: usize = 10_000_000;
pub fn init() {
    unsafe {
        TICKS = 0;
        sie::set_stimer();
    }
    rearm();
}

// TODO: Remove me
pub fn rearm() {
    let next = rtc() + TIMEBASE;
    set_timer(next);
}

pub fn rtc() -> usize {
    time::read() as usize
}

pub fn now() -> usize {
    rtc() / TIMEBASE
}

pub fn trigger() {
    crate::mprintln!("Timer triggered at {} ({})", now(), rtc());
    rearm();

    // TODO: dispatch to scheduler
}
