use crate::errors::PowerManagerError;

/// Possible power states
pub enum PowerState {
    /// Peripherals off
    Standby,
    /// Clock down cpu
    Suspend,
    /// Save RAM to Disk
    Hibernation,
    /// Turn PC fully off
    Off,
}

/// How to power off
pub enum PowerOffKind {
    /// Reboot
    Reboot,
    /// Shutdown
    Shutdown,
}

/// Trait for managing power on a platform
pub unsafe trait PowerManager {
    /// Get the current power state
    fn get_state(&self) -> Result<PowerState, PowerManagerError>;

    /// Switch to another state
    fn switch_state(&self, new_state: PowerState) -> Result<(), PowerManagerError>;

    /// Shutdown
    fn shutdown(&self, kind: PowerOffKind) -> !;
}
