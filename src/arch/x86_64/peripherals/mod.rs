/// The UART
pub mod uart;
pub use uart::Uart;

/// Structures and functions relating to the CPU
pub mod cpu;

use crate::sync::Singleton;

// Peripherals that are hardcoded in because it'd be annoying to have overhead for something like that
// pub static mut PERIPHERALS: Peripherals = Peripherals { uart: Uart::new() };

/// UART Singleton
pub static UART: Singleton<Uart> = Singleton::new(Uart::new());
