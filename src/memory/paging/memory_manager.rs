use stivale2::boot::tags::structures::MemoryMapStructure;

use crate::memory::paging::{
    addresses::{Address, Physical, Virtual},
    tables::TableLevel4,
};

use super::{
    addresses::AlignedAddress,
    traits::{VirtualMemoryManager, VirtualMemoryManagerError},
};

/// I'm not gonna have this hold data rn, might later for reasons.
pub struct MemoryManagerImpl {}

impl MemoryManagerImpl {
    /// Create a new virtual memory manager
    pub const fn new() -> Self {
        Self {}
    }

    /// Get the level 4 paging table
    unsafe fn get_p4_table() -> &'static mut TableLevel4 {
        let cr3: u64;

        asm!("mov {}, cr3", out(reg) cr3);

        &mut *(cr3 as *mut TableLevel4)
    }
}

impl VirtualMemoryManager for MemoryManagerImpl {
    type VMMResult<T> = Result<T, VirtualMemoryManagerError>;

    /// Initialize the virtual memory manager
    unsafe fn init(&self, _mmap: &MemoryMapStructure) -> Self::VMMResult<()> {
        /*
        let max = mmap.memmap.iter().last().unwrap();
        let begin_at = u64::MAX - max.end() as u64;
        println!("Beginning at 0x{:x}", begin_at);
        for i in (4096..max.end()).step_by(4096) {
            let frame = Address::<Physical>::new(i as usize)
                .try_into()
                .map_err(|_| VirtualMemoryManagerError::UnalignedAddress)?;
            let page = Address::<Virtual>::new((begin_at + i as u64) as *mut u8)
                .map_err(|_| VirtualMemoryManagerError::AddressNotCanonical)?
                .try_into()
                .map_err(|_| VirtualMemoryManagerError::UnalignedAddress)?;
            self.map(frame, page, 0)?;
        }
        for entry in mmap.memmap.iter() {
            for i in entry.base..entry.length {
                let frame =
            }
        }
        */
        Ok(())
    }

    /// Convert a given virtual address to its physical counterpart
    ///
    /// # Example
    /// ```
    /// let x = 9u64;
    /// let x_ptr = &x as *const u64;
    ///
    /// let addr = MEMORY_MANAGER.virtual_to_physical(x_ptr).unwrap();
    /// ```
    fn virtual_to_physical(&self, src: Address<Virtual>) -> Option<Address<Physical>> {
        let p4 = unsafe { Self::get_p4_table() };

        let p3 = p4.sub_table(src.p4_index())?;

        let p2_raw = p3.data[src.p3_index()].clone();

        if p2_raw.get_huge_page() && p2_raw.get_present() {
            return unsafe {
                Some(Address::<Physical>::new(
                    p2_raw.address().add(src.level_2_huge_offset()) as usize,
                ))
            };
        }

        let p2 = p3.sub_table(src.p3_index())?;

        println!("Got P2");

        let p1_raw = p2.data[src.p2_index()].clone();

        if p1_raw.get_present() && p1_raw.get_huge_page() {
            println!("Level 1 Base: {:?}", p1_raw.address());
            println!("Level 1 Huge Offset: 0x{:x}", src.level_1_huge_offset());
            return unsafe {
                Some(Address::<Physical>::new(
                    p1_raw.address().add(src.level_1_huge_offset()) as usize,
                ))
            };
        }

        let p1 = p2.sub_table(src.p2_index())?;

        println!("Got P1");

        let frame = p1.frame(src.p1_index())?;

        Some(Address::<Physical>::new(
            frame.get_address() + src.frame_offset(),
        ))
    }

    /// Map the specified frame to the destination, with the option to provide additional flags
    ///
    /// # Example
    /// ```
    /// let frame = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
    /// let page = Page::new(0xdeadc000).unwrap();
    ///
    /// let _ = MEMORY_MANAGER.map(frame, page, 0).unwrap();
    fn map(
        &self,
        src: AlignedAddress<Physical>,
        dst: AlignedAddress<Virtual>,
        flags: usize,
    ) -> Self::VMMResult<()> {
        let mut src = Address::<Physical>::new(src.get_inner() | flags).align_lossy();
        src.set_present();
        src.set_writable();
        let p4 = unsafe { Self::get_p4_table() };
        let p3 = p4.sub_table_create(dst.p4_index());
        if p3.data[dst.p3_index()].clone().get_huge_page() {
            return Err(VirtualMemoryManagerError::AttemptedToMapToHugePage);
        }
        let p2 = p3.sub_table_create(dst.p3_index());
        if p2.data[dst.p2_index()].get_huge_page() {
            return Err(VirtualMemoryManagerError::AttemptedToMapToHugePage);
        }
        let p1 = p2.sub_table_create(dst.p2_index());
        let _frame = p1.frame_set_specified(dst.p1_index(), src);

        Ok(())
    }

    fn unmap(&self, src: AlignedAddress<Virtual>) -> Self::VMMResult<()> {
        let p4 = unsafe { Self::get_p4_table() };
        let p3 = p4
            .sub_table(src.p4_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        if p3.data[src.p3_index()].get_huge_page() {
            p3.data[src.p3_index()].0 = 0;
        }

        let p2 = p3
            .sub_table(src.p3_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        if p2.data[src.p2_index()].get_huge_page() {
            p2.data[src.p2_index()].0 = 0;
        }

        let p1 = p2
            .sub_table(src.p2_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        p1.data[src.p1_index()].0 = 0;

        Ok(())
    }
}
