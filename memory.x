MEMORY
{
  /**
   * Microchip [ATSAME54P] Cortex-M4 microcontroller @ 120 MHz
   *
   *  1MB Flash
   *  256kB SRAM
   *  8MB SPI Flash chip
   *d
   *  https://github.com/atsamd-rs/atsamd/blob/master/boards/atsame54_xpro/memory.x
   */
  FLASH (rx)  : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH =  256K
}
