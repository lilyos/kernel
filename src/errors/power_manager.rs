use super::GenericError;

#[derive(Debug, Clone, Copy)]
pub enum PowerManagerError {
    FailedToSwitchState,
    InsufficientPermissions,
    InvalidStateSwitch,
    Generic(GenericError),
}
