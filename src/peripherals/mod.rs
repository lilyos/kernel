pub mod uart;
pub use uart::Uart;

pub mod cpu;

mod mailbox;
pub use mailbox::*;

use crate::sync::Singleton;

// Peripherals that are hardcoded in because it'd be annoying to have overhead for something like that
// pub static mut PERIPHERALS: Peripherals = Peripherals { uart: Uart::new() };

pub static mut UART: Singleton<Uart> = Singleton::new(Uart::new());

pub static mut MAILBOX: Singleton<Mailbox> = Singleton::new(Mailbox::new());
