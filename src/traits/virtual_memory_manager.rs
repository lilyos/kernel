use limine_protocol::structures::memory_map_entry::MemoryMapEntry;

use crate::memory::addresses::{Address, AlignedAddress, Physical, Virtual};

/// Errors for the Virtual Memory Manager
#[derive(Debug)]
pub enum VirtualMemoryManagerError {
    /// The requested feature isn't implemented
    NotImplemented,
    /// Huge pages can't have children
    AttemptedToMapToHugePage,
    /// The desired page in the virtual address doesn't exist
    PageNotFound,
    /// The address was unaligned
    UnalignedAddress,
    /// The address was non-canonical
    AddressNotCanonical,
}

/// The trait that a Virtual Memory Maager must implement
pub trait VirtualMemoryManager {
    /// Result type for the virtual memory manager
    type VMMResult<T> = Result<T, VirtualMemoryManagerError>;

    /// Initialize the virtual memory manager
    ///
    /// # Arguments
    /// * `mmap` - A slice of MemoryDescriptor
    ///
    /// # Safety
    /// The mmap must be correctly formed and all writes and reads must be able succeed
    unsafe fn init(&self, mmap: &[&MemoryMapEntry]) -> Self::VMMResult<()>;

    /// Convert a virtual address to a physical address
    ///
    /// # Arguments
    /// * `src` - The virtual address to convert to a physical address
    fn virtual_to_physical(&self, src: Address<Virtual>) -> Option<Address<Physical>>;

    /// Map a physical address to a virtual address
    ///
    /// # Arguments
    /// * `src` - The physical address to map
    /// * `dst` - The address to map to
    /// * `flags` - Additional flags for the virtual address
    fn map(
        &self,
        src: AlignedAddress<Physical>,
        dst: AlignedAddress<Virtual>,
        flags: usize,
    ) -> Self::VMMResult<()>;

    /// Unmap a virtual address
    ///
    /// # Arguments
    /// * `src` - The address to unmap
    fn unmap(&self, src: AlignedAddress<Virtual>) -> Self::VMMResult<()>;
}
