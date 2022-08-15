use core::{
    arch::asm,
    fmt::{Error, Write},
};

use crate::traits::Init;

/// Uart structure for reading and writing to itself
pub struct Uart {}

/// COM Port 1 location
pub const COM_1: u16 = 0x3F8;

#[allow(dead_code)]
impl Uart {
    /// Construct a new UART instance. There should be no more than one
    pub const fn new() -> Self {
        Uart {}
    }

    /// Check if the outbound fifo is full
    fn write_full(&self) -> bool {
        (inb(COM_1 + 5) & 0x20) == 0
    }

    /// Check if the inbound fifo is empty
    fn read_empty(&self) -> bool {
        inb(COM_1 + 5) & 1 == 0
    }

    /// Read a byte from the UART
    pub fn read_byte(&mut self) -> u8 {
        loop {
            if self.read_empty() {
                unsafe { asm!("nop") }
            } else {
                break;
            }
        }
        inb(COM_1)
    }

    /// Write a byte to the UART
    pub fn write_byte(&mut self, c: u8) {
        loop {
            if self.write_full() {
                unsafe { asm!("nop") }
            } else {
                break;
            }
        }

        outb(c, COM_1);
    }
}

/// Write a byte wide value to a port
pub fn outb(val: u8, port: u16) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack, preserves_flags)
        )
    }
}

/// Read a byte wide value from a port
pub fn inb(port: u16) -> u8 {
    let result: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") result,
            options(nomem, nostack, preserves_flags)
        )
    }
    result
}

impl Write for Uart {
    fn write_str(&mut self, data: &str) -> Result<(), Error> {
        for c in data.chars() {
            self.write_byte(c as u8);
        }
        Ok(())
    }
}

impl Init for Uart {}
