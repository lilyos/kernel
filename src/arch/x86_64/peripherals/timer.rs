use crate::{
    errors::{GenericError, TimerManagerError},
    traits::{Init, TimerManager as TimerManagerTrait},
};

pub struct TimerManager {}

unsafe impl TimerManagerTrait for TimerManager {
    type Error = TimerManagerError;

    fn set_timer(
        &self,
        timer_id: u64,
        interval: f64,
        interrupt_num: u64,
    ) -> Result<(), Self::Error> {
        Err(TimerManagerError::Generic(GenericError::NotImplemented))
    }

    fn clear_timer(&self, timer_id: u64) -> Result<(), Self::Error> {
        Err(TimerManagerError::Generic(GenericError::NotImplemented))
    }
}

impl Init for TimerManager {
    type Error = core::convert::Infallible;

    type Input = ();
}
