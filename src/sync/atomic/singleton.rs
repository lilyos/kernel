use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use crate::traits::Init;

pub struct Singleton<T: Init> {
    ready: AtomicBool,
    _data: UnsafeCell<T>,
}

impl<T: Init> Singleton<T> {
    pub const fn new(init: T) -> Singleton<T> {
        Singleton {
            ready: AtomicBool::new(false),
            _data: UnsafeCell::new(init),
        }
    }

    fn ready(&self) {
        if !self.ready.load(Ordering::Acquire) {
            let item = unsafe { &mut *self._data.get() };
            item.pre_init();
            item.init();
            item.post_init();
            self.ready.store(true, Ordering::SeqCst);
        }
    }
}

impl<T: Init> Deref for Singleton<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.ready();
        unsafe { &*self._data.get() }
    }
}

impl<T: Init> DerefMut for Singleton<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.ready();
        unsafe { &mut *self._data.get() }
    }
}

unsafe impl<T: Init> Sync for Singleton<T> {}
