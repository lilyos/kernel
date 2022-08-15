/// Generic errors
#[derive(Debug, Clone, Copy)]
pub enum GenericError {
    /// The function has not been implemented
    NotImplemented,
    /// This operation is not supported
    NotSupported,
    /// This has not been initialized
    NotInitialized,
    /// An integer conversion error occurred
    IntConversionError,
    /// An Integer overflowed or underflowed
    IntOverflowOrUnderflow,
}
