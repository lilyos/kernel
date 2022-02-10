use super::MemoryDescriptor;

/// Errors that can be returned by these allocators.
#[derive(Debug)]
pub enum AllocatorError {
    /// The action has failed because an internal container was full.
    InternalStorageFull,
    /// Shrinking isn't possible because the spare space isn't large enough
    CompactionTooLow,
    /// The action has failed because it hasn't been implemented.
    #[allow(dead_code)]
    NotImplemented,
    /// The allocation has failed because no region was large enough for the request.
    NoLargeEnoughRegion,
    /// The region is too small for the requested size.
    RegionTooSmall,
    /// An internal unexpected error has occured with the following message.
    InternalError(&'static str),
    /// The allocation has failed because there is no free memory.
    OutOfMemory,
    /// The deallocation has failed because it was already freed.
    DoubleFree,
}

pub trait PhysicalAllocatorImpl {
    /// Result for Physical Allocators
    type PAResult<T> = Result<T, AllocatorError>;

    /// Initialize the allocator
    ///
    /// # Arguments
    /// * `mmap` - Slice of memory descriptors
    unsafe fn init(&self, mmap: &[MemoryDescriptor]) -> Self::PAResult<()>;

    /// Allocate physical memory aligned to page
    ///
    /// # Arguments
    /// * `size` - The desired allocation size in kilobytes
    fn alloc(&self, size: usize) -> Self::PAResult<(*mut u8, usize)>;

    /// Deallocate physical memory
    ///
    /// # Arguments
    /// * `block_start` - The block the allocation started on
    /// * `kilos_allocated` - The amount of kilobytes allocated
    fn dealloc(&self, block_start: usize, kilos_allocated: usize) -> Self::PAResult<()>;
}

pub struct PhysicalAllocator<T>(T)
where
    T: PhysicalAllocatorImpl;

impl<T> PhysicalAllocator<T>
where
    T: PhysicalAllocatorImpl,
{
    /// Create a new wrapper with the contained value
    pub const fn new(i: T) -> Self {
        Self(i)
    }

    /// Initialize the allocator
    ///
    /// # Arguments
    /// * `mmap` - Slice of memory descriptors
    pub unsafe fn init(&self, mmap: &[MemoryDescriptor]) -> T::PAResult<()> {
        self.0.init(mmap)
    }

    /// Allocate physical memory aligned to page
    ///
    /// # Arguments
    /// * `size` - The desired allocation size in kilobytes
    pub fn alloc(&self, size: usize) -> T::PAResult<(*mut u8, usize)> {
        self.0.alloc(size)
    }

    /// Deallocate physical memory
    ///
    /// # Arguments
    /// * `block_start` - The block the allocation started on
    /// * `kilos_allocated` - The amount of kilobytes allocated
    pub fn dealloc(&self, block_start: usize, kilos_allocated: usize) -> T::PAResult<()> {
        self.0.dealloc(block_start, kilos_allocated)
    }
}
