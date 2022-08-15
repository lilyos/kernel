use core::fmt::Debug;

use crate::{
    errors::{AddressError, GenericError},
    macros::bitflags::bitflags,
    memory::utilities::is_address_canonical,
    traits::PlatformAddress,
};

bitflags! {
    #[derive(Debug)]
    pub struct AddressWithFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH_CACHING = 1 << 3;
        const DISABLE_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        // 9-11 Free Use
        // 52-62 Free Use
        const NO_EXECUTE = 1 << 63;
    }
}

#[derive(Clone, Copy)]
/// The Raw Address struct for x86_64
pub struct RawAddress {
    address: AddressWithFlags,
}

impl RawAddress {
    /// The address mask
    pub const ADDRESS_MASK: usize = 0x000F_FFFF_FFFF_F000;

    /// Create a new raw address
    pub fn new(ptr: u64) -> Result<Self, AddressError> {
        if !is_address_canonical(
            ptr.try_into()
                .map_err(|_| AddressError::Generic(GenericError::IntConversionError))?,
            48,
        ) {
            Err(AddressError::AddressNonCanonical)
        } else {
            Ok(Self {
                address: unsafe {
                    AddressWithFlags::from_bits_unchecked(
                        (ptr as usize)
                            .try_into()
                            .map_err(|_| AddressError::ConversionError)?,
                    )
                },
            })
        }
    }

    /// Get the contained address
    pub fn get_address_raw(&self) -> u64 {
        self.address.bits()
    }

    /// Get the inner flag type as a reference
    pub fn get_flags(&self) -> &AddressWithFlags {
        &self.address
    }

    /// Get the inner flag type as a mutable referece
    pub fn get_flags_mut(&mut self) -> &mut AddressWithFlags {
        &mut self.address
    }

    /// Bits 39-47
    pub fn p4_index(&self) -> usize {
        (self.address.bits() as usize >> 39) & 0x1FF
    }

    /// Bits 30-38
    pub fn p3_index(&self) -> usize {
        (self.address.bits() as usize >> 30) & 0x1FF
    }

    /// Bits 21-29
    pub fn p2_index(&self) -> usize {
        (self.address.bits() as usize >> 21) & 0x1FF
    }

    /// Bits 12-20
    pub fn p1_index(&self) -> usize {
        (self.address.bits() as usize >> 12) & 0x1FF
    }

    /// Bits 0-29
    pub fn level_2_huge_offset(&self) -> usize {
        self.address.bits() as usize & 0x3FFF_FFFF
    }

    /// Bits 0-20
    pub fn level_1_huge_offset(&self) -> usize {
        self.address.bits() as usize & 0xF_FFFF
    }

    /// Bits 0-11
    pub fn frame_offset(&self) -> usize {
        self.address.bits() as usize & 0xFFFF
    }
}

impl Debug for RawAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualAddress")
            .field("Address", &format_args!("{:#x}", &self.get_address_raw()))
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

impl PlatformAddress for RawAddress {
    type Error = AddressError;

    type AddressType = RawAddress;

    type UnderlyingType = u64;

    fn new_address(addr: Self::UnderlyingType) -> Result<Self::AddressType, Self::Error> {
        Self::new(addr)
    }

    fn address_valid<T>(addr: crate::memory::addresses::Address<T>) -> bool {
        if let Ok(addr) = addr.inner().address.bits().try_into() {
            is_address_canonical(addr, 48)
        } else {
            false
        }
    }

    fn into_raw(self) -> Self::UnderlyingType {
        self.get_address_raw()
    }
}
