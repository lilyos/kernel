use core::cell::UnsafeCell;

use crate::{
    errors::{GenericError, InterruptManagerError},
    traits::{Init, InterruptManager as InterruptManagerTrait},
};

use super::structures::InterruptDescriptorTable;

pub struct InterruptManager {
    idt: UnsafeCell<InterruptDescriptorTable>,
}

impl InterruptManager {
    pub const fn new() -> Self {
        Self {
            idt: UnsafeCell::new(InterruptDescriptorTable::new()),
        }
    }

    /// Get a mutable reference to the contained IDT
    ///
    /// # Safety
    /// The caller must ensure that multiple mutable references to the IDT
    /// do not exist at the same time, as that would be Undefined Behavior
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn idt(&self) -> &mut InterruptDescriptorTable {
        &mut *(self.idt.get())
    }
}

unsafe impl InterruptManagerTrait for InterruptManager {
    fn disable_interrupts(&self) -> Result<(), InterruptManagerError> {
        unsafe { asm!("cli") }
        Ok(())
    }

    fn enable_interrupts(&self) -> Result<(), InterruptManagerError> {
        unsafe { asm!("sti") }
        Ok(())
    }

    fn set_handler<T: Fn(crate::interrupts::InterruptType)>(
        &self,
        _func: &T,
    ) -> Result<(), InterruptManagerError> {
        Err(InterruptManagerError::Generic(GenericError::NotImplemented))
    }
}

impl Init for InterruptManager {
    type Error = core::convert::Infallible;

    type Input = ();

    fn init(&self, _: Self::Input) -> Result<(), Self::Error> {
        unsafe { super::structures::install_interrupt_handler() }
        Ok(())
    }
}

unsafe impl Sync for InterruptManager {}
