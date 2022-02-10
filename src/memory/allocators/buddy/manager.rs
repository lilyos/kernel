use super::BuddyAllocator;
use crate::{
    memory::allocators::{align, AllocatorError, MemoryDescriptor, MemoryEntry, MemoryKind},
    peripherals::uart::println,
    sync::Mutex,
};

pub struct BuddyManager<'a> {
    buddies: Mutex<&'a mut [*mut BuddyAllocator<'a>]>,
}

impl<'a> BuddyManager<'a> {
    /// Construct a new BuddyManager
    pub const fn new() -> Self {
        Self {
            buddies: Mutex::new(&mut []),
        }
    }

    /// Initializes the BuddyManager
    ///
    /// # Arguments
    /// * `mmap` - A slice pointing to MemoryDescriptors to be used for allocation
    ///
    /// # Example
    /// ```
    /// // Assume mmap is a slice to a set of valid MemoryDescriptor
    /// let manager = BuddyManager::new();
    /// unsafe { manager.init(mmap) }
    /// ```
    pub unsafe fn init(&self, mmap: &[MemoryDescriptor]) -> Result<(), AllocatorError> {
        assert!(!mmap.is_empty());

        let len = mmap.len() + 1;

        let to_scratch: MemoryEntry = mmap[0].into();
        let scratch = align(
            (to_scratch.start as *mut u8).add(len * core::mem::size_of::<*mut BuddyAllocator>())
                as usize,
            1024,
        );

        {
            let mut buddies = self.buddies.lock();
            *buddies =
                core::slice::from_raw_parts_mut(to_scratch.start as *mut *mut BuddyAllocator, len);
            buddies.fill(core::ptr::null_mut());
        }

        self.add_new_buddy(scratch as *mut u8, to_scratch.end as *mut u8)?;

        for item in mmap.iter().skip(1).map(MemoryEntry::from) {
            if item.kind == MemoryKind::Reclaim {
                match self.add_new_buddy(item.start as *mut u8, item.end as *mut u8) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                };
            }
        }

        Ok(())
    }

    /// Print out all the buddies that are being managed
    /// # Example
    /// ```
    /// // Assume mmap is a slice to a set of valid MemoryDescriptor
    /// let manager = BuddyManager::new();
    /// unsafe { manager.init(mmap) }
    ///
    /// manager.get_buddies();
    /// ```
    pub fn get_buddies(&self) {
        let buddies = self.buddies.lock();
        for item in buddies.iter().filter(|i| !i.is_null()) {
            println!("{}", unsafe { &**item });
        }
    }

    /// Push a new buddy to the internal list
    ///
    /// # Arguments
    /// * `value` - The value to push
    fn push_to_buddies(&self, value: *mut BuddyAllocator<'a>) -> Result<(), AllocatorError> {
        println!("Pushing {:?} to buddies", value);
        let mut buddies = self.buddies.lock();

        for (index, item) in buddies.iter().enumerate() {
            if item.is_null() {
                buddies[index] = value;
                return Ok(());
            }
        }

        Err(AllocatorError::InternalStorageFull)
    }

    /// Add a new buddy for the specified region and block size
    ///
    /// # Arguments
    /// * `region_start` - A pointer to the start of the region to manage
    /// * `region_end` - A pointer to the end of the region to manage
    fn add_new_buddy(
        &self,
        region_start: *mut u8,
        region_end: *mut u8,
    ) -> Result<(), AllocatorError> {
        let size = region_end as usize - region_start as usize;

        let blocks = size / 8;
        let region_offset = unsafe {
            align(
                (region_start).add(blocks + core::mem::size_of::<BuddyAllocator>()) as usize,
                1024,
            )
        };

        let mut buddy = BuddyAllocator::new();
        let ptr = region_start as *mut BuddyAllocator;
        unsafe {
            buddy.init(
                region_offset as *mut u8,
                region_start.add(core::mem::size_of::<BuddyAllocator>()),
                blocks,
            );

            ptr.write(buddy);
        }

        self.push_to_buddies(ptr)
    }

    /// Allocate memory, returning a pointer to the allocated memory and the block that the allocation started on
    ///
    /// # Arguments
    /// * `size` - The size to allocate in kilobytes
    ///
    /// # Example
    /// ```
    /// // Assume mmap is a slice to a set of valid MemoryDescriptor
    /// let manager = BuddyManager::new();
    /// unsafe { manager.init(mmap) }
    ///
    /// let (alloc, blocks) = manager.alloc(2).unwrap(); // Allocates two kilobytes
    /// // later...
    /// manager.dealloc(alloc, blocks).unwrap();
    /// ```
    pub fn alloc(&self, size: usize) -> Result<(*mut u8, usize), AllocatorError> {
        let buddies = self.buddies.lock();
        for buddy in buddies
            .iter()
            .filter(|i| !i.is_null())
            .map(|i| unsafe { &mut **i })
        {
            if let Ok(alloc) = buddy.alloc(size) {
                return Ok(alloc);
            }
        }
        Err(AllocatorError::OutOfMemory)
    }

    /// Deallocate memory, freeing it and zeroing it
    ///
    /// # Arguments
    /// * `addr` - The address for the allocation
    /// * `block_start` - What block the allocation started on
    /// * `block_count` - How many blocks/kilobytes were allocated
    pub fn dealloc(
        &self,
        addr: *mut u8,
        block_start: usize,
        block_count: usize,
    ) -> Result<(), AllocatorError> {
        let buddies = self.buddies.lock();
        for buddy in buddies
            .iter()
            .filter(|i| !i.is_null())
            .map(|i| unsafe { &mut **i })
        {
            if buddy.is_address_in_region(addr) {
                buddy.dealloc(block_start, block_count)?;
            }
        }
        Err(AllocatorError::InternalError(
            "The address wasn't inside the allocation space",
        ))
    }
}
