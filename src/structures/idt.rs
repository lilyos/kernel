#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
/// Interrupt descriptor, entry in Interrupt Descriptor Table
pub struct InterruptDescriptor {
    /// Offset bits 0-15
    pub offset_1: u16,
    /// Selector into Global Descriptor Table (GDT) or Local Descriptor Table (LDT)
    pub selector: u16,
    /// Interrupt stack table offset, 0-2 used, rest are zeroed
    pub ist: u8,
    /// Gate type, DPL, and P fields
    pub type_attributes: u8,
    /// Offset bits 16-31
    pub offset_2: u16,
    /// Offset bits 32-63
    pub offset_3: u32,
    /// Reserved
    reserved: u32,
}

impl InterruptDescriptor {
    const INTERRUPT_GATE: u8 = 0xE;
    const TRAP_GATE: u8 = 0xF;

    pub fn zeroed() -> Self {
        Self {
            offset_1: 0,
            selector: 0,
            ist: 8,
            type_attributes: 0,
            offset_2: 0,
            offset_3: 0,
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
/// The Interrupt Descriptor Table. It holds 256 entries, from 0-255
pub struct InterruptDescriptorTable {
    pub inner: [InterruptDescriptor; 256],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            inner: [InterruptDescriptor::zeroed(); 256],
        }
    }
}
