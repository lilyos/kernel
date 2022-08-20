use super::{AddressError, GenericError};

/// Errors returned from allocators
#[derive(Clone, Copy, Debug)]
pub enum AllocatorErrorTyped<T: Clone + Copy + core::fmt::Debug = ()> {
    /// The allocator is out of memory
    OutOfMemory,
    /// The allocator doesn't have enough memory to fufill the allocation request
    NotEnoughMemory,
    /// The allocator is unable to fufill the request due to another reason
    RequestUnfulfillable,
    /// An internal error, free for use
    InternalError(T),
    /// A generic error occurred
    Generic(GenericError),
    /// An address error occured
    Address(AddressError),
}

/// Errors returned from allocators
#[derive(Clone, Copy, Debug)]
pub enum AllocatorError {
    /// The allocator is out of memory
    OutOfMemory,
    /// The allocator doesn't have enough memory to fufill the allocation request
    NotEnoughMemory,
    /// The allocator is unable to fufill the request due to another reason
    RequestUnfulfillable,
    /// An internal error. This is unspecified in this variant for those that don't care to read the internal error
    InternalError,
    /// A generic error occurred
    Generic(GenericError),
    /// An address error occured
    Address(AddressError),
}

impl<T: Clone + Copy + core::fmt::Debug> From<AllocatorErrorTyped<T>> for AllocatorError {
    fn from(val: AllocatorErrorTyped<T>) -> Self {
        match val {
            AllocatorErrorTyped::OutOfMemory => AllocatorError::OutOfMemory,
            AllocatorErrorTyped::NotEnoughMemory => AllocatorError::NotEnoughMemory,
            AllocatorErrorTyped::RequestUnfulfillable => AllocatorError::RequestUnfulfillable,
            AllocatorErrorTyped::InternalError(_) => AllocatorError::InternalError,
            AllocatorErrorTyped::Generic(e) => AllocatorError::Generic(e),
            AllocatorErrorTyped::Address(e) => AllocatorError::Address(e),
        }
    }
}
