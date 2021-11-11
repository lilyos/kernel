use core::ptr::{read_volatile, write_volatile};

struct Gpio {
    base: u32;
}

impl Gpio {
    pub fn write(&mut self, address: *mut u32, value: u32) {
        self.init();
        unsafe {
            write_volatile(address, value);
        }
    }

    pub fn read<T>(&mut self, address: *const T) -> T {
        self.init();
        unsafe {
            read_volatile(address)
        }
    }

    pub fn pi_version() -> u32 {
        self.init();
        return self.pi_type;
    }
}