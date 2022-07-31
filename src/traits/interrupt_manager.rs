use crate::{errors::InterruptManagerError, interrupts::InterruptType};

/// Trait for a [Platform](crate::traits::Platform)'s Interrupt Manager
pub unsafe trait InterruptManager {
    fn disable_interrupts(&self) -> Result<(), InterruptManagerError>;

    fn enable_interrupts(&self) -> Result<(), InterruptManagerError>;

    fn set_handler<T: Fn(InterruptType)>(&self, func: &T) -> Result<(), InterruptManagerError>;
}
