use core::alloc::Layout;

use crate::{
    errors::AllocatorError,
    memory::addresses::{AlignedAddress, Physical},
};

/// Trait for allocating physical memory
pub unsafe trait PhysicalAllocator {
    /// Allocate a specified layout using the physical allocator
    fn allocate(&self, layout: Layout) -> Result<AlignedAddress<Physical>, AllocatorError>;

    /// Deallocate the physical address
    unsafe fn deallocate(&self, addr: AlignedAddress<Physical>, layout: Layout);
}

unsafe impl<'a, T> PhysicalAllocator for &'a T
where
    T: PhysicalAllocator,
{
    fn allocate(&self, layout: Layout) -> Result<AlignedAddress<Physical>, AllocatorError> {
        (**self).allocate(layout)
    }

    unsafe fn deallocate(&self, addr: AlignedAddress<Physical>, layout: Layout) {
        (**self).deallocate(addr, layout)
    }
}
