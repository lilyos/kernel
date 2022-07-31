use crate::{
    errors::{GenericError, PowerManagerError},
    traits::{Init, PowerManager as PowerManagerTrait},
};

pub struct PowerManager {}

impl PowerManager {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl PowerManagerTrait for PowerManager {
    type Error = PowerManagerError;

    fn get_state(&self) -> Result<crate::traits::PowerState, Self::Error> {
        Err(Self::Error::Generic(GenericError::NotImplemented))
    }

    fn switch_state(&self, new_state: crate::traits::PowerState) -> Result<(), Self::Error> {
        Err(Self::Error::Generic(GenericError::NotImplemented))
    }

    fn shutdown(&self, kind: crate::traits::PowerOffKind) -> ! {
        loop {}
    }
}

impl Init for PowerManager {
    type Error = PowerManagerError;

    type Input = ();
}
