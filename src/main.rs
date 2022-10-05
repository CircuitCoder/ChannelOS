#![feature(naked_functions)]
#![cfg_attr(not(test), no_std)]
#![no_main]

mod boot;
mod sbi;
mod serial;
mod lang_items;

extern "C" {
    fn _fw_start();
    fn _fw_end();
    fn _bss_start();
    fn _bss_end();
}

#[no_mangle]
fn boot(hartid: usize, fdt_addr: usize) {
    serial::early_serial_init();
    mprintln!("Hello world!");
    loop { }
}