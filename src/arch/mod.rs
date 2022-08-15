#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::IMPLEMENTATION as PLATFORM_MANAGER;
#[cfg(target_arch = "x86_64")]
pub type PlatformType = x86_64::X86_64;
