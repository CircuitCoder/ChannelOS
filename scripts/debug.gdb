file target/riscv64gc-unknown-none-elf/debug/channel_os
target remote localhost:1234
layout asm
break trap_entry
break trap_exit
c
