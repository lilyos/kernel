use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use limine_protocol::structures::memory_map_entry::{EntryType, MemoryMapEntry};
use log::{debug, info};

use crate::{
    arch::PlatformType,
    collections::BitSlice,
    errors::GenericError,
    memory::{errors::AllocatorError, utilities::align},
    sync::RwLock,
    traits::Init,
};

type RawAddress = <PlatformType as crate::traits::Platform>::RawAddress;
type UnderlyingType = <RawAddress as crate::traits::RawAddress>::UnderlyingType;

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

    const fn address_for_block(&self, block_index: usize) -> *const u8 {
        unsafe { self.region.add(block_index * Self::BLOCK_SIZE) }
    }

    const fn address_fits_alignment(address: usize, alignment: usize) -> bool {
        address % alignment == 0
    }

    const fn page_count_for_layout(layout: Layout) -> usize {
        align(layout.size(), Self::BLOCK_SIZE) / Self::BLOCK_SIZE
    }

    fn get_zone_for_layout(&self, layout: Layout) -> Option<usize> {
        let page_count = Self::page_count_for_layout(layout);

        let mut block = 0;
        let mut consecutive = 0;
        {
            let scratch = self.scratch.read();
            let iter = scratch.iter();
            for (index, item) in iter.enumerate() {
                if consecutive == page_count
                    && Self::address_fits_alignment(
                        self.address_for_block(block) as usize,
                        layout.align(),
                    )
                {
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

impl<'a> Init for PageAllocator<'a> {
    type Error = AllocatorError;

    type Input = &'a [&'a MemoryMapEntry];

    fn init(&self, mmap: Self::Input) -> Result<(), Self::Error> {
        assert!(!mmap.is_empty());
        let mut pages: usize = 0;
        let mut end: usize = 0;

        for mentry in mmap.iter() {
            let mmen_end: usize = mentry.end().try_into().unwrap();
            if mmen_end > end {
                end = mmen_end as usize;
            }
            pages +=
                (mmen_end - TryInto::<usize>::try_into(mentry.base).unwrap()) / Self::BLOCK_SIZE;
        }
        let scratch_bytes = align(end / 4096, 8) / 8;
        self.pages.store(pages, Ordering::SeqCst);

        let scratch_entry = mmap.iter().find(|i| i.base >= 4096).unwrap();

        let scratch_start: usize = scratch_entry.base.try_into().unwrap();

        let scratch_end = align(
            (scratch_start + scratch_bytes)
                .try_into()
                .map_err(|_| AllocatorError::Generic(GenericError::IntConversionError))?,
            Self::BLOCK_SIZE,
        ) - 1;

        {
            let mut sscratch = self.scratch.write();
            unsafe {
                sscratch.init(
                    scratch_start as *mut u8,
                    scratch_bytes
                        .try_into()
                        .map_err(|_| AllocatorError::Generic(GenericError::IntConversionError))?,
                )
            };
            sscratch.set(0, true);
            for i in mmap.iter() {
                for a in (i.base..i.end()).step_by(4096) {
                    if a < 4096
                        || (a
                            >= scratch_start.try_into().map_err(|_| {
                                AllocatorError::Generic(GenericError::IntConversionError)
                            })?
                            && a < scratch_end.try_into().map_err(|_| {
                                AllocatorError::Generic(GenericError::IntConversionError)
                            })?)
                        || i.kind == EntryType::Reserved
                        || i.kind == EntryType::AcpiNonVolatile
                        || i.kind == EntryType::BadMemory
                        || i.kind == EntryType::Framebuffer
                        || i.kind == EntryType::KernelAndModules
                    {
                        sscratch.set(
                            (a / 4096).try_into().map_err(|_| {
                                AllocatorError::Generic(GenericError::IntConversionError)
                            })?,
                            true,
                        )
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
}

unsafe impl<'a> Allocator for PageAllocator<'a> {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        if layout.size() >= self.pages.load(Ordering::SeqCst) * Self::BLOCK_SIZE {
            return Err(AllocError);
        }

        let pages = Self::page_count_for_layout(layout);
        let block = self.get_zone_for_layout(layout).ok_or(AllocError)?;

        let ptr = NonNull::from_raw_parts(
            NonNull::new(self.address_for_block(block) as *mut ()).ok_or(AllocError)?,
            layout.size(),
        );

        self.set_range(pages, block, true);

        Ok(ptr)
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        let pages = Self::page_count_for_layout(layout);

        self.set_range(pages, ptr.as_ptr() as usize / 4096, false);
    }
}

unsafe impl<'a> Sync for PageAllocator<'a> {}
