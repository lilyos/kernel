use log::{error, trace};

use crate::{
    errors::MemoryManagerError,
    memory::addresses::{Address, Physical, Virtual},
    traits::{Init, MemoryFlags},
};

use crate::{memory::addresses::AlignedAddress, traits::MemoryManager as MemoryManagerTrait};

use super::{addresses::AddressWithFlags, tables::TableLevel4};

/// I'm not gonna have this hold data rn, might later for reasons.
pub struct MemoryManager {}

impl MemoryManager {
    /// Create a new virtual memory manager
    pub const fn new() -> Self {
        Self {}
    }
}

unsafe impl MemoryManagerTrait for MemoryManager {
    type RootTable = TableLevel4;

    unsafe fn current_table(&self, tr: &mut TableLevel4) -> Result<(), MemoryManagerError> {
        asm!("mov cr3, {}", in(reg) (tr) as *mut TableLevel4);
        Ok(())
    }

    unsafe fn get_current_table(&self) -> Result<&'static mut TableLevel4, MemoryManagerError> {
        let cr3: u64;

        asm!("mov {}, cr3", out(reg) cr3);
        Ok(&mut *(cr3 as *mut TableLevel4))
    }

    /// Map the specified frame to the destination, with the option to provide additional flags
    /// These flags will be applied to all **created** frames, it will not affect already present entries
    unsafe fn map(
        &self,
        rtable: &mut Self::RootTable,
        src: AlignedAddress<Physical>,
        dst: AlignedAddress<Virtual>,
        flags: MemoryFlags,
    ) -> Result<(), MemoryManagerError> {
        let dst = dst.inner();

        let p3 = rtable.sub_table_create(dst.p4_index(), flags)?;

        if p3.data[dst.p3_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
            return Err(MemoryManagerError::CannotMapToHugePage);
        }

        let p2 = p3.sub_table_create(dst.p3_index(), flags)?;

        if p2.data[dst.p2_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
            return Err(MemoryManagerError::CannotMapToHugePage);
        }

        let p1 = p2.sub_table_create(dst.p2_index(), flags)?;

        let _frame = p1.frame_set_specified(dst.p1_index(), src, flags);

        Ok(())
    }

    /// Unmap the specified virtual address
    unsafe fn unmap(
        &self,
        rtable: &mut Self::RootTable,
        addr: AlignedAddress<Virtual>,
    ) -> Result<(), MemoryManagerError> {
        let addr = addr.inner();

        let p3 = rtable
            .sub_table_mut(addr.p4_index())
            .ok_or(MemoryManagerError::AddressUnmapped)?;

        if p3.data[addr.p3_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
            p3.data[addr.p3_index()].0 = AddressWithFlags::none();
        }

        let p2 = p3
            .sub_table_mut(addr.p3_index())
            .ok_or(MemoryManagerError::AddressUnmapped)?;

        if p2.data[addr.p2_index()]
            .get_flags()
            .contains(AddressWithFlags::HUGE_PAGE)
        {
            p2.data[addr.p2_index()].0 = AddressWithFlags::none();
        }

        let p1 = p2
            .sub_table_mut(addr.p2_index())
            .ok_or(MemoryManagerError::AddressUnmapped)?;

        p1.data[addr.p1_index()].0 = AddressWithFlags::none();

        Ok(())
    }

    /// Convert a given virtual address to its physical counterpart
    fn virtual_to_physical(
        &self,
        rtable: &Self::RootTable,
        addr: Address<Virtual>,
    ) -> Option<Address<Physical>> {
        let addr = addr.inner();

        let p3 = rtable.sub_table(addr.p4_index())?;

        let p2_raw = p3.data[addr.p3_index()].clone();

        if p2_raw.get_flags().contains(AddressWithFlags::HUGE_PAGE)
            && p2_raw.get_flags().contains(AddressWithFlags::PRESENT)
        {
            trace!("Level 2 Base: {:?}", p2_raw.get_address());
            trace!("Level 2 Huge Offset: {:#X}", addr.level_1_huge_offset());

            return match Address::<Physical>::new(p2_raw.get_address() + addr.level_2_huge_offset())
            {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Failed to create physical address during address translation: {e:?}");
                    None
                }
            };
        }

        let p2 = p3.sub_table(addr.p3_index())?;

        trace!("Got P2");

        let p1_raw = p2.data[addr.p2_index()].clone();

        if p1_raw.get_flags().contains(AddressWithFlags::HUGE_PAGE)
            && p1_raw.get_flags().contains(AddressWithFlags::PRESENT)
        {
            trace!("Level 1 Base: {:?}", p1_raw.get_address());
            trace!("Level 1 Huge Offset: {:#X}", addr.level_1_huge_offset());
            return match Address::<Physical>::new(p1_raw.get_address() + addr.level_1_huge_offset())
            {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Failed to create physical address during address translation: {e:?}");
                    None
                }
            };
        }

        let p1 = p2.sub_table(addr.p2_index())?;

        trace!("Got P1");

        let frame = p1.frame(addr.p1_index())?;

        match Address::<Physical>::new(Into::<usize>::into(*frame) + addr.frame_offset()) {
            Ok(v) => Some(v),
            Err(e) => {
                error!("Failed to create physical address during address translation: {e:?}");
                None
            }
        }
    }
}

impl Init for MemoryManager {
    type Error = core::convert::Infallible;

    type Input = ();
}
