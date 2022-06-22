use core::sync::atomic::{AtomicU32, Ordering};

pub enum SemaphoreError {
    /// The amount of tickets has been exhausted
    TicketsExhausted,
}

/// Semaphore (It distributes tickets)
#[derive(Debug)]
pub struct Semaphore {
    count: AtomicU32,
}

impl Semaphore {
    /// Create a new semaphore with the intial ticket count `initial`
    pub const fn new(initial: u32) -> Semaphore {
        Semaphore {
            count: AtomicU32::new(initial),
        }
    }

    /// Increase count
    #[inline]
    pub fn up(&self) {
        self.count.fetch_add(1, Ordering::AcqRel);
    }

    /// Decrease count
    #[inline]
    pub fn down(&self) {
        loop {
            if self.try_down().is_ok() {
                return;
            }
        }
    }

    /// Try to decrease semaphore value
    #[inline]
    pub fn try_down(&self) -> Result<(), SemaphoreError> {
        let mut value = self.count.load(Ordering::Acquire);
        if value > 0 {
            value -= 1;
            self.count.store(value, Ordering::Release);

            Ok(())
        } else {
            Err(SemaphoreError::TicketsExhausted)
        }
    }
}

impl Default for Semaphore {
    fn default() -> Self {
        Semaphore::new(0)
    }
}

unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}
