use core::{fmt::Debug, marker::PhantomData, ops::Deref};

use crate::{arch::memory::addresses::RawAddress, memory::utilities::align_down};

use super::errors::AddressError;

/// Enum representing that something contains a virtual address
#[derive(Clone, Copy, Debug)]
pub enum Virtual {}

/// Enum representing that something contains a physical address
#[derive(Clone, Copy, Debug)]
pub enum Physical {}

/// Struct representing an address
pub struct Address<T>(pub RawAddress<T>);

impl<T> Address<T> {
    /// Create a new ph

    /// Page align an address by truncating the spare bytes
    pub fn align_lossy(&self) -> AlignedAddress<Physical> {
        AlignedAddress(
            unsafe {
                RawAddress::new_unchecked(
                    align_down(self.0.get_address_raw() as usize, 4096) as *const ()
                )
            },
            PhantomData,
        )
    }

    /// Get the raw address as reference
    pub fn get_raw_address(&self) -> &RawAddress<T> {
        &self.0
    }

    /// Get the raw address as a mutable reference
    pub fn get_raw_address_mut(&mut self) -> &mut RawAddress<T> {
        &mut self.0
    }

    /// Get the inner value as a usize
    pub fn get_address_raw(&self) -> usize {
        self.0.get_address_raw() as usize
    }
}

impl<T> Deref for Address<T> {
    type Target = RawAddress<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for Address<Virtual> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.get_raw_address().fmt(f)
    }
}

impl Debug for Address<Physical> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.get_raw_address().fmt(f)
    }
}

impl Address<Virtual> {
    /// Create a new virtual address
    pub fn new(address: *const ()) -> Result<Self, AddressError> {
        Ok(Self(RawAddress::new(address as *const ())?))
    }

    /// Get the inner value as a pointer
    pub fn get_inner_ptr(&self) -> *const () {
        self.0.get_address_raw() as *const ()
    }

    /// Get the inner value as a mutable pointer
    pub fn get_inner_ptr_mut(&mut self) -> *mut () {
        self.0.get_address_raw() as *mut ()
    }
}

impl Clone for Address<Physical> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for Address<Physical> {}

impl Clone for Address<Virtual> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for Address<Virtual> {}

impl TryFrom<*mut u8> for Address<Virtual> {
    type Error = AddressError;

    fn try_from(value: *mut u8) -> Result<Self, Self::Error> {
        Address::<Virtual>::new(value as *const ())
    }
}

impl TryFrom<*const u8> for Address<Virtual> {
    type Error = AddressError;

    fn try_from(value: *const u8) -> Result<Self, Self::Error> {
        Address::<Virtual>::new(value as *const ())
    }
}

impl Address<Physical> {
    /// Create a new virtual address
    pub fn new(address: usize) -> Result<Self, AddressError> {
        Ok(Self(RawAddress::new(address as *const ())?))
    }

    /// Get the address as a usize
    pub fn get_address(&self) -> usize {
        self.0.get_address_raw() as usize
    }
}

/// Struct representing an aligned address
pub struct AlignedAddress<T>(RawAddress<T>, PhantomData<T>);

impl<T> Deref for AlignedAddress<T> {
    type Target = RawAddress<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for AlignedAddress<Physical> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl Copy for AlignedAddress<Physical> {}

impl Clone for AlignedAddress<Virtual> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl Copy for AlignedAddress<Virtual> {}

impl TryFrom<*mut u8> for AlignedAddress<Virtual> {
    type Error = AddressError;

    fn try_from(value: *mut u8) -> Result<Self, Self::Error> {
        Self::new(value as *const ())
    }
}

impl TryFrom<*const u8> for AlignedAddress<Virtual> {
    type Error = AddressError;

    fn try_from(value: *const u8) -> Result<Self, Self::Error> {
        Self::new(value as *const ())
    }
}

impl<T> TryFrom<Address<T>> for AlignedAddress<T> {
    type Error = AddressError;

    fn try_from(value: Address<T>) -> Result<Self, Self::Error> {
        Self::new(value.get_address_raw() as *const ())
    }
}

impl<T> AlignedAddress<T> {
    /// The address mask
    pub const ADDRESS_MASK: usize = 0x000F_FFFF_FFFF_F000;

    /// Get the raw address as a reference
    pub fn get_raw_address(&self) -> &RawAddress<T> {
        &self.0
    }

    /// Get the raw address as a mutable reference
    pub fn get_raw_address_mut(&mut self) -> &mut RawAddress<T> {
        &mut self.0
    }

    /// Get the inner value as a usize
    pub fn get_address_raw(&self) -> usize {
        self.0.get_address_raw() as usize
    }

    /// Try to form an aligned address from a usize
    fn new(addr: *const ()) -> Result<Self, AddressError> {
        let addr = addr as usize;
        if addr % 4096 != 0 {
            Err(AddressError::AddressNotAligned)
        } else {
            Ok(AlignedAddress(
                unsafe { RawAddress::new_unchecked(addr as *const ()) },
                PhantomData,
            ))
        }
    }
}

impl AlignedAddress<Virtual> {
    /// Get an immutable pointer for the address
    fn get_address(&self) -> *const () {
        self.0.get_address_raw() as *const ()
    }

    /// Get a mutable pointer for the address
    fn get_address_mut(&mut self) -> *mut () {
        self.0.get_address_raw() as *mut ()
    }
}

impl AlignedAddress<Physical> {
    /// Get the address as a usize
    pub fn get_address(&self) -> usize {
        self.0.get_address_raw() as usize & Self::ADDRESS_MASK
    }
}

impl core::fmt::Debug for AlignedAddress<Virtual> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualAlignedAddress")
            .field("Address", &self.get_address())
            .field("Inner", &self.0)
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
