#![allow(asm_sub_register)]

#[inline]
pub fn pi_version() -> u32 {
    let midr: u32;
    unsafe {
        asm!("mrs {}, midr_el1", out(reg) midr);
    }

    match (midr >> 4) & 0xFFF {
        0xC07 => 2,
        0xD03 => 3,
        0xD08 => 4,
        _ => 0,
    }
}
