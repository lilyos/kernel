use core::{
    arch::asm,
    fmt::{Error, Write},
};
// use core::ptr::{read_volatile, write_volatile};

use crate::traits::Init;

// use super::cpu::delay;

// Uart struct storing a lot of pointers for various things
pub struct Uart {}

const COM_1: u16 = 0x3F8;

#[allow(dead_code)]
impl Uart {
    pub const fn new() -> Self {
        Uart {}
    }

    fn write_full(&self) -> bool {
        (inb(COM_1 + 5) & 0x20) == 0
    }

    fn read_empty(&self) -> bool {
        inb(COM_1 + 5) & 1 == 0
    }

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

fn outb(val: u8, port: u16) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
        )
    }
}

fn inb(port: u16) -> u8 {
    let result: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") result
        )
    }
    result
}

impl Init for Uart {
    fn pre_init(&mut self) {}

    fn init(&mut self) {
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
    }

    fn post_init(&mut self) {}
}

impl Write for Uart {
    fn write_str(&mut self, data: &str) -> Result<(), Error> {
        for c in data.chars() {
            self.write_byte(c as u8);
        }
        Ok(())
    }
}

macro_rules! print {
    ($($arg:tt)*) => (
        {
            use core::fmt::Write;
            let mut uart = crate::peripherals::UART.lock();
            uart.write_fmt(format_args!($($arg)*)).unwrap();
        }
    );
}

pub(crate) use print;

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use println;
