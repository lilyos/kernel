use super::GenericError;

#[derive(Debug, Clone, Copy)]
pub enum TimerManagerError {
    TimerAlreadySet,
    TimerNotPresent,
    Generic(GenericError),
}
