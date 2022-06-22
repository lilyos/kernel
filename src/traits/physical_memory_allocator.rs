use limine_protocol::structures::memory_map_entry::MemoryMapEntry;

use crate::memory::{allocators::AllocGuard, errors::AllocatorError};

/// The trait physical memory allocators must implement
pub trait PhysicalMemoryAllocator {
    /// Result for Physical Allocators
    type PAResult<T> = Result<T, AllocatorError>;

    /// Initialize the allocator
    ///
    /// # Arguments
    /// * `mmap` - Slice of memory descriptors
    ///
    /// # Safety
    /// The memory map must be properly formed and non-overlapping
    unsafe fn init(&self, mmap: &[&MemoryMapEntry]) -> Self::PAResult<()>;

    /// Allocate physical memory aligned to page
    ///
    /// # Arguments
    /// * `size` - The desired allocation size in kilobytes
    fn alloc<'a>(&self, size: usize) -> Self::PAResult<AllocGuard<'a>>;

    /// Deallocate physical memory
    ///
    /// # Arguments
    /// * `block_start` - The block the allocation started on
    /// * `kilos_allocated` - The amount of kilobytes allocated
    fn dealloc(&self, block_start: usize, kilos_allocated: usize) -> Self::PAResult<()>;
}
