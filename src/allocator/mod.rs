mod heap;
pub use heap::HeapAllocator;

mod paging;
pub use paging::PageAllocator;

/// Must be power of 2
pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum MemoryKind {
    Reclaim,
    ACPIReclaim,
    ACPINonVolatile,
}

#[repr(C)]
#[derive(Debug)]
pub struct MemoryEntry {
    pub start: usize,
    pub end: usize,
    pub kind: MemoryKind,
}
