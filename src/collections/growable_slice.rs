use core::cmp::Ordering;

use crate::memory::allocators::AllocatorError;

/// A Vector-like container that makes use of the phsyical allocator
#[derive(Debug)]
pub struct GrowableSlice<T: 'static> {
    /// The inner storage of the slice
    pub storage: &'static mut [Option<T>],
    block_start: usize,
    blocks_allocated: usize,
}

impl<T: PartialEq + Clone + core::fmt::Debug + core::cmp::Ord> GrowableSlice<T> {
    /// Create a new growable slice
    pub const fn new() -> Self {
        Self {
            storage: &mut [],
            block_start: 0,
            blocks_allocated: 0,
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
        let (alloc, block_start) = crate::PHYSICAL_ALLOCATOR.alloc(1)?;
        self.storage = core::slice::from_raw_parts_mut(
            alloc as *mut Option<T>,
            1024 / core::mem::size_of::<T>(),
        );
        self.storage.fill(None);
        self.block_start = block_start;
        self.blocks_allocated = 1;
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
        let (alloc, alloc_blocks) = crate::PHYSICAL_ALLOCATOR.alloc(self.blocks_allocated * 2)?;

        let new = unsafe {
            core::slice::from_raw_parts_mut(alloc as *mut Option<T>, alloc_blocks * 1024)
        };
        new.fill(None);

        new[0..self.storage.len()].clone_from_slice(self.storage);

        crate::PHYSICAL_ALLOCATOR.dealloc(self.block_start, self.blocks_allocated)?;

        self.storage = new;
        self.blocks_allocated = alloc_blocks;

        Ok((self.blocks_allocated / 2) * 1024)
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
        if self.storage.len() != self.present() && self.blocks_allocated > 1 {
            self.sort(Self::none_to_end);
            let slice_bytes = self.present() * core::mem::size_of::<T>();
            let bytes_alloc = self.blocks_allocated * 1024;
            let diff = bytes_alloc - slice_bytes;
            if diff % 1024 == 0 {
                let spare = diff / 1024;

                let new_size = self.blocks_allocated - spare;

                let (alloc, block_start) = crate::PHYSICAL_ALLOCATOR.alloc(new_size)?;
                let storage = unsafe {
                    core::slice::from_raw_parts_mut(
                        alloc as *mut Option<T>,
                        (new_size * 1024) / core::mem::size_of::<T>(),
                    )
                };
                storage.fill(None);

                storage.clone_from_slice(&self.storage[0..self.present()]);

                crate::PHYSICAL_ALLOCATOR.dealloc(self.block_start, self.blocks_allocated)?;

                self.storage = storage;

                self.block_start = block_start;
                self.blocks_allocated = new_size;
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

unsafe impl<T: Copy> Sync for GrowableSlice<T> {}
