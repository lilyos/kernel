/// Enum representing that something contains a virtual address
#[derive(Clone, Copy, Debug)]
pub enum Virtual {}

/// Enum representing that something contains a physical address
#[derive(Clone, Copy, Debug)]
pub enum Physical {}

type RawAddress = <crate::arch::PlatformType as crate::traits::Platform>::RawAddress;
type UnderlyingType = <RawAddress as crate::traits::PlatformAddress>::UnderlyingType;

mod aligned;
pub use aligned::AlignedAddress;

mod nonaligned;
pub use nonaligned::Address;
