use super::GenericError;

#[derive(Debug, Clone, Copy)]
pub enum PhysicalAllocatorError {
    OutOfMemory,
    AllocationTooLarge,
    NoMatchingAlignment,
    Generic(GenericError),
}
