use core::fmt;

use crate::{errors::AddressError, memory::addresses::Address};

pub trait RawAddress: fmt::Debug + Clone + Copy {
    type Error = AddressError;

    type AddressType;

    type UnderlyingType;

    fn new_address(addr: Self::UnderlyingType) -> Result<Self::AddressType, Self::Error>;

    fn address_valid<T>(addr: Address<T>) -> bool;

    fn into_raw(self) -> Self::UnderlyingType;
}
