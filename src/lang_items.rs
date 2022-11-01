#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::mprintln!("ChannelOS Panic: {:?}", info);
    loop {}
}

#[no_mangle]
fn abort() -> ! {
    panic!("abort called");
}