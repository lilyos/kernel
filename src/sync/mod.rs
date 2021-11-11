/*
#[cfg(feature = "atomic")]
pub mod atomic;
#[cfg(feature = "atomic")]
pub use atomic::*;
#[cfg(feature = "sync")]
pub mod notatomic;
#[cfg(feature = "sync")]
pub use notatomic::*;
*/

#[allow(dead_code)]
mod atomic;
pub use atomic::*;
