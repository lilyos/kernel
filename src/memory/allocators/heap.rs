use super::{align, AllocatorError};

use crate::{collections::GrowableSlice, sync::Mutex};

extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};

use core::{cmp::Ordering, ptr};

/// A struct representing a free region in the heap allocator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeRegion {
    start: *mut u8,
    size: usize,
}

impl FreeRegion {
    /// Make a new free region
    ///
    /// # Arguments
    /// * `start` - The start of the region
    /// * `size` - The size of the region
    pub const fn new(start: *mut u8, size: usize) -> Self {
        Self { start, size }
    }

    /// Get the end of the region
    pub fn end(&self) -> *const u8 {
        unsafe { self.start.add(self.size) }
    }
}

impl PartialOrd for FreeRegion {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some((self.start as usize).cmp(&(other.start as usize)))
    }
}

impl core::cmp::Ord for FreeRegion {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.start as usize).cmp(&(other.start as usize))
    }
}

/// The Lotus OS Heap Allocator
pub struct HeapAllocator {
    storage: Mutex<GrowableSlice<FreeRegion>>,
}

static DIV: &str = "================================================================";

impl HeapAllocator {
    /// Create a new heap allocator
    ///
    /// # Example
    /// ```
    /// // Assume that heap_start is a *mut u8 and that heap_size is its length in bytes
    /// let heap = HeapAllocator::new();
    /// unsafe { heap.init(heap_start, heap_size) }
    /// ```
    pub const fn new() -> Self {
        HeapAllocator {
            storage: Mutex::new(GrowableSlice::new()),
        }
    }

    /// Sort the items based on ascending base
    ///
    /// # Arguments
    /// * `a` - The first item
    /// * `b` - The second item
    fn sort_ascending_base(a: &Option<FreeRegion>, b: &Option<FreeRegion>) -> Ordering {
        if a.is_none() && b.is_none() {
            Ordering::Equal
        } else if a.is_none() && b.is_some() {
            Ordering::Greater
        } else if b.is_none() && a.is_some() {
            Ordering::Less
        } else {
            a.as_ref().unwrap().start.cmp(&b.as_ref().unwrap().start)
        }
    }

    /// Initialize the heap allocator
    ///
    /// # Example
    /// ```
    /// // Assume that heap_start is a *mut u8 and that heap_size is its length in bytes
    /// let heap = HeapAllocator::new();
    /// unsafe { heap.init(heap_start, heap_size) }
    /// ```
    ///
    /// # Safety
    /// The provided region must not overlap with any important data
    pub unsafe fn init(&self, start: *mut u8, size: usize) -> Result<(), AllocatorError> {
        {
            let mut storage = self.storage.lock();
            storage.init()?;
        }
        self.add_free_region(start, size)?;
        Ok(())
    }

    /// Add a free region
    ///
    /// # Arguments
    /// * `addr` - The address of the free region
    /// * `size` - The size of the free region
    ///
    /// # Safety
    /// The provided region must not overlap with any important data
    pub unsafe fn add_free_region(&self, addr: *mut u8, size: usize) -> Result<(), AllocatorError> {
        self.join_nearby();
        let items = &mut *self.storage.lock();
        items.push(FreeRegion::new(addr, size))
    }

    /// Find a region with the specified size and alignment
    ///
    /// # Arguments
    /// * `size` - The size to find
    /// * `alignment` - The desired alignment
    ///
    /// # Returns
    /// * A pointer to the found region
    /// * The starting address for the specified alignment
    /// * The index for the region in the internal storage
    pub fn find_region(
        &self,
        size: usize,
        alignment: usize,
    ) -> Option<(*mut FreeRegion, usize, usize)> {
        let items = &mut *self.storage.lock();
        items.sort(Self::sort_ascending_base);
        for (index, item) in items
            .storage
            .iter()
            .filter(|i| i.is_some())
            .map(|i| i.as_ref().unwrap())
            .enumerate()
        {
            if let Ok(alloc_start) = self.check_region_allocation(item, size, alignment) {
                return Some((
                    &*item as *const FreeRegion as *mut FreeRegion,
                    alloc_start,
                    index,
                ));
            }
        }

        None
    }

    /// Checks the validitity of a specified region for a certain size and alignment
    ///
    /// # Arguments
    /// * `region` - The region to test against
    /// * `size` - The desired size
    /// * `alignment` - The desired alignment
    ///
    /// # Returns
    /// * The starting address for the specified region
    fn check_region_allocation(
        &self,
        region: &FreeRegion,
        size: usize,
        alignment: usize,
    ) -> Result<usize, AllocatorError> {
        let alloc_start = align(region.start as usize, alignment);
        let alloc_end = alloc_start
            .checked_add(size)
            .ok_or(AllocatorError::InternalError(
                "Integer overflow in HeapAllocator::check_region_allocation",
            ))?;

        if alloc_end > region.end() as usize {
            return Err(AllocatorError::RegionTooSmall);
        }

        Ok(alloc_start)
    }

    /// Join nearby regions by adding an item's start and checking if it equals the
    /// next item's start.
    fn join_nearby(&self) {
        let mut items = self.storage.lock();
        loop {
            items.sort(Self::sort_ascending_base);

            let mut tbreak = true;
            for index in 0..items.present() {
                let b = match items.storage.get(index + 1) {
                    Some(Some(v)) => v.clone(),
                    _ => continue,
                };

                let a = match items.storage.get_mut(index) {
                    Some(Some(v)) => v,
                    _ => continue,
                };

                if unsafe { a.start.add(a.size) } == b.start {
                    let n_size = b.size;
                    a.size += n_size;
                    let _ = items.pop(index + 1);

                    tbreak = false;
                }
            }

            if tbreak {
                break;
            }
        }
    }
}

impl core::fmt::Display for HeapAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "{}", DIV)?;

        let items = self.storage.lock();
        for item in items
            .storage
            .iter()
            .filter(|i| i.is_some())
            .map(|i| i.as_ref().unwrap())
        {
            writeln!(f, "Allocator Node {:#?}", item)?;
        }

        writeln!(f, "{}", DIV)
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    /// I really don't want to explain this, buttttttttttttttttttttt
    /// It
    /// * Aligns the layout
    /// * Finds an appropriate region
    /// * Does some math to calculate the spare space in the region
    /// * Adds the spare space as a new region
    /// * Sorts the regions based on ascending base
    /// * Returns a pointer to the region and then pops it
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = (layout.size(), layout.align());
        if let Some((region_ptr, alloc_start, region_idx)) = self.find_region(size, align) {
            let region = &mut *region_ptr;
            let region_end = region.end();
            let end = alloc_start
                .checked_add(size)
                .expect("Integer overflow when calculating end of region");

            let spare = region_end as usize - end;

            if spare > 0 {
                let _ = self
                    .add_free_region((alloc_start as *mut u8).add(size), spare)
                    .is_ok();
            }

            {
                let mut storage = self.storage.lock();
                let _ = storage.pop(region_idx);
                storage.sort(Self::sort_ascending_base);
            }

            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    /// This
    /// * Aligns the layout
    /// * Adds it to the free region list
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();

        self.add_free_region(ptr, size)
            .expect("Failed to deallocate memory, {layout:#?}");
        self.join_nearby();
    }
}
