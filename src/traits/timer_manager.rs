use crate::errors::TimerManagerError;

pub unsafe trait TimerManager {
    type Error = TimerManagerError;

    fn set_timer(
        &self,
        timer_id: u64,
        interval: f64,
        interrupt_num: u64,
    ) -> Result<(), Self::Error>;

    fn clear_timer(&self, timer_id: u64) -> Result<(), Self::Error>;
}
