use super::{AddressError, GenericError};

#[derive(Debug, Clone, Copy)]
pub enum MemoryManagerError {
    AddressUnmapped,
    AddressMapped,
    CannotMapToHugePage,
    Generic(GenericError),
    Address(AddressError),
}
