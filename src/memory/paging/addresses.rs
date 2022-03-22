use core::marker::PhantomData;

use kernel_macros::bit_field_accessors;

use crate::memory::allocators::{align_down, is_address_canonical};

/// Enum representing that something contains a virtual address
pub enum Virtual {}

/// Enum representing that something contains a physical address
pub enum Physical {}

/// Struct representing an address
pub struct Address<T>(usize, PhantomData<T>);

impl<T> Address<T> {
    /// The address mask
    pub const ADDRESS_MASK: usize = 0x000F_FFFF_FFFF_F000;

    bit_field_accessors! {
        present 0;
        writable 1;
        user_accessible 2;
        write_through_caching 3;
        disable_cache 4;
        accessed 5;
        dirty 6;
        huge_page 7;
        global 8;
        // 9-11 Free Use
        reserved 9;
        // 52-62 Free Use
        no_execute 63;
    }
}

impl Address<Virtual> {
    /// Create a new virtual address
    pub fn new(address: *const u8) -> Result<Self, AddressError> {
        if !is_address_canonical(address as usize, 48) {
            Err(AddressError::AddressNonCanonical)
        } else {
            Ok(Self(address as usize, PhantomData))
        }
    }

    /// Page align an address by truncating the spare bytes
    pub fn align_lossy(&self) -> AlignedAddress<Virtual> {
        AlignedAddress(align_down(self.0, 4096), PhantomData)
    }

    /// Get the inner value
    pub fn get_inner(&self) -> usize {
        self.0
    }

    /// Get an immutable pointer for the address
    fn get_address(&self) -> *const u8 {
        (self.0 & Self::ADDRESS_MASK) as *const u8
    }

    /// Get a mutable pointer for the address
    fn get_address_mut(&mut self) -> *mut u8 {
        (self.0 & Self::ADDRESS_MASK) as *mut u8
    }

    /// Bits 39-47
    pub fn p4_index(&self) -> usize {
        (self.0 as usize >> 39) & 0x1FF
    }

    /// Bits 30-38
    pub fn p3_index(&self) -> usize {
        (self.0 as usize >> 30) & 0x1FF
    }

    /// Bits 21-29
    pub fn p2_index(&self) -> usize {
        (self.0 as usize >> 21) & 0x1FF
    }

    /// Bits 12-20
    pub fn p1_index(&self) -> usize {
        (self.0 as usize >> 12) & 0x1FF
    }

    /// Bits 0-29
    pub fn level_2_huge_offset(&self) -> usize {
        self.0 & 0x3FFF_FFFF
    }

    /// Bits 0-20
    pub fn level_1_huge_offset(&self) -> usize {
        self.0 & 0xF_FFFF
    }

    /// Bits 0-11
    pub fn frame_offset(&self) -> usize {
        self.0 as usize & 0xFFFF
    }
}

impl Clone for Address<Physical> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl Clone for Address<Virtual> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl core::fmt::Debug for Address<Virtual> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualAddress")
            .field("Address", &self.get_address())
            .field("Level4Index", &self.p4_index())
            .field("Level3Index", &self.p3_index())
            .field("Level2Index", &self.p2_index())
            .field("Level1Index", &self.p1_index())
            .field("Level2HugeOffset", &self.level_2_huge_offset())
            .field("Level1HugeOffset", &self.level_1_huge_offset())
            .field("FrameOffset", &self.frame_offset())
            .finish()
    }
}

impl TryFrom<*mut u8> for Address<Virtual> {
    type Error = AddressError;

    fn try_from(value: *mut u8) -> Result<Self, Self::Error> {
        Address::<Virtual>::new(value as *const u8)
    }
}

impl TryFrom<*const u8> for Address<Virtual> {
    type Error = AddressError;

    fn try_from(value: *const u8) -> Result<Self, Self::Error> {
        Address::<Virtual>::new(value)
    }
}

impl core::fmt::Debug for Address<Physical> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhysicalAddress")
            .field("Address", &format_args!("0x{:x}", self.get_address()))
            .finish()
    }
}

impl Address<Physical> {
    /// Create a new physical address
    pub fn new(address: usize) -> Self {
        Self(address, PhantomData)
    }

    /// Page align an address by truncating the spare bytes
    pub fn align_lossy(&self) -> AlignedAddress<Physical> {
        AlignedAddress(align_down(self.0, 4096), PhantomData)
    }

    /// Get the inner value
    pub fn get_inner(&self) -> usize {
        self.0
    }

