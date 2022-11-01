file target/riscv64gc-unknown-none-elf/debug/channel_os
target remote localhost:1234
# layout asm
break channel_os::process::kickoff_process
