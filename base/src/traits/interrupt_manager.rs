use crate::{errors::InterruptManagerError, interrupts::InterruptType};

/// Trait for a [Platform](crate::traits::Platform)'s Interrupt Manager
///
/// # Safety
/// This must not overwrite any important memory or so forth
pub unsafe trait InterruptManager {
    /// Disable interrupts
    ///
    /// # Errors
    /// This will return an error if interrupts
    /// couldn't be disabled.
    /// This should be a cause for concern
    fn disable_interrupts(&self) -> Result<(), InterruptManagerError>;

    /// Enable interrupts
    ///
    /// # Errors
    /// This will return an error if interrupts
    /// couldn't be enabled.
    /// This should be a cause for concern
    fn enable_interrupts(&self) -> Result<(), InterruptManagerError>;

    /// Set the interrupt handler. This should only be done once.
    ///
    /// # Errors
    /// This will return an error if the interrupt
    /// handler couldn't be set.
    /// This should be a cause for concern
    fn set_handler<T: Fn(InterruptType)>(&self, func: &T) -> Result<(), InterruptManagerError>;
}
