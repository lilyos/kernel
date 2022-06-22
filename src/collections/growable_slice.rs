use core::cmp::Ordering;

use crate::{
    arch::PHYSICAL_ALLOCATOR,
    memory::{allocators::AllocGuard, errors::AllocatorError},
    traits::PhysicalMemoryAllocator,
};

/// A Vector-like container that makes use of the phsyical allocator
#[derive(Debug)]
pub struct GrowableSlice<'a, T> {
    /// The inner storage of the slice
    pub storage: &'a mut [Option<T>],
    guard: Option<AllocGuard<'a>>,
}

impl<'a, T: PartialEq + Clone + core::fmt::Debug + core::cmp::Ord> GrowableSlice<'a, T> {
    /// Create a new growable slice
    pub const fn new() -> Self {
        Self {
            storage: &mut [],
            guard: None,
        }
    }

    /// Get the guard
    const fn get_guard(&self) -> Result<&'a AllocGuard, AllocatorError> {
        match &self.guard {
            Some(v) => Ok(v),
            None => Err(AllocatorError::Uninitialized),
        }
    }

    /// Take the guard
    fn take_guard(&mut self) -> Result<AllocGuard, AllocatorError> {
        match self.guard.take() {
            Some(v) => Ok(v),
            None => Err(AllocatorError::Uninitialized),
        }
    }

    /// Initializes the GrowableSlice.
    /// This is unsafe because it is intended to be infallible.
    /// If it fails, behavior is undefined.
    ///
    /// # Arguments
    /// * `start` - Start of a region as a `*mut u8`
    /// * `size` - The length of the area specified
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
    /// let data = GrowableSlice::new::<u8>();
    /// unsafe { data.init(start, size) }
    /// ```
    ///
    /// # Safety
    /// All writes must succeed, and the allocated area must be empty, unless you want data gone
    pub unsafe fn init(&mut self) -> Result<(), AllocatorError> {
        let guard = PHYSICAL_ALLOCATOR.alloc(1)?;
        self.storage = core::slice::from_raw_parts_mut(
            guard.address_mut() as *mut Option<T>,
            1024 / core::mem::size_of::<T>(),
        );
        self.storage.fill(None);
        self.guard = Some(guard);
        Ok(())
    }

    /// Push an item to the storage, returning the error if any occurs
    ///
    /// # Arguments
    /// * `to_add` - The item to add
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
    /// let data = GrowableSlice::new::<u8>();
    /// unsafe { data.init(start, size) }
    ///
    /// data.push(1);
    /// ```
    pub fn push(&mut self, to_add: T) -> Result<(), AllocatorError> {
        if self.present() == self.storage.len() {
            self.grow_storage()?;
        } else if self.present() + 1 < self.storage.len() {
            let _ = self.shrink_storage().ok();
        }
        for (ind, item) in self.storage.iter().enumerate() {
            if *item == None {
                self.storage[ind] = Some(to_add);
                return Ok(());
            }
        }

        Err(AllocatorError::InternalError("Code shouldn't have reached here, GrowableSlice::push, as it grows if it's too small. This should be impossible"))
    }

    /// Pop an item from the storage
    ///
    /// # Arguments
    /// * `index` - The index to pop from
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
    /// let data = GrowableSlice::new::<u8>();
    /// unsafe { data.init(start, size) }
    ///
    /// data.push(1);
    /// assert!(data.pop(0) == Some(1));
    /// ```
    pub fn pop(&mut self, index: usize) -> Option<T> {
        let v = self.storage[index].clone();
        self.storage[index] = None;
        v
    }

    /// Returns how many items are currently in the internal storage
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
    /// let data = GrowableSlice::new::<u8>();
    /// unsafe { data.init(start, size) }
    ///
    /// data.push(1);
    /// assert!(data.present() == 1);
    ///
    /// data.push(2);
    /// assert!(data.present() == 2);
    /// ```
    pub fn present(&self) -> usize {
        let mut total = 0;
        for i in self.storage.iter() {
            if i.is_some() {
                total += 1;
            }
        }
        total
    }

    /// Allocates new area for storage, copies current to it, then deallocates the old one
    /// It will either return how many bytes were allocated or an error
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is `1`
    /// let data = GrowableSlice::new::<u8>();
    /// unsafe { data.init(start, size) }
    ///
    /// data.push(1);
    ///
    /// assert!(data.push(2).is_err());
    ///
    /// assert!(data.grow_storage().is_ok());
    ///
    /// assert!(data.push(2) == Ok(()));
    /// ```
    pub fn grow_storage(&mut self) -> Result<usize, AllocatorError> {
        let kilos_allocated = self.get_guard()?.kilos_allocated();
        let guard = PHYSICAL_ALLOCATOR.alloc(kilos_allocated * 2)?;

        let new = unsafe {
            core::slice::from_raw_parts_mut(
                guard.address_mut() as *mut Option<T>,
                guard.kilos_allocated() * 1024,
            )
        };
        new.fill(None);

        new[0..self.storage.len()].clone_from_slice(self.storage);

        {
            let guard_old = self.take_guard()?;
            drop(guard_old);
        }

        self.storage = new;
        self.guard = Some(guard);

        Ok(self.get_guard()?.kilos_allocated() * 1024)
    }

    /// Moves items towards the beginning of the region.
    /// Returns how many bytes were deallocated or an error describing why the shrink failed.
    /// It is guaranteed that it will never shrink below `4096` bytes (The size of a page on x86_64).
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is `8192` (2x the size of a page on x86_64)
    /// let data = GrowableSlice::new::<u8>();
    /// unsafe { data.init(start, size) }
    ///
    /// data.push(1);
    ///
    /// assert!(data.shrink_storage().is_ok());
    ///
    /// assert!(data.shrink_storage().is_err()));
    pub fn shrink_storage(&mut self) -> Result<usize, AllocatorError> {
        let kilos_allocated = self.get_guard()?.kilos_allocated();

        if self.storage.len() != self.present() && kilos_allocated > 1 {
            self.sort(Self::none_to_end);
            let slice_bytes = self.present() * core::mem::size_of::<T>();
            let bytes_alloc = kilos_allocated * 1024;
            let diff = bytes_alloc - slice_bytes;
            if diff % 1024 == 0 {
                let spare = diff / 1024;

                let new_size = kilos_allocated - spare;

                let guard = PHYSICAL_ALLOCATOR.alloc(new_size)?;
                let storage = unsafe {
                    core::slice::from_raw_parts_mut(
                        guard.address_mut() as *mut Option<T>,
                        (new_size * 1024) / core::mem::size_of::<T>(),
                    )
                };
                storage.fill(None);

                storage.clone_from_slice(&self.storage[0..self.present()]);

                {
                    let guard_old = self.take_guard()?;
                    drop(guard_old);
                }

                self.storage = storage;

                self.guard = Some(guard);
                Ok(diff)
            } else {
                Err(AllocatorError::CompactionTooLow)
            }
        } else {
            Err(AllocatorError::InternalStorageFull)
        }
    }

    /// Sort the slice in place using the provided function
    pub fn sort<F>(&mut self, fun: F)
    where
        F: FnMut(&Option<T>, &Option<T>) -> Ordering,
    {
        self.storage.sort_unstable_by(fun);
    }

    fn none_to_end(a: &Option<T>, b: &Option<T>) -> Ordering {
        if a.is_none() && b.is_some() {
            Ordering::Greater
        } else if b.is_none() && a.is_some() {
            Ordering::Less
        } else {
            a.cmp(b)
        }
    }
}
