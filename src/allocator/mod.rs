mod heap;
pub use heap::HeapAllocator;

mod paging;
pub use paging::PageAllocator;

// mod memory_manager;
// pub use memory_manager::MemoryManager;

mod uefi_interop;
pub use uefi_interop::{MemoryDescriptor, MemoryEntry, MemoryKind, MemoryType};

pub struct PhysAddr(pub usize);
pub struct VirtAddr(pub usize);

/// Must be power of 2
pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
