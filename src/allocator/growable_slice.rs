use core::ops::{Index, IndexMut};

use crate::{
    allocator::{AllocatorError, PageSize},
    println,
};

pub struct GrowableSlice<T: 'static + Copy> {
    pub storage: &'static mut [Option<T>],
}

impl<T: Copy + PartialEq> GrowableSlice<T> {
    pub const fn new() -> Self {
        Self {
            // storage: unsafe { core::slice::from_raw_parts_mut(core::ptr::null_mut(), 0) },
            storage: &mut [],
        }
    }

    pub unsafe fn init(&mut self, start: *mut u8, size: usize) {
        self.storage = core::slice::from_raw_parts_mut(
            start as *mut Option<T>,
            size / core::mem::size_of::<T>(),
        );
        self.storage.fill(None);
    }

    pub fn push(&mut self, to_add: T) -> Result<(), AllocatorError> {
        println!("Testing storage actions");
        if self.present() == self.storage.len() {
            self.grow_storage()?;
        } else if self.present() + 1 < self.storage.len() {
            let _ = self.shrink_storage().ok();
        }

        println!("Iterating to push");
        for (ind, item) in self.storage.iter().enumerate() {
            if *item == None {
                self.storage[ind] = Some(to_add);
                println!("Pushed");
                return Ok(());
            }
        }
        println!("Internal storage full");
        Err(AllocatorError::InternalError(40))
    }

    pub fn find<F>(&self, pred: F) -> Option<T>
    where
        F: Fn(T) -> bool,
    {
        for i in self.storage.iter().filter(|i| i.is_some()) {
            if pred(i.unwrap()) {
                return *i;
            }
        }
        None
    }

    pub fn present(&self) -> usize {
        let mut total = 0;
        for i in self.storage.iter() {
            if i.is_some() {
                total += 1;
            }
        }
        total
    }

    /// Allocates new area for storage, copies current to it, then deallocates the old one. Returns how many bytes were allocated
    pub fn grow_storage(&mut self) -> Result<usize, AllocatorError> {
        let storage_len = self.storage.len();
        let new_size = (storage_len + 8) * core::mem::size_of::<T>();

        let (alloc, alloc_size) = crate::PAGE_ALLOCATOR.alloc(PageSize::Normal, new_size / 4096)?;

        let new = unsafe { core::slice::from_raw_parts_mut(alloc as *mut Option<T>, alloc_size) };
        new.fill(None);

        new[0..storage_len].copy_from_slice(&self.storage[..]);

        crate::PAGE_ALLOCATOR.dealloc(self.storage.as_ptr() as *mut u8);

        self.storage = new;

        Ok(new_size - storage_len * core::mem::size_of::<T>())
    }

    /// Joins nearby regions then moves them all to the beginning.
    /// Allocates a new region for the newly rejoined regions and deallocates the old one.
    /// Returns how many bytes were deallocated
    pub fn shrink_storage(&mut self) -> Result<usize, AllocatorError> {
        if self.storage.len() == self.present() {
            Err(AllocatorError::InternalStorageFull)
        } else {
            Err(AllocatorError::NotImplemented)
        }
    }
}

impl<T: Copy> Index<usize> for GrowableSlice<T> {
    type Output = Option<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.storage[index]
    }
}

impl<T: Copy> IndexMut<usize> for GrowableSlice<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.storage[index]
    }
}

unsafe impl<T: Copy> Sync for GrowableSlice<T> {}
