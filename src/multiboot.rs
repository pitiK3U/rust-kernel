#[used]
#[link_section = ".multiboot"]
pub static MAGIC: u32 = 0x1BADB002;
#[link_section = ".multiboot"]
pub static FLAGS: u32 = 1 << 1 | 1 << 0;
#[link_section = ".multiboot"]
pub static CHECKSUM: u32 = (-((MAGIC + FLAGS) as i32) as u32);

#[link_section = ".multiboot"]
asm!(".set ALIGN,    1<<0             /* align loaded modules on page boundaries */
.set MEMINFO,  1<<1             /* provide memory map */
.set FLAGS,    ALIGN | MEMINFO  /* this is the Multiboot 'flag' field */
.set MAGIC,    0x1BADB002       /* 'magic number' lets bootloader find the header */
.set CHECKSUM, -(MAGIC + FLAGS) /* checksum of above, to prove we are multiboot */"
);
