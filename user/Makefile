.PHONY: all clean

all: test.elf

clean:
	rm -rf test.elf stub.so

stub.so: stub.c
	riscv64-linux-gnu-gcc -shared -o stub.so -fPIC -nostdlib stub.c

test.elf: test.c linker.ld stub.so
	riscv64-linux-gnu-gcc -Tlinker.ld -o test.elf -nostartfiles -nostdlib test.c -L. -l:stub.so -Wl,--build-id=none -Wl,--no-omagic
