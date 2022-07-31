#[derive(Debug, Clone, Copy)]
pub enum GenericError {
    NotImplemented,
    NotSupported,
    NotInitialized,
    IntConversionError,
    AllocationFailed,
}
