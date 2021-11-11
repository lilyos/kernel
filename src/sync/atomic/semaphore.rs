use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct Semaphore {
    count: AtomicU32,
}

impl Semaphore {
    pub const fn new(initial: u32) -> Semaphore {
        Semaphore {
            count: AtomicU32::new(initial),
        }
    }

    // Increase count
    #[inline]
    pub fn up(&self) {
        self.count.fetch_add(1, Ordering::AcqRel);

        unsafe {
            // Ensure memory accesses are completed
            asm!("dmb sy");

            // Pause execution until memory, cache, branch prediction, and tlb operations are complete
            asm!("dsb sy");

            // Tell all cores to wake up
            asm!("sev");
        }
    }

    // Decrease count
    #[inline]
    pub fn down(&self) {
        loop {
            if self.try_down().is_ok() {
                return;
            }

            unsafe {
                asm!("wfe");
            }
        }
    }

    // Try to decrease semaphore value
    #[inline]
    pub fn try_down(&self) -> Result<(), ()> {
        let mut value = self.count.load(Ordering::Acquire);
        if value > 0 {
            value -= 1;
            self.count.store(value, Ordering::Release);

            unsafe {
                asm!("dmb sy");
            }
            Ok(())
        } else {
            self.count.store(value, Ordering::Release);
            Err(())
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
