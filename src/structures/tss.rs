use crate::{collections::BitSlice, memory::paging::VirtualAddress};

#[repr(C, packed)]
// Page 373
pub struct TaskStateSegment {
    reserved1: u32,
    /// The stack pointer to load when jumping to ring 0
    pub rsp0: VirtualAddress,
    /// The stack pointer to load when jumping to ring 1
    pub rsp1: VirtualAddress,
    /// The stack pointer to load when jumping to ring 2
    pub rsp2: VirtualAddress,
    reserved2: u64,
    /// Honestly idk
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    reserved3: u64,
    reserved4: u16,
    /// Where the IOPB is situated from the base of the tss
    pub iopb_offset: u16,
}

impl TaskStateSegment {
    // In bytes?
    pub const BASE_LENGTH: u16 = 103;

    pub fn new_no_ports(rsp0: VirtualAddress, rsp1: VirtualAddress, rsp2: VirtualAddress) -> Self {
        Self {
            reserved1: 0,
            rsp0,
            rsp1,
            rsp2,
            reserved2: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved3: 0,
            reserved4: 0,
            iopb_offset: Self::BASE_LENGTH + 1,
        }
    }
}

#[repr(u8)]
pub enum IOWidth {
    Single = 1,
    Double = 2,
    Quad = 4,
}

/// The I/O Permissions bitmap
#[repr(packed)]
pub struct IOPB<'a>(&'a [u32]);

impl<'a> IOPB<'a> {
    pub fn new(data: &'a [u32]) -> Self {
        Self(data)
    }

    pub fn set_port(&mut self, port: usize, width: IOWidth, usable: bool) {
        let mut slice = BitSlice::new();
        unsafe { slice.new_from_init(self.0.as_ptr() as *mut u8, self.0.len() * 4) };

        for i in port..(port + width as u8 as usize) {
            slice.set(i, usable);
        }
    }
}
