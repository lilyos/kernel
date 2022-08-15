use super::GenericError;

/// Errors for the timer manager
#[derive(Debug, Clone, Copy)]
pub enum TimerManagerError {
    /// The specified timer has already been set
    TimerAlreadySet,
    /// The specified timer is not present
    TimerNotPresent,
    /// A generic error occurred
    Generic(GenericError),
}
