use core::arch::asm;

use log::trace;

use crate::macros::bitflags::bitflags;

bitflags! {
    pub struct CR0: u64 {
        const PROTECTED_MODE = 1 << 0;
        const MONITOR_COPROCESSOR = 1 << 1;
        const EMULATION = 1 << 2;
        const TASK_SWITCHED = 1 << 3;
        const EXTENSION_TYPE = 1 << 4;
        const NUMERIC_ERROR = 1 << 5;
        const WRITE_PROTECT = 1 << 16;
        const ALIGNMENT_MASK = 1 << 18;
        const NOT_WRITE_THROUGH = 1 << 29;
        const CACHE_DISABLE = 1 << 30;
        const PAGING = 1 << 31;
    }
}

impl CR0 {
    /// Get the CR0 register
    pub fn get() -> Self {
        let mut cr0: u64;
        unsafe { asm!("mov {}, cr0", out(reg) cr0) }
        Self { bits: cr0 }
    }

    /// Push the new value
    pub fn update(&mut self) {
        unsafe { asm!("mov cr0, {}", in(reg) self.bits) }
    }
}

/// Struct representing the RSP register
pub struct RSP(pub *mut u8);

impl RSP {
    /// Get the value of the register
    pub fn get() -> Self {
        let rsp: *mut u8;
        unsafe { asm!("mov {}, rsp", out(reg) rsp) }
        trace!("RSP: {rsp:?}");
        Self(rsp)
    }
}
