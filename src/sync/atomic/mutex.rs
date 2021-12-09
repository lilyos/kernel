use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T> {
    _data: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Mutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if !self.lock.swap(true, Ordering::Acquire) {
            Some(MutexGuard { _data: self })
        } else {
            None
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        loop {
            if let Some(data) = self.try_lock() {
                return data;
            }
        }
    }

    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self._data.lock.swap(false, Ordering::Release);
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self._data.data.get() }
    }
}

unsafe impl<T> Sync for Mutex<T> {}
