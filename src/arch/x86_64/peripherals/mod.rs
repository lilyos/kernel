/// The UART
pub mod uart;
pub use uart::Uart;

use crate::sync::{lazy_static, Mutex};

/// Structures and functions relating to the CPU
pub mod cpu;

// use crate::sync::Singleton;

// Peripherals that are hardcoded in because it'd be annoying to have overhead for something like that
// pub static mut PERIPHERALS: Peripherals = Peripherals { uart: Uart::new() };

// pub static UART: Singleton<Uart> = Singleton::new(Uart::new());

lazy_static! {
    /// UART Structure
    pub lazy static UART: Mutex<Uart> = {
        use self::uart::{inb, outb, COM_1};

        outb(0, COM_1 + 1); // Disable all interrupts
        outb(0x80, COM_1 + 3); // Enable DLAB (set baud rate divisor)
        outb(0x03, COM_1); // Set divisor to 3 (lo byte) 38400 baud
        outb(0, COM_1 + 1); //                  (hi byte)
        outb(0x03, COM_1 + 3); // 8 bits, no parity, one stop bit
        outb(0xC7, COM_1 + 2); // Enable FIFO, clear them, with 14-byte threshold
        outb(0x08, COM_1 + 4); // IRQs enabled, RTS/DSR set
        outb(0x1E, COM_1 + 4); // Set in loopback mode, test the serial chip
        outb(0xAE, COM_1); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i.e: not same byte as sent)
        if inb(COM_1) != 0xAE {
            panic!("SERIAL FAILED LOOPBACK TEST");
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(0x0F, COM_1 + 4);
        Mutex::new(Uart::new())
    };
}
