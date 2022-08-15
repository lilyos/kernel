use super::{AddressError, GenericError, PhysicalAllocatorError};

#[derive(Debug, Clone, Copy)]
pub enum MemoryManagerError {
    AddressUnmapped,
    AddressMapped,
    CannotMapToHugePage,
    VirtualMemoryExhausted,
    Generic(GenericError),
    Address(AddressError),
    PhysicalAllocator(PhysicalAllocatorError),
}
