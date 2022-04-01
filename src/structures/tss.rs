use crate::{
    collections::BitSlice,
    memory::paging::addresses::{Address, Virtual},
};

/// Task State Segment structure
#[repr(C, packed)]
// Page 373
pub struct TaskStateSegment {
    reserved1: u32,
    /// The RSPs to load when jumping to a specific level
    pub rsp: [Address<Virtual>; 3],
    reserved2: u64,
    /// What is this, even?
    pub ists: [Address<Virtual>; 7],
    reserved3: u64,
    reserved4: u16,
    /// Where the IOPB is situated from the base of the tss
    pub iopb_offset: u16,
}

impl TaskStateSegment {
    /// Base size In bytes?
    pub const BASE_LENGTH: u16 = 103;

    /// Create a new blank TSS with no ports
    pub fn new_blank() -> Self {
        Self {
            reserved1: 0,
            rsp: [Address::<Virtual>::new(core::ptr::null()).unwrap(); 3],
            reserved2: 0,
            ists: [Address::<Virtual>::new(core::ptr::null()).unwrap(); 7],
            reserved3: 0,
            reserved4: 0,
            iopb_offset: Self::BASE_LENGTH + 1,
        }
    }

    /// Create a new TSS with no ports
    pub fn new_no_ports(
        rsp0: Address<Virtual>,
        rsp1: Address<Virtual>,
        rsp2: Address<Virtual>,
    ) -> Self {
        Self {
            reserved1: 0,
            rsp: [rsp0, rsp1, rsp2],
            reserved2: 0,
            ists: [Address::<Virtual>::new(core::ptr::null()).unwrap(); 7],
            reserved3: 0,
            reserved4: 0,
            iopb_offset: Self::BASE_LENGTH + 1,
        }
    }
}

/// Width of the IO port
#[repr(u8)]
pub enum IOWidth {
    /// One byte wide
    Single = 1,
    /// Two bytes wide
    Double = 2,
    /// Four bytes wide
    Quad = 4,
}

/// The I/O Permissions bitmap
#[repr(packed)]
pub struct IOPB<'a>(&'a [u32]);

impl<'a> IOPB<'a> {
    /// A new iopb from the specified data
    ///
    /// # Arguments
    /// * `data` - The slice of double words to use as data
    pub fn new(data: &'a [u32]) -> Self {
        Self(data)
    }

    /// Set a specified port with a size to usable
    ///
    /// # Arguments
    /// * `port` - The port to access
    /// * `width` - The width of the port to set
    /// * `usable` - Whether it's usable or not
    pub fn set_port(&mut self, port: usize, width: IOWidth, usable: bool) {
        let mut slice = BitSlice::new();
        unsafe { slice.new_from_init(self.0.as_ptr() as *mut u8, self.0.len() * 4) };

        for i in port..(port + width as u8 as usize) {
            slice.set(i, usable);
        }
    }
}
