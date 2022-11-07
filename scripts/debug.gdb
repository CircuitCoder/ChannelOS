file target/riscv64gc-unknown-none-elf/release/channel_os
target remote localhost:1234
# layout asm
break channel_os::service::putchar_kboot
