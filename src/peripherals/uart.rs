use core::fmt::{Error, Write};
use core::ptr::{read_volatile, write_volatile};

use crate::traits::Init;

use super::cpu::delay;

// Uart struct storing a lot of pointers for various things
pub struct Uart {
    addr: *mut u32,
    dr: *mut u32,
    rsrecr: *mut u32,
    fr: *const u32,
    ilpr: *mut u32,
    ibrd: *mut u32,
    fbrd: *mut u32,
    lcrh: *mut u32,
    cr: *mut u32,
    ifls: *mut u32,
    imsc: *mut u32,
    ris: *mut u32,
    mis: *mut u32,
    icr: *mut u32,
    dmacr: *mut u32,
    itcr: *mut u32,
    itip: *mut u32,
    itop: *mut u32,
    tdr: *mut u32,
}

#[allow(dead_code)]
impl Uart {
    pub const fn new() -> Self {
        Uart {
            addr: 0 as *mut u32,
            dr: 0 as *mut u32,
            rsrecr: 0 as *mut u32,
            fr: 0 as *const u32,
            ilpr: 0 as *mut u32,
            ibrd: 0 as *mut u32,
            fbrd: 0 as *mut u32,
            lcrh: 0 as *mut u32,
            cr: 0 as *mut u32,
            ifls: 0 as *mut u32,
            imsc: 0 as *mut u32,
            ris: 0 as *mut u32,
            mis: 0 as *mut u32,
            icr: 0 as *mut u32,
            dmacr: 0 as *mut u32,
            itcr: 0 as *mut u32,
            itip: 0 as *mut u32,
            itop: 0 as *mut u32,
            tdr: 0 as *mut u32,
        }
    }

    fn write_full(&self) -> bool {
        (unsafe { read_volatile(self.fr) } & (1 << 5)) > 0
    }

    fn read_empty(&self) -> bool {
        (unsafe { read_volatile(self.fr) } & (1 << 4)) > 0
    }

    pub fn read_byte(&mut self) -> u8 {
        loop {
            if !self.read_empty() {
                break unsafe { read_volatile(self.dr) as u8 };
            }
        }
    }

    pub fn write_byte(&mut self, c: u8) {
        loop {
            if !self.write_full() {
                unsafe { write_volatile(self.dr, c as u32) };
                break;
            }
        }
    }
}

impl Init for Uart {
    fn pre_init(&mut self) {
        let address = crate::info::peripheral_base();

        // MMIO Address + GPIO Offset + UART0 Address
        let base = address + 0x200000 + 0x1000;

        // Initialize all fields
        self.dr = base as *mut u32;
        self.rsrecr = (base + 0x04) as *mut u32;
        self.fr = (base + 0x18) as *const u32;
        self.ilpr = (base + 0x20) as *mut u32;
        self.ibrd = (base + 0x24) as *mut u32;
        self.fbrd = (base + 0x28) as *mut u32;
        self.lcrh = (base + 0x2c) as *mut u32;
        self.cr = (base + 0x30) as *mut u32;
        self.ifls = (base + 0x34) as *mut u32;
        self.imsc = (base + 0x38) as *mut u32;
        self.ris = (base + 0x3c) as *mut u32;
        self.mis = (base + 0x40) as *mut u32;
        self.icr = (base + 0x44) as *mut u32;
        self.dmacr = (base + 0x48) as *mut u32;
        self.itcr = (base + 0x80) as *mut u32;
        self.itip = (base + 0x84) as *mut u32;
        self.itop = (base + 0x88) as *mut u32;
        self.tdr = (base + 0x8c) as *mut u32;
    }

    fn init(&mut self) {
        // GPIO Control Registers
        let up = (self.addr as u32 + 0x94) as *mut u32;
        let clk = (self.addr as u32 + 0x98) as *mut u32;

        unsafe {
            // Disable UART0
            write_volatile(self.cr, 0x0);

            // Disable all pull ups/downs
            write_volatile(up, 0x0);
            delay(150);

            // Disable pull ups/downs for pins 14/15
            write_volatile(clk, (1 << 14) | (1 << 15));
            delay(150);

            // Make writes take effect
            write_volatile(clk, 0x0);

            // Clear interrupts
            write_volatile(self.icr, 0x7ff);
            write_volatile(self.ibrd, 1);
            write_volatile(self.fbrd, 40);
            write_volatile(self.lcrh, (1 << 4) | (1 << 5) | (1 << 6));
            // Disable interrupts
            write_volatile(
                self.imsc,
                (1 << 1)
                    | (1 << 4)
                    | (1 << 5)
                    | (1 << 6)
                    | (1 << 7)
                    | (1 << 8)
                    | (1 << 9)
                    | (1 << 10),
            );
            write_volatile(self.cr, (1 << 0) | (1 << 8) | (1 << 9));
        }
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
