use crate::{collections::BitSlice, memory::allocators::AllocatorError};

pub struct BuddyAllocator<'a> {
    blocks: usize,
    region: *mut u8,
    scratch: BitSlice<'a>,
}

impl<'a> core::fmt::Display for BuddyAllocator<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "BuddyAllocator {{\n\tblocks: {},\n\tregion: {:?},\n\tscratch: {{ .. }},\n}}",
            self.blocks, self.region
        )
    }
}

impl<'a> BuddyAllocator<'a> {
    /// Return a new buddy allocator
    ///
    /// # Arguments
    /// * `blocks` - The number of blocks in the buddy allocator, must be a power of 2. Probably.
    pub const fn new() -> Self {
        Self {
            blocks: 0,
            region: core::ptr::null_mut(),
            scratch: BitSlice::new(),
        }
    }

    pub unsafe fn init(&mut self, region: *mut u8, scratch: *mut u8, blocks: usize) {
        /*
        assert!(region_size % 2 == 0);
        assert!(scratch_size >= region_size / 8);
        */
        self.region = region;
        self.blocks = blocks;
        self.scratch.init(scratch, (self.blocks * 1024) / 8);
    }

    /// Allocate physical memory, returning a pointer to the allocated memory and the block that the allocation started on
    ///
    /// # Arguments
    /// * `size` - Size of memory desired in kilobytes
    ///
    /// # Example
    pub fn alloc(&mut self, size: usize) -> Result<(*mut u8, usize), AllocatorError> {
        assert!(size < 256);

        let found = match self.get_zone_with_size(size) {
            Some(v) => v,
            None => {
                if self.get_used() == 256 {
                    return Err(AllocatorError::OutOfMemory);
                } else {
                    return Err(AllocatorError::NoLargeEnoughRegion);
                }
            }
        };

        self.set_range(size, found, true);

        Ok((unsafe { self.region.add(found << size) }, found))
    }

    /// Deallocate physical memory, freeing it
    ///
    /// # Arguments
    /// * `block_count` - How many blocks/kilobytes were allocated
    /// * `block_start` - The block the allocation started on
    pub fn dealloc(
        &mut self,
        block_start: usize,
        block_count: usize,
    ) -> Result<(), AllocatorError> {
        assert!(block_start < self.blocks);

        if self.scratch[block_start] {
            return Err(AllocatorError::DoubleFree);
        }

        self.set_range(block_count, block_start, false);

        Ok(())
    }

    fn get_used(&mut self) -> usize {
        let mut total = 0;
        for item in &mut self.scratch {
            if item {
                total += 1;
            }
        }
        self.scratch.reset_iterator();
        total
    }

    fn get_zone_with_size(&mut self, block_count: usize) -> Option<usize> {
        let mut block = 0;
        let mut consecutive = 0;
        for (index, item) in (&mut self.scratch).enumerate() {
            if consecutive == block_count {
                return Some(block);
            } else if item {
                consecutive += 1;
            } else {
                block = index;
                consecutive = 0;
            }
        }
        self.scratch.reset_iterator();
        None
    }

    fn set_range(&mut self, blocks_to_set: usize, starting_pos: usize, value: bool) {
        assert!(blocks_to_set < self.blocks);
        assert!(starting_pos < (self.blocks * 1024) / 8);

        for i in 0..blocks_to_set {
            for x in
                (starting_pos << (blocks_to_set - i))..((starting_pos + 1) << (blocks_to_set - i))
            {
                self.scratch.set(x, value);
            }

            if value {
                for i in blocks_to_set..self.blocks {
                    if self.scratch[starting_pos >> (i - blocks_to_set)] {
                        break;
                    }
                    self.scratch.set(starting_pos >> (i - blocks_to_set), true);
                }
            } else {
                for i in blocks_to_set..self.blocks {
                    self.scratch.set(starting_pos >> (i - blocks_to_set), false);
                    if self.scratch[(starting_pos >> (i - blocks_to_set)) ^ 1] {
                        break;
                    }
                }
            }
        }
    }

    pub fn is_address_in_region(&self, addr: *mut u8) -> bool {
        let addr = addr as usize;
        let bottom = self.region as usize;
        let top = unsafe { self.region.add(self.blocks * 1024) } as usize;
        bottom <= addr && addr <= top
    }
}
