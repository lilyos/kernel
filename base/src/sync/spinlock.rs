use core::{
    arch::asm,
    sync::atomic::{AtomicBool, Ordering},
};

/// A spinlock
#[derive(Debug)]
pub struct Spinlock {
    flag: AtomicBool,
}

impl Spinlock {
    /// Create a new spinlock
    #[must_use]
    pub const fn new() -> Self {
        Self {
            flag: AtomicBool::new(false),
        }
    }

    /// Spin to acquire the lock
    #[inline]
    pub fn aquire(&self) {
        // Set lock if previously not
        while self
            .flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Acquire)
            .is_err()
        {
            unsafe { asm!("nop") }
        }
    }

    /// Release the lock
    #[inline]
    pub fn release(&self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}
