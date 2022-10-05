OUTPUT_ARCH(riscv)
ENTRY(entry)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
  . = BASE_ADDRESS;
  PROVIDE(_fw_start = .);

  .text : {
    *(.text.entry)
    .ALIGN = 0x4;
    *(.text.trap)
    *(.text .text.*)
  }

  .rodata : {
    *(.rodata .rodata.*)
    *(.rodata.payload)
  }

  .sdata : {
    *(.sdata .sdata.*)
  }

  .data : {
    *(.data .data.*)
  }

  PROVIDE(_bss_start = .);
  .bss : {
    *(.bss .bss.*)
  }

  .sbss : {
    *(.sbss .sbss.*)
  }
  PROVIDE(_bss_end = .);

  PROVIDE(_fw_end = .);
}
