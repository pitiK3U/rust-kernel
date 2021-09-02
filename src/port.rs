pub struct Port {
    port: u16,
}

impl Port {
    pub fn new(port: u16) -> Self {
        Self {
            port: port,
        }
    }

    #[inline]
    pub fn write(&self, value: u32) {
        unsafe {
            asm!(
                "out dx, eax",
                in("dx") self.port,
                in("eax") value,
                options(nostack, preserves_flags, nomem),
            );
        }
    }

    #[inline]
    pub fn write_byte(&self, value: u8) {
        unsafe {
            asm!(
                "out dx, al",
                in("dx") self.port,
                in("al") value,
                options(nostack, preserves_flags, nomem),
            );
        }
    }
}

