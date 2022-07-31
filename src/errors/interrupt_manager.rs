use super::GenericError;

#[derive(Debug, Clone, Copy)]
pub enum InterruptManagerError {
    InterruptsAlreadyEnabled,
    InterruptsAlreadyDisabled,
    HandlerAlreadySet,
    Generic(GenericError),
}
