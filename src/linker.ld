OUTPUT_ARCH(riscv)
ENTRY(entry)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
  . = BASE_ADDRESS;
  PROVIDE(_kernel_start = .);

  . = ALIGN(0x1000);
  PROVIDE(_text_start = .);
  .text : {
    *(.text.entry)
    .ALIGN = 0x4;
    *(.text.trap)
    *(.text)

    . = ALIGN(0x1000);
    PROVIDE(_text_vdso_start = .);
    *(.text.vdso)
    PROVIDE(_text_vdso_end = .);
  }
  
  . = ALIGN(0x1000);
  PROVIDE(_text_end = .);
  PROVIDE(_ro_start = .);

  .rodata : {
    *(.rodata .rodata.*)
    *(.rodata.payload)
  }

  . = ALIGN(0x1000);
  PROVIDE(_ro_end = .);
  PROVIDE(_rw_start = .);

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

  . = ALIGN(0x1000);
  PROVIDE(_bss_end = .);
  PROVIDE(_rw_end = .);
  PROVIDE(_kernel_end = .);
  PROVIDE(_frames_start = .);
}
