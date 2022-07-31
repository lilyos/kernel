use core::{fmt::Debug, marker::PhantomData};

use crate::{
    errors::{AddressError, GenericError},
    traits::RawAddress as RawAddressTrait,
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
    /// The address mask
    pub const ADDRESS_MASK: usize = 0x000F_FFFF_FFFF_F000;

    /// Get the inner raw address
    pub fn inner(&self) -> RawAddress {
        self.0
    }
}

impl AlignedAddress<Virtual> {
    /// Try to form an aligned address from a usize
    fn new(addr: *const ()) -> Result<Self, AddressError> {
        let addr = addr as usize;
        if addr % 4096 != 0 {
            Err(AddressError::AddressNotAligned)
        } else {
            Ok(AlignedAddress(
                unsafe { RawAddress::new_address(addr as UnderlyingType)? },
                PhantomData,
            ))
        }
    }
}

impl AlignedAddress<Physical> {
    /// Try to form an aligned address from a usize
    fn new(addr: usize) -> Result<Self, AddressError> {
        let addr = addr;
        if addr % 4096 != 0 {
            Err(AddressError::AddressNotAligned)
        } else {
            Ok(AlignedAddress(
                unsafe { RawAddress::new_address(addr as UnderlyingType)? },
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

impl Into<*const ()> for AlignedAddress<Virtual> {
    fn into(self) -> *const () {
        self.inner().into_raw() as *const ()
    }
}

impl Into<*mut ()> for AlignedAddress<Virtual> {
    fn into(self) -> *mut () {
        self.inner().into_raw() as *mut ()
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

impl Into<usize> for AlignedAddress<Physical> {
    fn into(self) -> usize {
        self.inner().into_raw() as usize
    }
}

impl Into<UnderlyingType> for AlignedAddress<Physical> {
    fn into(self) -> UnderlyingType {
        self.inner().into_raw() as UnderlyingType
    }
}
