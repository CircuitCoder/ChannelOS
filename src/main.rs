#![feature(naked_functions, asm_const, alloc_error_handler, step_trait)]
#![no_std]
#![no_main]

extern crate alloc;

mod consts;
mod boot;
mod sbi;
mod serial;
mod lang_items;
mod trap;
mod timer;
mod mem;
mod process;
mod elf;
mod provided;
mod sched;

#[link_section = ".data"]
#[no_mangle]
pub static mut INIT_STACK: [u8; consts::KERNEL_STACK_SIZE] = [0; consts::KERNEL_STACK_SIZE];

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
    mprintln!("Hello world!");

    trap::init();
    mem::init();
    timer::init();

    let init = process::Process::new_user(process::TEST_PROGRAM);
    // Make sure nothing is on stack
    sched::bootstrap(init);
}