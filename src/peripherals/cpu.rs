// Delay `time` cpu cycles
pub fn delay(time: u32) {
    for _ in 0..time {
        unsafe { asm!("nop") };
    }
}

pub unsafe fn get_exception_level() -> u32 {
    let x: u32;
    asm!("mrs {:x}, CurrentEL", out(reg) x);
    x >> 2
}
