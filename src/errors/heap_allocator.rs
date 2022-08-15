use crate::errors::{GenericError, MemoryManagerError, PhysicalAllocatorError};

use super::AddressError;

/// Errors returned by the heap allocator
#[derive(Debug)]
pub enum HeapAllocatorError {
    /// The action has failed because an internal container was full.
    InternalStorageFull,
    /// The allocation has failed because no region was large enough for the request.
    NoLargeEnoughRegion,
    /// The region is too small for the requested size.
    RegionTooSmall,
    /// The allocation has failed because there is no free memory.
    OutOfMemory,
    /// The deallocation has failed because it was already freed.
    DoubleFree,
    /// Generic Error
    Generic(GenericError),
    /// Physical Allocator Error
    PhysicalAllocator(PhysicalAllocatorError),
    /// Memory Manager Error
    MemoryManager(MemoryManagerError),
    /// Errors from addresses
    Address(AddressError),
}