mod address;
pub use address::AddressError;

mod generic;
pub use generic::GenericError;

mod interrupt_manager;
pub use interrupt_manager::InterruptManagerError;

mod memory_manager;
pub use memory_manager::MemoryManagerError;

mod power_manager;
pub use power_manager::PowerManagerError;

mod timer_manager;
pub use timer_manager::TimerManagerError;

// mod heap_allocator;
// pub use heap_allocator::HeapAllocatorError;

mod allocator;
pub use allocator::{AllocatorError, AllocatorErrorTyped};