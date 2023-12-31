/* memory.x - Linker script for the STM32F103C8T6 */
MEMORY
{
  /* Flash memory begins at 0x80000000 and has a size of 64kB*/
  /* We keep the last 4K for persistency */
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K
  /* RAM begins at 0x20000000 and has a size of 20kB*/
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}

