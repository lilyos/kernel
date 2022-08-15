use crate::{errors::InterruptManagerError, interrupts::InterruptType};

/// Trait for a [Platform](crate::traits::Platform)'s Interrupt Manager
pub unsafe trait InterruptManager {
    /// Disable interrupts
    fn disable_interrupts(&self) -> Result<(), InterruptManagerError>;

    /// Enable interrupts
    fn enable_interrupts(&self) -> Result<(), InterruptManagerError>;

    /// Set the interrupt handler. This should only be done once.
    fn set_handler<T: Fn(InterruptType)>(&self, func: &T) -> Result<(), InterruptManagerError>;
}
