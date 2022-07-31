use crate::errors::PowerManagerError;

pub enum PowerState {
    Standby,
    DeepSleep,
    Suspend,
    Hibernation,
    Off,
}

pub enum PowerOffKind {
    Reboot,
    Shutdown,
}

pub unsafe trait PowerManager {
    type Error = PowerManagerError;

    fn get_state(&self) -> Result<PowerState, Self::Error>;

    fn switch_state(&self, new_state: PowerState) -> Result<(), Self::Error>;

    fn shutdown(&self, kind: PowerOffKind) -> !;
}
