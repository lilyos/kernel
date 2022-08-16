use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A **M**utual **E**xclusion synchronization device
///
/// # Example
/// ```rust
/// let mtx = Mutex::new(8u32);
///
/// assert!(mtx.try_lock().is_some());
/// ```
#[derive(Debug)]
pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

#[doc(hidden)]
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct MutexGuard<'a, T> {
    data: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    /// Return a new mutex
    ///
    /// # Example
    /// ```
    /// let mtx = Mutex::new(8u32);
    /// ```
    ///
    /// # Arguments
    /// * `value` - The initial value for the mutex
    pub const fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Try to lock the mutex
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if self.lock.swap(true, Ordering::Acquire) {
            None
        } else {
            Some(MutexGuard { data: self })
        }
    }

    /// Lock the mutex, looping if it's currently not available
    ///
    /// # Example
    ///
    /// ```
    /// let mtx = Mutex::new(true);
    ///
    /// {
    ///     let mut a = mtx.lock();
    ///     *a = false;
    /// }
    ///
    /// assert!(!mtx.into_inner());
    /// ```
    pub fn lock(&self) -> MutexGuard<T> {
        loop {
            if let Some(data) = self.try_lock() {
                return data;
            }
        }
    }

    /// Get the inner value of the mutex
    ///
    /// # Example
    /// ```
    /// let mtx = Mutex::new(8u32);
    ///
    /// assert!(mtx.into_inner() == 8u32);
    /// ```
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.data.lock.swap(false, Ordering::Release);
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.data.get() }
    }
}

unsafe impl<T> Sync for Mutex<T> {}
