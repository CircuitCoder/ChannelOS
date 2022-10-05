#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::mprintln!("MeowSBI Panic: {:?}", info).unwrap();
    loop {}
}

#[no_mangle]
fn abort() -> ! {
    panic!("abort called");
}