mod physical_memory_allocator;
pub use physical_memory_allocator::PhysicalMemoryAllocator;

mod virtual_memory_manager;
pub use virtual_memory_manager::{VirtualMemoryManager, VirtualMemoryManagerError};
