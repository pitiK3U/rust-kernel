use bit_field::BitField;

pub type HandlerFunc = extern "C" fn() -> !;

#[derive(PartialEq, Eq)]
pub enum TableIndex {
    /// Global Descriptor Table
    Gdt = 0,
    /// Local Descriptor Table
    Ldt = 1,
}

/// Selectors are called the segment registers - CS, DS, ES, FS, GS - in protected mode, which are index registers holding an index pointer into a table (GDT, LDT), in other words holding indexes to a Descriptor.
///
/// 15 ... 3   2   1  0
/// +-------+----+--+--+
/// | Index | TI | RPL |
/// +-------+----+--+--+
///
/// `RPL` - Requested Privilege Level. The CPU checks these bits before any selector is changed. Also system calls can be executed in userspace (ring 3, see this) without misfeasance using the ARPL (Adjust Requested Privilege Level) instruction. 
///
/// `TI` - Table index; 0 = GDT, 1 = LDT
///
/// `Index` - Index to a Descriptor of the table.
#[derive(Debug, Clone, Copy)]
pub struct Selector(u16);

impl Selector {
    pub const fn new() -> Self {
        Self(0)
    }

    // TODO:
    pub fn set_rpl(&mut self, rpl: u8) -> &mut Self {
        self
    }

    pub fn set_table_index(&mut self, table_index: TableIndex) -> &mut Self {
        self.0.set_bit(2, table_index == TableIndex::Ldt);
        self
    }

    // TODO:
    pub fn set_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(3..=15, index);
        self
    }
}

fn get_code_segment() -> u16 {
    let mut result: u16;
    unsafe {
    asm!(
        "mov {:x}, cs",
        out(reg) result,
    );
    }
    result
}

pub mod IDT {
    use super::*;

    static IDT: Idt = {
        let mut idt = Idt::new();

        idt
    };

    pub fn init() {
        IDT.load();
    }

    pub struct Idt([Entry; 16]);

    impl Idt {
        const fn new() -> Self {
            Self([Entry::missing(); 16])
        }

        fn set_hadler(&mut self, entry: u8, handler: HandlerFunc) -> &mut TypeAttribute {
            let mut selector = Selector::new();
            let selector = selector.set_index(get_code_segment());
            self.0[entry as usize] = Entry::new(*selector , handler);
            &mut self.0[entry as usize].type_attribute
        }

        fn load(&self) {
            let ptr = Dpt {
                base: self as *const _ as usize,
                size: (core::mem::size_of::<Self>() - 1) as u16,
            };

            unsafe {
            asm!(
                "lidt [{}]",
                in(reg) &ptr,
            );
            }
        }
    }

    #[repr(C,packed)]
    struct Dpt {
        size: u16,
        base: usize,
    }

    #[derive(Debug, Copy, Clone)]
    #[repr(C, packed)]
    pub struct Entry {
        ///  Lower part of the interrupt function's offset address (also known as pointer).
        offset_lower: u16,
        ///  Selector of the interrupt function (to make sense - the kernel's selector). The selector's descriptor's DPL field has to be 0 so the iret instruction won't throw a #GP exeption when executed.
        selector: Selector,
        /// Must be `0`.
        zero: u8,
        /// Types and attributes.
        type_attribute: TypeAttribute,
        ///  Higher part of the offset.
        offset_higher: u16,
    }

    impl Entry {
        pub fn new(selector: Selector, handler: HandlerFunc) -> Self {
            let pointer = handler as usize;
            Entry {
                selector: selector,
                offset_lower: pointer as u16,
                offset_higher: (pointer >> 16) as u16,
                type_attribute: TypeAttribute::new(),
                zero: 0,
            }
        }

        const fn missing() -> Self {
            Entry {
                selector: Selector::new(),
                offset_lower: 0,
                offset_higher: 0,
                type_attribute: TypeAttribute::new(),
                zero: 0,
            }
        }
    }

    /// 4 bit value
    enum GateType {
        /// 80386 32 bit task gate
        Task32 =  0b0101,
        /// 80286 16-bit interrupt gate
        Interrupt16 = 0b0110,
        /// 80286 16-bit trap gate
        Trap16 = 0b0111,
        /// 80386 32-bit interrupt gate
        Interrupt32 = 0b1110,
        /// 80386 32-bit trap gate
        Trap32 = 0b1111,
    }

    // Descriptor Privilage Level
    enum Dpl {
        /// Typically kernel.
        High   = 0b00,
        Medium = 0b01,
        /// Typically userland.
        Low    = 0b11,
    }

    #[derive(Clone, Copy, Debug)]
    struct TypeAttribute(u8);

    impl TypeAttribute {
        const fn new() -> Self {
            Self(0)
        }

        fn set_present(&mut self, present: bool) -> &mut Self {
            self.0.set_bit(7, present);
            self
        }

        fn set_dpl(&mut self, dpl: Dpl) -> &mut Self {
            self.0.set_bits(5..=6, dpl as u8);
            self
        }

        fn set_gate(&mut self, gate: GateType) -> &mut Self {
            self.0.set_bits(0..=4, gate as u8);
            self
        }
    }

}