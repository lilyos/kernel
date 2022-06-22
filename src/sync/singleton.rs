use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    sync::{Mutex, MutexGuard},
    traits::Init,
};

/// A struct to initialize any contained object with the trait `Init`
#[derive(Debug)]
pub struct Singleton<T: Init> {
    initialized: AtomicBool,
    _data: Mutex<T>,
}

impl<T: Init> Singleton<T> {
    /// Construct a new instance with the initial value `T`
    pub const fn new(init: T) -> Singleton<T> {
        Singleton {
            initialized: AtomicBool::new(false),
            _data: Mutex::new(init),
        }
    }

    /// Lock the singleton, returning an instance
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.ready();
        self._data.lock()
    }

    fn ready(&self) {
        if !self.initialized.load(Ordering::Acquire) {
            {
                let mut item = self._data.lock();
                item.pre_init();
                item.init();
                item.post_init();
            }
            self.initialized.store(true, Ordering::Release);
        }
    }
}

// unsafe impl<T: Init> Sync for Singleton<T> {}
