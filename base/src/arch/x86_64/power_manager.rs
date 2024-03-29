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
    fn get_state(&self) -> Result<crate::traits::PowerState, PowerManagerError> {
        Err(PowerManagerError::Generic(GenericError::NotImplemented))
    }

    fn switch_state(&self, _new_state: crate::traits::PowerState) -> Result<(), PowerManagerError> {
        Err(PowerManagerError::Generic(GenericError::NotImplemented))
    }

    fn shutdown(&self, _kind: crate::traits::PowerOffKind) -> ! {
        loop {}
    }
}

impl Init for PowerManager {
    type Error = PowerManagerError;

    type Input = ();
}
