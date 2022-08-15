use core::alloc::{Allocator, GlobalAlloc};

/// Struct that implements [Allocator] and [GlobalAlloc], but never actually allocates.
/// It instead returns an error each time
pub struct NeverAllocator;

unsafe impl Allocator for NeverAllocator {
    fn allocate(
        &self,
        _: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        Err(core::alloc::AllocError)
    }

    unsafe fn deallocate(&self, _: core::ptr::NonNull<u8>, _: core::alloc::Layout) {
        unreachable!()
    }
}

unsafe impl GlobalAlloc for NeverAllocator {
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
        unreachable!()
    }
}
