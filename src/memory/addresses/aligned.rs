use core::{fmt::Debug, marker::PhantomData};

use crate::{
    errors::{AddressError, GenericError},
    traits::PlatformAddress,
};

use super::{Address, Physical, RawAddress, UnderlyingType, Virtual};

/// Struct representing an aligned address
pub struct AlignedAddress<T>(pub(crate) RawAddress, pub(crate) PhantomData<T>);

impl<T> Clone for AlignedAddress<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> Copy for AlignedAddress<T> {}

impl<T> AlignedAddress<T> {
    /// Get the inner raw address
    pub fn inner(&self) -> RawAddress {
        self.0
    }
}

impl AlignedAddress<Virtual> {
    /// Try to form an aligned address from a usize
    pub fn new(addr: *const ()) -> Result<Self, AddressError> {
        let addr = addr as usize;
        if addr % 4096 != 0 {
            Err(AddressError::AddressNotAligned)
        } else {
            Ok(AlignedAddress(
                RawAddress::new(addr as UnderlyingType)?,
                PhantomData,
            ))
        }
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

impl AlignedAddress<Physical> {
    /// Try to form an aligned address from a usize
    pub fn new(addr: usize) -> Result<Self, AddressError> {
        let addr = addr;
        if addr % 4096 != 0 {
            Err(AddressError::AddressNotAligned)
        } else {
            Ok(AlignedAddress(
                RawAddress::new(addr as UnderlyingType)?,
                PhantomData,
            ))
        }
    }
}

impl Debug for AlignedAddress<Virtual> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualAlignedAddress")
            .field("Address", &format_args!("{:#X}", self.inner().into_raw()))
            .field("Inner", &self.0)
            .finish()
    }
}

impl Debug for AlignedAddress<Physical> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhysicalAlignedAddress")
            .field("Address", &format_args!("{:#X}", self.inner().into_raw()))
            .finish()
    }
}

impl TryFrom<Address<Virtual>> for AlignedAddress<Virtual> {
    type Error = AddressError;

    fn try_from(value: Address<Virtual>) -> Result<Self, Self::Error> {
        Self::new(value.inner().into_raw() as *const ())
    }
}

impl TryFrom<*const u8> for AlignedAddress<Virtual> {
    type Error = AddressError;

    fn try_from(value: *const u8) -> Result<Self, Self::Error> {
        Self::new(value as *const ())
    }
}

impl TryFrom<*mut u8> for AlignedAddress<Virtual> {
    type Error = AddressError;

    fn try_from(value: *mut u8) -> Result<Self, Self::Error> {
        Self::new(value as *const ())
    }
}

impl From<AlignedAddress<Virtual>> for *const () {
    fn from(val: AlignedAddress<Virtual>) -> Self {
        val.inner().into_raw() as *const ()
    }
}

impl From<AlignedAddress<Virtual>> for *mut () {
    fn from(val: AlignedAddress<Virtual>) -> Self {
        val.inner().into_raw() as *mut ()
    }
}

impl TryFrom<Address<Physical>> for AlignedAddress<Physical> {
    type Error = AddressError;

    fn try_from(value: Address<Physical>) -> Result<Self, Self::Error> {
        Self::new(
            value
                .inner()
                .into_raw()
                .try_into()
                .map_err(|_| AddressError::Generic(GenericError::IntConversionError))?,
        )
    }
}

impl TryFrom<usize> for AlignedAddress<Physical> {
    type Error = AddressError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<UnderlyingType> for AlignedAddress<Physical> {
    type Error = AddressError;

    fn try_from(value: UnderlyingType) -> Result<Self, Self::Error> {
        Self::new(
            value
                .try_into()
                .map_err(|_| AddressError::Generic(GenericError::IntConversionError))?,
        )
    }
}

impl From<AlignedAddress<Physical>> for usize {
    fn from(val: AlignedAddress<Physical>) -> Self {
        val.inner().into_raw() as usize
    }
}

impl From<AlignedAddress<Physical>> for UnderlyingType {
    fn from(val: AlignedAddress<Physical>) -> Self {
        val.inner().into_raw() as UnderlyingType
    }
}

impl<T> From<AlignedAddress<T>> for Address<T> {
    fn from(val: AlignedAddress<T>) -> Self {
        Address(val.inner(), PhantomData)
    }
}
