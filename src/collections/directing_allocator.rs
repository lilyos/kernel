use core::alloc::Allocator;

pub struct DirectingAllocator<'a, A: Allocator> {
    allocator: &'a A,
}

impl<'a, A: Allocator> DirectingAllocator<'a, A> {
    pub fn from_allocator_ref(allocator: &'a A) -> Self {
        Self { allocator }
    }
}

unsafe impl<'a, A: Allocator> Allocator for DirectingAllocator<'a, A> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        self.allocator.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        self.allocator.deallocate(ptr, layout)
    }
}
