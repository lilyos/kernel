use crate::{
    errors::{GenericError, TimerManagerError},
    traits::{Init, TimerManager as TimerManagerTrait},
};

pub struct TimerManager {}

unsafe impl TimerManagerTrait for TimerManager {
    fn set_timer(
        &self,
        _: u64,
        _: f64,
        _: u64,
    ) -> Result<(), TimerManagerError> {
        Err(TimerManagerError::Generic(GenericError::NotImplemented))
    }

    fn clear_timer(&self, _: u64) -> Result<(), TimerManagerError> {
        Err(TimerManagerError::Generic(GenericError::NotImplemented))
    }
}

impl Init for TimerManager {
    type Error = core::convert::Infallible;

    type Input = ();
}
