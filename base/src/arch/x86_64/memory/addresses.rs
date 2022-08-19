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
/// The Raw Address struct for `x86_64`
pub struct RawAddress {
    address: AddressWithFlags,
}

impl RawAddress {
    /// The address mask
    pub const ADDRESS_MASK: usize = 0x000F_FFFF_FFFF_F000;

    /// Create a new raw address
    pub fn new(ptr: u64) -> Result<Self, AddressError> {
        if is_address_canonical(
            ptr.try_into()
                .map_err(|_| AddressError::Generic(GenericError::IntConversionError))?,
            48,
        ) {
            Ok(Self {
                address: unsafe { AddressWithFlags::from_bits_unchecked(ptr) },
            })
        } else {
            Err(AddressError::AddressNonCanonical)
        }
    }

    /// Get the contained address
    pub const fn get_address_raw(self) -> u64 {
        self.address.bits()
    }

    /// Get the inner flag type as a reference
    pub const fn get_flags(&self) -> &AddressWithFlags {
        &self.address
    }

    /// Get the inner flag type as a mutable referece
    pub fn get_flags_mut(&mut self) -> &mut AddressWithFlags {
        &mut self.address
    }

    /// Bits 39-47
    pub const fn p4_index(self) -> usize {
        (TryInto::<usize>::try_into(self.address.bits()).unwrap_or(0) >> 39) & 0x1FF
    }

    /// Bits 30-38
    pub const fn p3_index(self) -> usize {
        (TryInto::<usize>::try_into(self.address.bits()).unwrap_or(0) >> 30) & 0x1FF
    }

    /// Bits 21-29
    pub fn p2_index(self) -> usize {
        (TryInto::<usize>::try_into(self.address.bits()).unwrap_or(0) >> 21) & 0x1FF
    }

    /// Bits 12-20
    pub const fn p1_index(self) -> usize {
        (TryInto::<usize>::try_into(self.address.bits()).unwrap_or(0) >> 12) & 0x1FF
    }

    /// Bits 0-29
    pub const fn level_2_huge_offset(self) -> usize {
        TryInto::<usize>::try_into(self.address.bits()).unwrap_or(0) & 0x3FFF_FFFF
    }

    /// Bits 0-20
    pub const fn level_1_huge_offset(self) -> usize {
        TryInto::<usize>::try_into(self.address.bits()).unwrap_or(0) & 0xF_FFFF
    }

    /// Bits 0-11
    pub const fn frame_offset(self) -> usize {
        TryInto::<usize>::try_into(self.address.bits()).unwrap_or(0) & 0xFFFF
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
    type AddressType = Self;

    type UnderlyingType = u64;

    fn new_address(addr: Self::UnderlyingType) -> Result<Self::AddressType, AddressError> {
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
