#![feature(naked_functions, asm_const)]
#![no_std]
#![no_main]

mod boot;
mod sbi;
mod serial;
mod lang_items;
mod trap;
mod timer;

extern "C" {
    fn _fw_start();
    fn _fw_end();
    fn _bss_start();
    fn _bss_end();
}

#[no_mangle]
fn boot(hartid: usize, fdt_addr: usize) {
    serial::early_serial_init();
    serial::sbi_print("Test SBI\n");
    mprintln!("Hello world!").unwrap();

    trap::init();
    timer::init();
    loop {
        crate::trap::wfi();
    }
}