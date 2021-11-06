/* Linker script for the STM32F407VGT6 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 1M
  RAM : ORIGIN = 0x20000000, LENGTH = 128K  /* 192k - 64k for stack */
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
