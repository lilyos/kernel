use self::memory::{memory_manager::MemoryManager, page_allocator::PageAllocator};

/// Architecture-specific structures, such as the IDT or GDT
pub mod structures;

/// Architecture-specific code relating to memory management and virtual memory
pub mod memory;

/// Peripherals
pub mod peripherals;

/// The Physical Memory Allocator
pub(crate) static PHYSICAL_ALLOCATOR: PageAllocator = PageAllocator::new();

/// The Virtual Memory manager
pub(crate) static MEMORY_MANAGER: MemoryManager = MemoryManager::new();
