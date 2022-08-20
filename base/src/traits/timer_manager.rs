use crate::errors::TimerManagerError;

/// Trait for managing timers
pub unsafe trait TimerManager {
    /// Set a timer
    fn set_timer(&self, _: u64, _: f64, _: u64) -> Result<(), TimerManagerError>;

    /// Clear a timer
    fn clear_timer(&self, _: u64) -> Result<(), TimerManagerError>;
}
