use super::GenericError;

#[derive(Debug, Clone, Copy)]
/// Errors that occur when trying to convert an address between types
pub enum AddressError {
    /// The address wasn't aligned
    AddressNotAligned,
    /// The address wasn't canonical
    AddressNonCanonical,
    /// The address was greater than the underlying type
    ConversionError,
    /// An unspecified error occurred
    Other,
    /// Generic Error
    Generic(GenericError),
}
