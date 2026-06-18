OUTPUT_ARCH(aarch64)
ENTRY(_start)

LOAD_ADDR = 0x02080000;

SECTIONS
{
  . = LOAD_ADDR;

  .text : ALIGN(4K) {
    KEEP(*(.text.boot))
    *(.text .text.*)
  }

  .rodata : ALIGN(4K) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(4K) {
    *(.data .data.*)
  }

  __image_end = .;

  .bss (NOLOAD) : ALIGN(4K) {
    __bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    . = ALIGN(16);
    __bss_end = .;
  }

  .stack (NOLOAD) : ALIGN(16) {
    _boot_stack_bottom = .;
    . += 64K;
    _boot_stack_top = .;
  }

  /DISCARD/ : {
    *(.comment)
    *(.eh_frame*)
    *(.note*)
    *(.gnu*)
  }
}
