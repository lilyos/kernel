use core::fmt;

use crate::{errors::AddressError, memory::addresses::Address};

/// Trait for abstracting addresses
pub trait PlatformAddress: fmt::Debug + Clone + Copy {
    /// The type the trait is being implemented for
    type AddressType;

    /// The underlying type of the address (i.e. `u32`, `u64`, so forth)
    type UnderlyingType;

    /// Construct a new address
    fn new_address(addr: Self::UnderlyingType) -> Result<Self::AddressType, AddressError>;

    /// Test if an address is valid
    fn address_valid<T>(addr: Address<T>) -> bool;

    /// Convert an address into its raw type
    fn into_raw(self) -> Self::UnderlyingType;
}
