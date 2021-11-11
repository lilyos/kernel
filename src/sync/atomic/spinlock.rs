use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct Spinlock {
    flag: AtomicBool,
}

impl Spinlock {
    pub const fn new() -> Spinlock {
        Spinlock {
            flag: AtomicBool::new(false),
        }
    }

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

        unsafe {
            // Ensure memory accesses are completed
            asm!("dmb");
        }
    }

    #[inline]
    pub fn release(&self) {
        self.flag.store(false, Ordering::SeqCst);

        unsafe {
            // Ensure memory accesses are completed
            asm!("dmb");

            // Pause execution until memory, cache, branch prediction, and tlb operations all complete
            asm!("dsb");

            // Tell all cores to wake the fuck up
            asm!("sev");
        }
    }
}
