use crate::errors::GenericError;

/// Errors that can be returned by these allocators.
#[derive(Debug)]
pub enum AllocatorError {
    /// The action has failed because an internal container was full.
    InternalStorageFull,
    /// Shrinking isn't possible because the spare space isn't large enough
    CompactionTooLow,
    /// The action has failed because it hasn't been implemented.
    #[allow(dead_code)]
    NotImplemented,
    /// The allocation has failed because no region was large enough for the request.
    NoLargeEnoughRegion,
    /// The region is too small for the requested size.
    RegionTooSmall,
    /// An internal unexpected error has occured with the following message.
    InternalError(&'static str),
    /// The allocation has failed because there is no free memory.
    OutOfMemory,
    /// The deallocation has failed because it was already freed.
    DoubleFree,
    /// If the allocator or any of its children haven't been initialized
    Uninitialized,
    /// Generic Error
    Generic(GenericError),
}
