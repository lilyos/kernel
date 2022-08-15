use super::{AddressError, GenericError, AllocatorError};

/// Errors returned by the memory manager
#[derive(Debug, Clone, Copy)]
pub enum MemoryManagerError {
    /// The provided address was unmapped
    AddressUnmapped,
    /// The provided address was mapped
    AddressMapped,
    /// Huge pages cannot be mapped into
    CannotMapToHugePage,
    /// Virtual memory space has been exhausted, immediate panic is encouraged
    VirtualMemoryExhausted,
    /// A generic error occurred
    Generic(GenericError),
    /// An address error occurred
    Address(AddressError),
    /// An allocator error occurred
    Allocator(AllocatorError),
}
