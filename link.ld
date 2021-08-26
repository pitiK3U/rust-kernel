/* Link.ld -- Linker script for the kernel - ensure everything goes in the */
/*            Correct place.  */
/*            Original file taken from Bran's Kernel Development */
/*            tutorials: http://www.osdever.net/bkerndev/index.php. */

ENTRY(start)
SECTIONS
{
  . = 1M;

  .text :
  {
     code = .; _code = .; __code = .;
     *(.multiboot)
     *(.text)
     . = ALIGN(4096);
  }

  .rodata :
  {
     rodata = .; _rodata = .; __rodata = .;
     *(.rodata)
     . = ALIGN(4096);
  }

  .data :
  {
     data = .; _data = .; __data = .;
     *(.data)
     . = ALIGN(4096);
  }

  .bss :
  {
    bss = .; _bss = .; __bss = .;
    *(.bss)
    . = ALIGN(4096);
  }

  end = .; _end = .; __end = .;
}

