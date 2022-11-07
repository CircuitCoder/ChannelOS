#![feature(naked_functions, asm_const, alloc_error_handler, step_trait)]
#![no_std]
#![no_main]

extern crate alloc;

mod boot;
mod consts;
mod elf;
mod lang_items;
mod mem;
mod process;
mod provided;
mod sbi;
mod sched;
mod serial;
mod timer;
mod trap;
mod prog;
mod service;

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
    serial::sbi_print("Early print initialized\n");

    trap::init();
    mem::init();
    timer::init();

    let init = process::Process::new_user(prog::TEST, [0, 0], Default::default());
    sched::push(init);

    // Make sure nothing is on stack
    sched::bootstrap();
}