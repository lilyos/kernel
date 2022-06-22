mod heap;
use core::{fmt::Debug, marker::PhantomData};

pub use heap::HeapAllocator;
use log::error;

use crate::traits::PhysicalMemoryAllocator;

/// Allocation guard
/// Automatically drops physically allocated memory
pub struct AllocGuard<'a> {
    block_start: usize,
    kilos_allocated: usize,
    address: *mut u8,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> Debug for AllocGuard<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AllocGuard")
            .field("block_start", &self.block_start)
            .field("kilos_allocated", &self.kilos_allocated)
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

impl<'a> AllocGuard<'a> {
    /// Create a new AllocGuard
    ///
    /// # Safety
    /// The data must be exclusive to this struct, otherwise it'll cause memory corruption
    pub unsafe fn new(block_start: usize, kilos_allocated: usize, address: *mut u8) -> Self {
        Self {
            block_start,
            kilos_allocated,
            address,
            lifetime: PhantomData,
        }
    }

    /// Get the address of the allocation
    pub fn address_mut(&self) -> *mut u8 {
        self.address
    }

    /// Get the amount of blocks allocated
    pub fn kilos_allocated(&self) -> usize {
        self.kilos_allocated
    }
}

impl<'a> Drop for AllocGuard<'a> {
    fn drop(&mut self) {
        match crate::PHYSICAL_ALLOCATOR.dealloc(self.block_start, self.kilos_allocated) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "Failed to deallocate memory for {:#?} with error {e:?}",
                    self
                );
            }
        }
    }
}
