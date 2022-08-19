use core::fmt::Write;

use crate::smp::CoreLocalData;

use super::{Init, InterruptManager, MemoryManager, PlatformAddress, PowerManager, TimerManager};

/// The trait for abstracting platforms for use in the kernel
pub unsafe trait Platform: Init {
    /// The Platform's Memory Manager
    type MemoryManager: MemoryManager + Init;

    /// The Platform's Interrupt Manager
    type InterruptManager: InterruptManager + Init;

    /// The Platform's Power Manager
    type PowerManager: PowerManager + Init;

    /// The Platform's Timer Manager (or a facade to the real one, doesn't matter in this instance)
    type TimerManager: TimerManager + Init;

    /// The Platform's Raw Address type, which Address routes through for genericness
    type RawAddress: PlatformAddress;

    /// The Platform's Text Output
    type TextOutput: Write + Init;

    /// Get the platform's memory manager
    fn get_memory_manager(&'static self) -> &'static Self::MemoryManager;

    /// Get the platform's interrupt manager
    fn get_interrupt_manager(&'static self) -> &'static Self::InterruptManager;

    /// Get the platform's power manager
    fn get_power_manager(&'static self) -> &'static Self::PowerManager;

    /// Get the platform's text output
    #[allow(clippy::mut_from_ref)]
    fn get_text_output(&'static self) -> &'static mut Self::TextOutput;

    /// Initialize the cure this is ran on
    fn initialize_current_core(&'static self);

    /// Get the core local data structure
    fn get_core_local(&'static self) -> &'static CoreLocalData;
}
