use core::{fmt::Debug, marker::PhantomData};

use crate::{
    errors::{AddressError, GenericError},
    memory::utilities::align_down,
    traits::PlatformAddress,
};

use super::{AlignedAddress, Physical, RawAddress, UnderlyingType, Virtual};

/// Struct representing an address
pub struct Address<T>(pub(crate) RawAddress, pub(crate) PhantomData<T>);

impl<T> Address<T> {
    /// Page align an address by truncating the spare bytes
    pub fn align_lossy(&self) -> Result<AlignedAddress<T>, AddressError> {
        Ok(AlignedAddress(
            RawAddress::new_address(
                align_down(
                    (self.0.get_address_raw() as usize).try_into().unwrap(),
                    4096,
                )
                .try_into()
                .map_err(|_| AddressError::Generic(GenericError::IntConversionError))?,
            )?,
            PhantomData,
        ))
    }

    /// Get the inner raw address
    pub fn inner(&self) -> RawAddress {
        self.0
    }
}

impl<T> Clone for Address<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T> Copy for Address<T> {}

impl<T> Debug for Address<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Address")
            .field("Value", &format_args!("{:#X}", self.inner().into_raw()))
            .field("Inner", &self.0)
            .finish()
    }
}

impl Address<Virtual> {
    /// Create a new virtual address
    pub fn new(address: *const ()) -> Result<Self, AddressError> {
        Ok(Self(
            RawAddress::new(address as UnderlyingType)?,
            PhantomData,
        ))
    }

    /// Get the inner value as a pointer
    pub fn get_inner_ptr(&self) -> *const () {
        self.inner().into_raw() as *const ()
    }

    /// Get the inner value as a mutable pointer
    pub fn get_inner_ptr_mut(&mut self) -> *mut () {
        self.inner().into_raw() as *mut ()
    }
}

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
        Ok(Self(
            RawAddress::new(address as UnderlyingType)?,
            PhantomData,
        ))
    }
}
