use core::sync::atomic::{AtomicUsize, Ordering};

use limine_protocol::structures::memory_map_entry::{EntryType, MemoryMapEntry};
use log::{debug, info};

use crate::{
    collections::BitSlice,
    memory::{allocators::AllocGuard, errors::AllocatorError, utilities::align},
    sync::RwLock,
    traits::PhysicalMemoryAllocator,
};

/// The Lotus OS Page Allocator
pub struct PageAllocator<'a> {
    pages: AtomicUsize,
    region: *const u8,
    scratch: RwLock<BitSlice<'a>>,
}

impl<'a> core::fmt::Display for PageAllocator<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "Allocator {{\n\tpages: {},\n\tregion: {:?},\n\tscratch: {{ .. }},\n}}",
            self.pages.load(Ordering::SeqCst),
            self.region
        )
    }
}

impl<'a> PageAllocator<'a> {
    const BLOCK_SIZE: usize = 4096;
    /// Return a new page allocator
    ///
    /// # Example
    /// ```
    /// // Assume mmap is a slice of MemoryDescriptor
    /// let alloc = PageAllocator::new();
    /// unsafe { alloc.init(mmap) }
    /// ```
    pub const fn new() -> Self {
        Self {
            pages: AtomicUsize::new(0),
            region: core::ptr::null(),
            scratch: RwLock::new(BitSlice::new()),
        }
    }

    /// Get the amount of used pages
    pub fn get_used(&self) -> usize {
        let mut total = 0;
        {
            let scratch = self.scratch.read();
            for item in scratch.iter() {
                if item {
                    total += 1;
                }
            }
        }
        total
    }

    /// Find a series of zones with a specific size
    ///
    /// # Arguments
    /// * `block_count` - The amount of blocks to find
    fn get_zone_with_size(&self, block_count: usize) -> Option<usize> {
        let mut block = 0;
        let mut consecutive = 0;
        {
            let scratch = self.scratch.read();
            let iter = scratch.iter();
            for (index, item) in iter.enumerate() {
                if consecutive == block_count {
                    return Some(block);
                } else if !item {
                    consecutive += 1;
                } else {
                    block = index + 1;
                    consecutive = 0;
                }
            }
        }

        None
    }

    /// Set blocks in a specified range
    ///
    /// # Arguments
    /// * `blocks_to_set` - How many blocks to set
    /// * `starting_pos` - What block to start at
    /// * `value` - The value to set
    fn set_range(&self, blocks_to_set: usize, starting_pos: usize, value: bool) {
        assert!(blocks_to_set < self.pages.load(Ordering::SeqCst));
        assert!(starting_pos < (self.pages.load(Ordering::SeqCst) * Self::BLOCK_SIZE) / 8);
        let mut scratch = self.scratch.write();

        for i in starting_pos..(starting_pos + blocks_to_set) {
            scratch.set(i, value);
        }
    }
}

impl<'a> PhysicalMemoryAllocator for PageAllocator<'a> {
    type PAResult<T> = Result<T, AllocatorError>;

    /// Initialize the allocator
    ///
    /// # Arguments
    /// * `mmap` - Slice of memory descriptors
    ///
    /// # Example
    /// ```
    /// // Assume mmap is a slice of MemoryDescriptor
    /// let alloc = PageAllocator::new();
    /// unsafe { alloc.init(mmap) }
    /// ```
    unsafe fn init(&self, mmap: &[&MemoryMapEntry]) -> Result<(), AllocatorError> {
        assert!(!mmap.is_empty());
        let mut pages: usize = 0;
        let mut end: usize = 0;

        for mentry in mmap.iter() {
            let mend: usize = mentry.end().try_into().unwrap();
            if mend > end {
                end = mend as usize;
            }
            pages += (mend - TryInto::<usize>::try_into(mentry.base).unwrap()) / Self::BLOCK_SIZE;
        }
        let scratch_bytes = align(end / 4096, 8) / 8;
        self.pages.store(pages, Ordering::SeqCst);

        let scratch_entry = mmap.iter().find(|i| i.base >= 4096).unwrap();

        let scratch_start: usize = scratch_entry.base.try_into().unwrap();

        let scratch_end = align(scratch_start + scratch_bytes, Self::BLOCK_SIZE) - 1;

        {
            let mut sscratch = self.scratch.write();
            sscratch.init(scratch_start as *mut u8, scratch_bytes);
            sscratch.set(0, true);
            for i in mmap.iter() {
                for a in (i.base..i.end()).step_by(4096) {
                    let a: usize = a.try_into().unwrap();
                    if a < 4096
                        || (a >= scratch_start && a < scratch_end)
                        || i.kind == EntryType::Reserved
                        || i.kind == EntryType::AcpiNonVolatile
                        || i.kind == EntryType::BadMemory
                        || i.kind == EntryType::Framebuffer
                        || i.kind == EntryType::KernelAndModules
                    {
                        sscratch.set(a / 4096, true)
                    }
                }
            }
        }

        let used = self.get_used();
        let free = pages - used;
        info!(
            "{}/{} usable ({}% free)",
            free,
            pages,
            ((free as f64 / pages as f64) * 100.0) as usize,
        );
        debug!("Using {}kb for page bitmap", scratch_bytes / 1024);

        Ok(())
    }

    /// Allocate physical memory, returning a pointer to the allocated memory and the block that the allocation started on
    ///
    /// # Arguments
    /// * `size` - Size of memory desired in kilobytes
    ///
    /// # Example
    /// ```
    /// // Assume mmap is a slice of MemoryDescriptor
    /// let alloc = PageAllocator::new();
    /// unsafe { alloc.init(mmap) }
    ///
    /// let (ptr, size) = alloc.alloc(4).unwrap();
    /// ```
    fn alloc<'b>(&self, size: usize) -> Result<AllocGuard<'b>, AllocatorError> {
        assert!(size < (self.pages.load(Ordering::SeqCst) * Self::BLOCK_SIZE));

        let pages = align(size * 1024, Self::BLOCK_SIZE) / Self::BLOCK_SIZE;

        let found = match self.get_zone_with_size(pages) {
            Some(v) => v,
            None => {
                if self.get_used() == self.pages.load(Ordering::SeqCst) {
                    return Err(AllocatorError::OutOfMemory);
                } else {
                    return Err(AllocatorError::NoLargeEnoughRegion);
                }
            }
        };

        assert!(found != 0, "The first page was found as an allocation");

        self.set_range(pages, found, true);

        Ok(unsafe {
            AllocGuard::new(
                found,
                size,
                self.region.add(found * Self::BLOCK_SIZE) as *mut u8,
            )
        })
    }

    /// Deallocate physical memory, freeing it
    ///
    /// # Arguments
    /// * `kilos_allocated` - How many blocks/kilobytes were allocated
    /// * `block_start` - The block the allocation started on
    fn dealloc(&self, block_start: usize, kilos_allocated: usize) -> Result<(), AllocatorError> {
        assert!(block_start < self.pages.load(Ordering::SeqCst));

        let block_count = align(kilos_allocated * 1024, Self::BLOCK_SIZE) / Self::BLOCK_SIZE;

        {
            let scratch = self.scratch.read();
            if !scratch[block_start] {
                return Err(AllocatorError::DoubleFree);
            }
        }

        self.set_range(block_count, block_start, false);

        Ok(())
    }
}

unsafe impl<'a> Sync for PageAllocator<'a> {}