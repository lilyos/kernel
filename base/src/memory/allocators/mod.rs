mod heap;
pub use heap::Allocator as HeapAllocator;

mod physical_allocator;
pub use physical_allocator::PageAllocator;

mod never_allocate;
pub use never_allocate::NeverAllocator;
