use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    sync::{Mutex, MutexGuard},
    traits::Init,
};

pub struct Singleton<T: Init> {
    ready: AtomicBool,
    _data: Mutex<T>,
}

impl<T: Init> Singleton<T> {
    pub const fn new(init: T) -> Singleton<T> {
        Singleton {
            ready: AtomicBool::new(false),
            _data: Mutex::new(init),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self._data.lock()
    }

    fn ready(&self) {
        if !self.ready.load(Ordering::Acquire) {
            {
                let mut item = self._data.lock();
                item.pre_init();
                item.init();
                item.post_init();
            }
            self.ready.store(true, Ordering::SeqCst);
        }
    }
}

unsafe impl<T: Init> Sync for Singleton<T> {}
