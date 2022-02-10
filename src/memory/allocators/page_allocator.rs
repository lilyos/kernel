use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    collections::BitSlice,
    memory::allocators::{align, AllocatorError, MemoryKind},
    peripherals::uart::println,
    sync::Mutex,
};

use super::{MemoryDescriptor, MemoryEntry, PhysicalAllocatorImpl};

/// The Lotus OS Page Allocator
pub struct PageAllocator<'a> {
    pages: AtomicUsize,
    region: *const u8,
    scratch: Mutex<BitSlice<'a>>,
}

impl<'a> core::fmt::Display for PageAllocator<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "BuddyAllocator {{\n\tpages: {},\n\tregion: {:?},\n\tscratch: {{ .. }},\n}}",
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
            scratch: Mutex::new(BitSlice::new()),
        }
    }

    /// Get the amount of used pages
    pub fn get_used(&self) -> usize {
        let mut total = 0;
        {
            let mut scratch = self.scratch.lock();
            for item in &mut *scratch {
                if item {
                    total += 1;
                }
            }
            scratch.reset_iterator();
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
            let mut scratch = self.scratch.lock();
            for (index, item) in (&mut *scratch).enumerate() {
                if consecutive == block_count {
                    return Some(block);
                } else if !item {
                    consecutive += 1;
                } else {
                    block = index;
                    consecutive = 0;
                }
            }
            scratch.reset_iterator();
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
        let mut scratch = self.scratch.lock();

        for i in 0..blocks_to_set {
            for x in
                (starting_pos << (blocks_to_set - i))..((starting_pos + 1) << (blocks_to_set - i))
            {
                scratch.set(x, value);
            }

            if value {
                for i in blocks_to_set..self.pages.load(Ordering::SeqCst) {
                    if scratch[starting_pos >> (i - blocks_to_set)] {
                        break;
                    }
                    scratch.set(starting_pos >> (i - blocks_to_set), true);
                }
            } else {
                for i in blocks_to_set..self.pages.load(Ordering::SeqCst) {
                    scratch.set(starting_pos >> (i - blocks_to_set), false);
                    if scratch[(starting_pos >> (i - blocks_to_set)) ^ 1] {
                        break;
                    }
                }
            }
        }
    }
}

impl<'a> PhysicalAllocatorImpl for PageAllocator<'a> {
    type PAResult<T> = Result<T, AllocatorError>;

    /// Initializes the page allocator
    ///
    /// # Example
    /// ```
    /// // Assume mmap is a slice of MemoryDescriptor
    /// let alloc = PageAllocator::new();
    /// unsafe { alloc.init(mmap) }
    /// ```
    unsafe fn init(&self, mmap: &[MemoryDescriptor]) -> Result<(), AllocatorError> {
        assert!(!mmap.is_empty());
        // println!("{:?}", mmap);
        let mut pages = 0;
        let mut end = 0;

        for i in mmap.iter() {
            let mentry: MemoryEntry = i.into();
            if mentry.end > end {
                end = mentry.end;
            }
            pages += (mentry.end - mentry.start) / Self::BLOCK_SIZE;
        }
        let scratch_bits = align(end / 4096, 8) / 8;
        self.pages.store(pages, Ordering::SeqCst);

        let scratch_entry = mmap.iter().find(|i| i.phys_start >= 4096).unwrap();

        let scratch_start = scratch_entry.phys_start;

        let scratch_end = align(scratch_start as usize + scratch_bits, Self::BLOCK_SIZE) - 1;
        {
            let mut sscratch = self.scratch.lock();
            sscratch.init(scratch_start as *mut u8, scratch_bits);

            for i in mmap.iter().map(MemoryEntry::from) {
                if i.start < 4096 {
                    sscratch.set(0, true);
                }
                if i.start == scratch_start.try_into().unwrap() {
                    for j in (i.start..i.end).step_by(4096) {
                        if j >= scratch_end {
                            break;
                        }
                        let bit = j / 4096;
                        sscratch.set(bit, true);
                    }
                } else if i.kind == MemoryKind::Reserve || i.kind == MemoryKind::ACPINonVolatile {
                    for j in (i.start..i.end).step_by(4096) {
                        let bit = j / 4096;
                        sscratch.set(bit, true);
                    }
                }
            }
        }

        println!(
            "{}/{} usable",
            self.pages.load(Ordering::SeqCst) - self.get_used(),
            self.pages.load(Ordering::SeqCst),
        );

        Ok(())
    }

    /// Allocate physical memory, returning a pointer to the allocated memory and the block that the allocation started on
    ///
    /// # Arguments
    /// * `size` - Size of memory desired in kilobytes
    ///
    /// # Example
    fn alloc(&self, size: usize) -> Result<(*mut u8, usize), AllocatorError> {
        assert!(size < (self.pages.load(Ordering::SeqCst) * Self::BLOCK_SIZE));

        let pages = align(size * 1024, Self::BLOCK_SIZE) / Self::BLOCK_SIZE;

        {
            let tmp = self.scratch.lock();
            assert!(tmp[0]);
        }

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

        self.set_range(pages, found, true);

        Ok((unsafe { self.region.add(found << size) as *mut u8 }, found))
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
            let scratch = self.scratch.lock();
            if scratch[block_start] {
                return Err(AllocatorError::DoubleFree);
            }
        }

        self.set_range(block_count, block_start, false);

        Ok(())
    }
}

unsafe impl<'a> Sync for PageAllocator<'a> {}
