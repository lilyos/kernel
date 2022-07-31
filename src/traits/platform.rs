use core::alloc::Allocator;

use log::Log;

use super::{Init, InterruptManager, MemoryManager, PowerManager, RawAddress, TimerManager};

pub unsafe trait Platform: Init {
    /// The Platform's Physical Allocator
    type PhysicalAllocator: Allocator + Init;

    /// The Platform's Memory Manager
    type MemoryManager: MemoryManager + Init;

    /// The Platform's Interrupt Manager
    type InterruptManager: InterruptManager + Init;

    /// The Platform's Power Manager
    type PowerManager: PowerManager + Init;

    /// The Platform's Timer Manager (or a facade to the real one, doesn't matter in this instance)
    type TimerManager: TimerManager + Init;

    /// The Platform's Raw Address type, which Address routes through for genericness
    type RawAddress: RawAddress;

    /// The Platform's Logger
    type Logger: Log + Init;

    fn get_physical_allocator(&self) -> &'static Self::PhysicalAllocator;

    fn get_memory_manager(&self) -> &'static Self::MemoryManager;

    fn get_interrupt_manager(&self) -> &'static Self::InterruptManager;

    fn get_power_manager(&self) -> &'static Self::PowerManager;

    fn get_logger(&self) -> &'static Self::Logger;
}
