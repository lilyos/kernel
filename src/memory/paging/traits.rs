use stivale2::boot::tags::structures::MemoryMapStructure;

use super::addresses::{Address, AlignedAddress, Physical, Virtual};

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
    unsafe fn init(&self, mmap: &MemoryMapStructure) -> Self::VMMResult<()>;

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

/// Wrapper for virtual memory managers
pub struct MemoryManager<T: VirtualMemoryManager>(pub T);

impl<T: VirtualMemoryManager> MemoryManager<T> {
    /// Create a new virtal memory manager wrapper
    pub const fn new(i: T) -> Self {
        Self(i)
    }

    /// Initialize the virtual memory manager
    ///
    /// # Arguments
    /// * `mmap` - A slice of MemoryDescriptor
    ///
    /// # Safety
    /// The mmap must be properly formed and all reads and writes must be able to succeed
    pub unsafe fn init(&self, mmap: &MemoryMapStructure) -> T::VMMResult<()> {
        self.0.init(mmap)
    }

    /// Convert a virtual address to a physical address
    ///
    /// # Arguments
    /// * `src` - The virtual address to convert to a physical address
    pub fn virtual_to_physical(&self, src: Address<Virtual>) -> Option<Address<Physical>> {
        self.0.virtual_to_physical(src)
    }

    /// Map a physical address to a virtual address
    ///
    /// # Arguments
    /// * `src` - The physical address to map
    /// * `dst` - The address to map to
    /// * `flags` - Additional flags for the virtual address
    pub fn map(
        &self,
        src: AlignedAddress<Physical>,
        dst: AlignedAddress<Virtual>,
        flags: usize,
    ) -> T::VMMResult<()> {
        self.0.map(src, dst, flags)
    }

    /// Unmap a virtual address
    ///
    /// # Arguments
    /// * `src` - The address to unmap
    pub fn unmap(&self, src: AlignedAddress<Virtual>) -> T::VMMResult<()> {
        self.0.unmap(src)
    }
}
