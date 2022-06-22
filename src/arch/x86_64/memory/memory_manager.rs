use limine_protocol::structures::memory_map_entry::MemoryMapEntry;
use log::{error, trace};

use crate::{
    arch::memory::addresses::AddressWithFlags,
    memory::{
        addresses::{Address, Physical, Virtual},
        errors::AddressError,
    },
};

use crate::memory::addresses::AlignedAddress;
use crate::traits::{VirtualMemoryManager, VirtualMemoryManagerError};

use super::tables::TableLevel4;

/// I'm not gonna have this hold data rn, might later for reasons.
pub struct MemoryManager {}

impl MemoryManager {
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

impl VirtualMemoryManager for MemoryManager {
    type VMMResult<T> = Result<T, VirtualMemoryManagerError>;

    /// Initialize the virtual memory manager
    unsafe fn init(&self, _mmap: &[&MemoryMapEntry]) -> Self::VMMResult<()> {
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

        if p2_raw.get_flags().contains(AddressWithFlags::HUGE_PAGE)
            && p2_raw.get_flags().contains(AddressWithFlags::PRESENT)
        {
            trace!("Level 2 Base: {:?}", p2_raw.get_address());
            trace!("Level 2 Huge Offset: {:#X}", src.level_1_huge_offset());

            return match Address::<Physical>::new(p2_raw.get_address() + src.level_2_huge_offset())
            {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Failed to create physical address during address translation: {e:?}");
                    None
                }
            };
        }

        let p2 = p3.sub_table(src.p3_index())?;

        trace!("Got P2");

        let p1_raw = p2.data[src.p2_index()].clone();

        if p1_raw.get_flags().contains(AddressWithFlags::HUGE_PAGE)
            && p1_raw.get_flags().contains(AddressWithFlags::PRESENT)
        {
            trace!("Level 1 Base: {:?}", p1_raw.get_address());
            trace!("Level 1 Huge Offset: {:#X}", src.level_1_huge_offset());
            return match Address::<Physical>::new(p1_raw.get_address() + src.level_1_huge_offset())
            {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Failed to create physical address during address translation: {e:?}");
                    None
                }
            };
        }

        let p1 = p2.sub_table(src.p2_index())?;

        trace!("Got P1");

        let frame = p1.frame(src.p1_index())?;

        match Address::<Physical>::new(frame.get_address() + src.frame_offset()) {
            Ok(v) => Some(v),
            Err(e) => {
                error!("Failed to create physical address during address translation: {e:?}");
                None
            }
        }
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
        let src: Address<Physical> =
            Address::<Physical>::new(src.get_address() | flags).map_err(|e| match e {
                AddressError::AddressNonCanonical => VirtualMemoryManagerError::AddressNotCanonical,
                _ => panic!("Unexpected error"),
            })?;
        let mut src: AlignedAddress<Physical> = src.try_into().map_err(|e| match e {
            AddressError::AddressNotAligned => VirtualMemoryManagerError::UnalignedAddress,
            _ => panic!("Unexpected error"),
        })?;
        {
            let flags = src.get_raw_address_mut().get_flags_mut();
            flags.insert(AddressWithFlags::PRESENT | AddressWithFlags::WRITABLE);
        }

        let p4 = unsafe { Self::get_p4_table() };

        let p3 = p4.sub_table_create(dst.p4_index());

        if p3.data[dst.p3_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
            return Err(VirtualMemoryManagerError::AttemptedToMapToHugePage);
        }

        let p2 = p3.sub_table_create(dst.p3_index());

        if p2.data[dst.p2_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
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

        if p3.data[src.p3_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
            p3.data[src.p3_index()].0 = unsafe { AddressWithFlags::from_bits_unchecked(0) };
        }

        let p2 = p3
            .sub_table(src.p3_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        if p2.data[src.p2_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
            p2.data[src.p2_index()].0 = unsafe { AddressWithFlags::from_bits_unchecked(0) };
        }

        let p1 = p2
            .sub_table(src.p2_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        p1.data[src.p1_index()].0 = unsafe { AddressWithFlags::from_bits_unchecked(0) };

        Ok(())
    }
}
