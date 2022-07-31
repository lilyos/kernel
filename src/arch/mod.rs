#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::IMPLEMENTATION as PlatformManager;
#[cfg(target_arch = "x86_64")]
pub type PlatformType = x86_64::X86_64<'static>;
