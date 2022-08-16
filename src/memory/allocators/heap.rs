use crate::{
    errors::{AllocatorErrorTyped, GenericError, MemoryManagerError},
    get_memory_manager,
    memory::{
        addresses::{Address, AlignedAddress, Virtual},
        utilities::align,
    },
    sync::RwLock,
    traits::{MemoryFlags, MemoryManager},
};

extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};
use log::trace;

use core::{cmp::Ordering, ptr, sync::atomic::AtomicUsize};

use super::NeverAllocator;

/// Internal heap allocator error
#[derive(Clone, Copy, Debug)]
pub enum InternalHeapAllocatorError {
    /// A memory manager error occurred
    MemoryManager(MemoryManagerError),
    /// No large enough region was found
    NoLargeEnoughRegion,
    /// The region is too small for the requested size.
    RegionTooSmall,
    /// The allocation has failed because there is no free memory.
    OutOfMemory,
    /// The deallocation has failed because it was already freed.
    DoubleFree,
}

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
    pub const fn end(&self) -> *const u8 {
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
pub struct Allocator {
    allocated_item_count: AtomicUsize,
    storage: RwLock<Vec<FreeRegion, NeverAllocator>>,
}

static DIV: &str = "================================================================";

type HeapAllocatorError = AllocatorErrorTyped<InternalHeapAllocatorError>;

impl Allocator {
    /// Create a new heap allocator
    ///
    /// # Example
    /// ```
    /// // Assume that heap_start is a *mut u8 and that heap_size is its length in bytes
    /// let heap = Allocator::new();
    /// unsafe { heap.init(heap_start, heap_size) }
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            allocated_item_count: AtomicUsize::new(0),
            storage: RwLock::new(Vec::new_in(NeverAllocator)),
        }
    }

    /// Sort the items based on ascending base
    ///
    /// # Arguments
    /// * `a` - The first item
    /// * `b` - The second item
    fn sort_ascending_base(a: &FreeRegion, b: &FreeRegion) -> Ordering {
        a.start.cmp(&b.start)
    }

    fn try_internal_push(&self, item: FreeRegion) -> Result<(), HeapAllocatorError> {
        let mut data = self.storage.write();
        if data.len() == data.capacity() {
            let original_size = self
                .allocated_item_count
                .load(core::sync::atomic::Ordering::Acquire);
            let new_size = original_size * 2;
            self.allocated_item_count
                .store(new_size, core::sync::atomic::Ordering::Release);

            let v_addr = unsafe {
                get_memory_manager()
                    .allocate_and_map(
                        get_memory_manager().get_current_table().map_err(|e| {
                            HeapAllocatorError::InternalError(
                                InternalHeapAllocatorError::MemoryManager(e),
                            )
                        })?,
                        (*crate::SAFE_UPPER_HALF_RANGE).clone(),
                        MemoryFlags::CACHABLE
                            | MemoryFlags::KERNEL_ONLY
                            | MemoryFlags::READABLE
                            | MemoryFlags::WRITABLE,
                        Self::layout_for_region_array(new_size),
                    )
                    .map_err(|e| {
                        HeapAllocatorError::InternalError(
                            InternalHeapAllocatorError::MemoryManager(e),
                        )
                    })?
            };

            let mut new_vec = unsafe {
                Vec::from_raw_parts_in(
                    Into::<Address<Virtual>>::into(v_addr)
                        .get_inner_ptr_mut()
                        .cast::<FreeRegion>(),
                    0,
                    32,
                    NeverAllocator,
                )
            };

            {
                new_vec.clone_from_slice(&data[..]);

                let old_data = core::mem::replace(&mut *data, new_vec);

                unsafe {
                    get_memory_manager()
                        .deallocate_and_unmap(
                            get_memory_manager().get_current_table().map_err(|e| {
                                HeapAllocatorError::InternalError(
                                    InternalHeapAllocatorError::MemoryManager(e),
                                )
                            })?,
                            AlignedAddress::<Virtual>::new(
                                old_data.into_raw_parts().0 as *const (),
                            )
                            .map_err(HeapAllocatorError::Address)?,
                            Self::layout_for_region_array(original_size),
                        )
                        .map_err(|e| {
                            HeapAllocatorError::InternalError(
                                InternalHeapAllocatorError::MemoryManager(e),
                            )
                        })?;
                };
            }
        }
        data.push(item);
        Ok(())
    }

    const fn layout_for_region_array(count: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                core::mem::size_of::<FreeRegion>() * count,
                core::mem::align_of::<FreeRegion>(),
            )
        }
    }

    /// Initialize the heap allocator
    ///
    /// # Example
    /// ```
    /// // Assume that heap_start is a *mut u8 and that heap_size is its length in bytes
    /// let heap = Allocator::new();
    /// unsafe { heap.init(heap_start, heap_size) }
    /// ```
    ///
    /// # Safety
    /// The provided region must not overlap with any important data
    ///
    /// # Errors
    /// This may return errors from the Memory Manager if mapping fails.
    /// It may also return errors if there is no free physical memory.
    pub unsafe fn init(&self, start: *mut u8, size: usize) -> Result<(), HeapAllocatorError> {
        self.allocated_item_count
            .store(32, core::sync::atomic::Ordering::Release);

        let v_addr = get_memory_manager()
            .allocate_and_map(
                get_memory_manager().get_current_table().map_err(|e| {
                    HeapAllocatorError::InternalError(InternalHeapAllocatorError::MemoryManager(e))
                })?,
                (*crate::SAFE_UPPER_HALF_RANGE).clone(),
                MemoryFlags::CACHABLE
                    | MemoryFlags::KERNEL_ONLY
                    | MemoryFlags::READABLE
                    | MemoryFlags::WRITABLE,
                Self::layout_for_region_array(32),
            )
            .map_err(|e| {
                HeapAllocatorError::InternalError(InternalHeapAllocatorError::MemoryManager(e))
            })?;

        {
            let mut lock = self.storage.write();
            *lock = Vec::from_raw_parts_in(
                Into::<Address<Virtual>>::into(v_addr)
                    .get_inner_ptr_mut()
                    .cast::<FreeRegion>(),
                0,
                32,
                NeverAllocator,
            );
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
    ///
    /// # Errors
    /// This will return errors if there is not enough room in the Vec and it is unable to allocate.
    pub unsafe fn add_free_region(
        &self,
        addr: *mut u8,
        size: usize,
    ) -> Result<(), HeapAllocatorError> {
        self.join_nearby();
        trace!("Sorted free regions");
        trace!("Pushing new free region");
        self.try_internal_push(FreeRegion::new(addr, size))
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
        let items = self.storage.read();
        for (index, item) in items.iter().enumerate() {
            if let Ok(alloc_start) = Self::check_region_allocation(item, size, alignment) {
                return Some((
                    item as *const FreeRegion as *mut FreeRegion,
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
        region: &FreeRegion,
        size: usize,
        alignment: usize,
    ) -> Result<usize, HeapAllocatorError> {
        let alloc_start = align(region.start as usize, alignment);
        let alloc_end = alloc_start
            .checked_add(size)
            .ok_or(HeapAllocatorError::Generic(
                GenericError::IntOverflowOrUnderflow,
            ))?;

        if alloc_end > region.end() as usize {
            return Err(HeapAllocatorError::InternalError(
                InternalHeapAllocatorError::RegionTooSmall,
            ));
        }

        Ok(alloc_start)
    }

    /// Join nearby regions by adding an item's start and checking if it equals the
    /// next item's start.
    fn join_nearby(&self) {
        let mut items = self.storage.write();
        loop {
            items.sort_by(Self::sort_ascending_base);

            let mut tbreak = true;
            for index in 0..items.len() {
                let b = match items.get(index + 1) {
                    Some(v) => v.clone(),
                    _ => continue,
                };

                let a = match items.get_mut(index) {
                    Some(v) => v,
                    _ => continue,
                };

                if unsafe { a.start.add(a.size) } == b.start {
                    let n_size = b.size;
                    a.size += n_size;
                    let _removed = items.drain(index + 1..=index + 1);

                    tbreak = false;
                }
            }

            if tbreak {
                break;
            }
        }
    }
}

impl core::fmt::Display for Allocator {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "{}", DIV)?;

        let items = self.storage.read();
        for item in items.iter() {
            writeln!(f, "Allocator Node {:#?}", item)?;
        }

        writeln!(f, "{}", DIV)
    }
}

unsafe impl GlobalAlloc for Allocator {
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
            let end = match alloc_start.checked_add(size) {
                Some(v) => v,
                None => return core::ptr::null_mut(),
            };

            let spare = region_end as usize - end;

            if spare > 0 {
                let _ = self
                    .add_free_region((alloc_start as *mut u8).add(size), spare)
                    .is_ok();
            }

            {
                let mut storage = self.storage.write();
                drop(storage.drain(region_idx..=region_idx));

                storage.sort_by(Self::sort_ascending_base);
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

        let _ = self.add_free_region(ptr, size).is_ok();
        self.join_nearby();
    }
}
