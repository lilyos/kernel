/*
#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "arm")]
mod arm6;
#[cfg(target_arch = "arm")]
pub use arm6::*;
*/
mod board;
pub use board::*;

#[inline]
pub fn peripheral_base() -> u32 {
    match pi_version() {
        2 | 3 => 0x3F000000,
        4 => 0xFE000000,
        _ => 0x20000000,
    }
}