    /// Get the address as a usize
    pub fn get_address(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
/// Errors that occur when trying to convert an address between types
pub enum AddressError {
    /// The address wasn't aligned
    AddressNotAligned,
    /// The address wasn't canonical
    AddressNonCanonical,
    /// An unspecified error occurred
    Other,
}

/// Struct representing an aligned address
pub struct AlignedAddress<T>(usize, PhantomData<T>);

impl Clone for AlignedAddress<Physical> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl Clone for AlignedAddress<Virtual> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> AlignedAddress<T> {
    /// The address mask
    pub const ADDRESS_MASK: usize = 0x000F_FFFF_FFFF_F000;

    bit_field_accessors! {
        present 0;
        writable 1;
        user_accessible 2;
        write_through_caching 3;
        disable_cache 4;
        accessed 5;
        dirty 6;
        huge_page 7;
        global 8;
        // 9-11 Free Use
        reserved 9;
        // 52-62 Free Use
        no_execute 63;
    }
}

impl AlignedAddress<Virtual> {
    /// Page align an address by truncating the spare bytes
    pub fn align_lossy(&self) -> AlignedAddress<Virtual> {
        AlignedAddress(align_down(self.0, 4096), PhantomData)
    }

    /// Get the inner value
    pub fn get_inner(&self) -> usize {
        self.0
    }

    /// Get an immutable pointer for the address
    fn get_address(&self) -> *const u8 {
        (self.0 & Self::ADDRESS_MASK) as *const u8
    }

    /// Get a mutable pointer for the address
    fn get_address_mut(&mut self) -> *mut u8 {
        (self.0 & Self::ADDRESS_MASK) as *mut u8
    }

    /// Bits 39-47
    pub fn p4_index(&self) -> usize {
        (self.0 as usize >> 39) & 0x1FF
    }

    /// Bits 30-38
    pub fn p3_index(&self) -> usize {
        (self.0 as usize >> 30) & 0x1FF
    }

    /// Bits 21-29
    pub fn p2_index(&self) -> usize {
        (self.0 as usize >> 21) & 0x1FF
    }

    /// Bits 12-20
    pub fn p1_index(&self) -> usize {
        (self.0 as usize >> 12) & 0x1FF
    }

    /// Bits 0-29
    pub fn level_2_huge_offset(&self) -> usize {
        self.0 & 0x3FFF_FFFF
    }

    /// Bits 0-20
    pub fn level_1_huge_offset(&self) -> usize {
        self.0 & 0xF_FFFF
    }

    /// Bits 0-11
    pub fn frame_offset(&self) -> usize {
        self.0 as usize & 0xFFFF
    }
}

impl AlignedAddress<Physical> {
    /// Get the inner value
    pub fn get_inner(&self) -> usize {
        self.0
    }

    /// Get the address as a usize
    pub fn get_address(&self) -> usize {
        self.0 & Self::ADDRESS_MASK
    }
}

impl TryFrom<Address<Virtual>> for AlignedAddress<Virtual> {
    type Error = AddressError;

    fn try_from(value: Address<Virtual>) -> Result<Self, Self::Error> {
        let addr = value.get_address() as usize;
        if addr % 4096 != 0 {
            Err(AddressError::AddressNotAligned)
        } else {
            Ok(AlignedAddress(addr, PhantomData))
        }
    }
}

impl TryFrom<Address<Physical>> for AlignedAddress<Physical> {
    type Error = AddressError;

    fn try_from(value: Address<Physical>) -> Result<Self, Self::Error> {
        let addr = value.get_address() as usize;
        if addr % 4096 != 0 {
            Err(AddressError::AddressNotAligned)
        } else {
            Ok(AlignedAddress(addr, PhantomData))
        }
    }
}

impl core::fmt::Debug for AlignedAddress<Virtual> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualAlignedAddress")
            .field("Address", &self.get_address())
            .field("Level4Index", &self.p4_index())
            .field("Level3Index", &self.p3_index())
            .field("Level2Index", &self.p2_index())
            .field("Level1Index", &self.p1_index())
            .field("Level2HugeOffset", &self.level_2_huge_offset())
            .field("Level1HugeOffset", &self.level_1_huge_offset())
            .field("FrameOffset", &self.frame_offset())
            .finish()
    }
}

impl core::fmt::Debug for AlignedAddress<Physical> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhysicalAlignedAddress")
            .field("Address", &format_args!("0x{:x}", self.get_address()))
            .finish()
    }
}
