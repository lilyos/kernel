mod heap;
pub use heap::HeapAllocator;

mod paging;
pub use paging::PageAllocator;

#[repr(C)]
#[derive(Debug)]
pub struct MemoryEntry {
    start: usize,
    end: usize,
}
