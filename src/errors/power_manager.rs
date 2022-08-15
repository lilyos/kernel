use super::GenericError;

/// Errors that occur when switching power state
#[derive(Debug, Clone, Copy)]
pub enum PowerManagerError {
    /// An unknown error occurred trying to switch state
    FailedToSwitchState,
    /// Invalid permissions
    InsufficientPermissions,
    /// Transition is invalid
    InvalidStateSwitch,
    /// A generic error occurred
    Generic(GenericError),
}
