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

/// Lie about something being sync
#[repr(transparent)]
pub struct FakeSync<T> {
    /// The inner data
    pub _inner: T,
}

impl<T> From<T> for FakeSync<T> {
    fn from(v: T) -> Self {
        Self { _inner: v }
    }
}

unsafe impl<T> Sync for FakeSync<T> {}
