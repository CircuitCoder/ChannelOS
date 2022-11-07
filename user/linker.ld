OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
  . = 0x1000;
  . = ALIGN(0x1000);
  .text : {
    *(.text .text.*)
  }

  . = ALIGN(0x1000);
  .dynstr : {
    *(.dynstr .dynstr.*)
  }

  . = ALIGN(0x1000);
  .dynsym : {
    *(.dynsym .dynsym.*)
  }

  . = ALIGN(0x1000);
  .gnu.hash : {
    *(.gnu.hash .gnu.hash.*)
  }

  
  . = ALIGN(0x1000);
  .rodata : {
    *(.rodata .rodata.*)
  }

  . = ALIGN(0x1000);
  .sdata : {
    *(.sdata .sdata.*)
  }

  . = ALIGN(0x1000);
  .data : {
    *(.data .data.*)
  }

  . = ALIGN(0x1000);
  .bss : {
    *(.bss .bss.*)
  }

  . = ALIGN(0x1000);
  .sbss : {
    *(.sbss .sbss.*)
  }

  . = ALIGN(0x1000);
  .dynamic : {
    *(.dynamic .dynamic.*)
  }

  . = ALIGN(0x1000);
  .rela.dyn : {
    *(.rela.dyn .rela.dyn.*)
  }

  . = ALIGN(0x1000);
  .plt : {
    *(.plt .plt.*)
  }

  . = ALIGN(0x1000);
  .got : {
    *(.got .got.*)
  }

  . = ALIGN(0x1000);
  .interp : {
    *(.interp .interp.*)
  }
}
