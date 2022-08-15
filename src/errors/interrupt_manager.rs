use super::GenericError;

/// Errors from the Interrupt Manager
#[derive(Debug, Clone, Copy)]
pub enum InterruptManagerError {
    /// Interrupts have already been enabled
    InterruptsAlreadyEnabled,
    /// Interrupts have already been disabled
    InterruptsAlreadyDisabled,
    /// The interrupt handler has already been set
    HandlerAlreadySet,
    /// A generic error occurred
    Generic(GenericError),
}
