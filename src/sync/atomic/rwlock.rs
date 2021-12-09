use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[derive(Debug)]
pub struct RWLock<T: ?Sized> {
    write_lock: AtomicBool,
    read_locks: AtomicU32,
    data: UnsafeCell<T>,
}

// Write handle
#[derive(Debug)]
pub struct WriteLockGuard<'a, T> {
    _data: &'a RWLock<T>,
}

// Read handle
#[derive(Debug)]
pub struct ReadLockGuard<'a, T> {
    _data: &'a RWLock<T>,
}

impl<T> RWLock<T> {
    pub const fn new(value: T) -> Self {
        RWLock {
            write_lock: AtomicBool::new(false),
            read_locks: AtomicU32::new(0),
            data: UnsafeCell::new(value),
        }
    }

    // Try to get write lock
    pub fn try_lock(&self) -> Option<WriteLockGuard<T>> {
        // Check if any read locks are present
        if self.read_locks.load(Ordering::Relaxed) > 0 {
            return None;
        }

        // Set status to locked
        if !self.write_lock.swap(true, Ordering::Acquire) {
            Some(WriteLockGuard { _data: self })
        } else {
            None
        }
    }

    // Get write lock to the data, won't work unless there's no read locks
    pub fn lock(&self) -> WriteLockGuard<T> {
        loop {
            if let Some(write_guard) = self.try_lock() {
                return write_guard;
            }
        }
    }

    // Check if data is available to read, returning none if not
    pub fn try_read(&self) -> Option<ReadLockGuard<T>> {
        if self.write_lock.load(Ordering::Relaxed) {
            None
        } else {
            self.read_locks.fetch_add(1, Ordering::Acquire);
            Some(ReadLockGuard { _data: self })
        }
    }

    // Wait until data is available, then return it
    pub fn read(&self) -> ReadLockGuard<T> {
        loop {
            if let Some(read_guard) = self.try_read() {
                return read_guard;
            }
        }
    }

    // Reference to inner data, only safe when used once and then goes out of scope
    pub unsafe fn as_ref_unchecked(&self) -> &T {
        &*self.data.get()
    }

    // Mutex into inner value
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }
}

impl<T> Drop for WriteLockGuard<'_, T> {
    fn drop(&mut self) {
        self._data.write_lock.store(false, Ordering::Release);
    }
}

impl<T> Drop for ReadLockGuard<'_, T> {
    fn drop(&mut self) {
        self._data.read_locks.fetch_sub(1, Ordering::Release);
    }
}

impl<T> Deref for WriteLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

impl<T> DerefMut for WriteLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self._data.data.get() }
    }
}

impl<T> Deref for ReadLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

unsafe impl<T> Sync for RWLock<T> {}
