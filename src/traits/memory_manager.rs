use core::alloc::Layout;

use crate::{
    errors::{GenericError, MemoryManagerError},
    macros::bitflags::bitflags,
    memory::{
        addresses::{Address, AlignedAddress, Physical, Virtual},
        utilities::align,
    },
    traits::PlatformAddress,
    PHYSICAL_ALLOCATOR,
};

use super::PhysicalAllocator;

bitflags! {
    /// Memory flags that can be used when mapping something
    pub struct MemoryFlags: u64 {
        const KERNEL_ONLY = 1 << 0;
        const READABLE = 1 << 1;
        const WRITABLE = 1 << 2;
        const EXECUTABLE = 1 << 3;
        const CACHABLE = 1 << 4;
    }
}

/// Trait for a [Platform](crate::traits::Platform)'s Memory Manager
///
/// # Safety
/// This must not overwrite random memory and must ensure
/// mappings and unmappings occur unless they return an error
pub unsafe trait MemoryManager {
    /// The Root Table type for the Platform
    type RootTable;

    /// Set the current Root Table
    ///
    /// # Safety
    /// The caller must guarantee the root table will not be freed for the duration it is used.
    /// Addtionally, the table must be identity mapped
    ///
    /// # Errors
    /// This will return an error if the table is unable to be set
    unsafe fn current_table(&self, tr: &mut Self::RootTable) -> Result<(), MemoryManagerError>;

    /// Get the current Root Table
    ///
    /// # Safety
    /// The returned reference must **not** be aliased, as that would violate exclusive access rules
    /// Addtionally, the table must be identity mapped
    ///
    /// # Errors
    /// This will return an error if the current table is not yet set
    unsafe fn get_current_table(&self) -> Result<&mut Self::RootTable, MemoryManagerError>;

    /// Map a given physical address to a specified virtual address
    ///
    /// # Safety
    /// The specified root table must be mapped in memory
    /// This memory must not be in use by the kernel, otherwise undefined behavior may occur
    ///
    /// # Errors
    /// This will return an error if the page or any intermediaries are unable to be mapped
    unsafe fn map(
        &self,
        rtable: &mut Self::RootTable,
        src: AlignedAddress<Physical>,
        dst: AlignedAddress<Virtual>,
        flags: MemoryFlags,
    ) -> Result<(), MemoryManagerError>;

    /// Unmap a given virtual address
    ///
    /// # Safety
    /// The specified root table must be mapped in memory
    /// This memory must not be in use by the kernel, otherwise undefined behavior may occur
    ///
    /// # Errors
    /// This will return an error if the address is unable to be unmapped
    unsafe fn unmap(
        &self,
        rtable: &mut Self::RootTable,
        addr: AlignedAddress<Virtual>,
    ) -> Result<(), MemoryManagerError>;

    /// Try to find the physical address for a given virtual address
    ///
    /// If the given root table is not mapped in memory, results are undefined
    fn virtual_to_physical(
        &self,
        rtable: &Self::RootTable,
        addr: Address<Virtual>,
    ) -> Option<Address<Physical>>;

    /// Find an area free for mapping `count` pages
    ///
    /// If the given root table is not mapped in memory, results are undefined
    fn find_free_mapping_area(
        &self,
        rtable: &Self::RootTable,
        allowed_range: impl Iterator<Item = usize>,
        count: usize,
        alignment: usize,
    ) -> Option<AlignedAddress<Virtual>> {
        let mut current = None;
        let mut consecutive = 0;
        for addr in allowed_range
            .filter(|addr| addr % alignment == 0)
            .filter_map(|addr| AlignedAddress::<Virtual>::new(addr as *mut ()).ok())
        {
            if count == consecutive {
                return current;
            }
            if self.virtual_to_physical(rtable, addr.into()).is_none() {
                current = Some(addr);
                consecutive += 1;
            } else {
                current = None;
                consecutive = 0;
            }
        }
        None
    }

    // TODO: Decide on how huge pages should be used, if at all
    /// Allocate a given [Layout] and map it in a free region
    ///
    /// # Safety
    /// The specified root table must be mapped in memory
    ///
    /// # Errors
    /// This will return an error if allocation fails or mapping fails
    unsafe fn allocate_and_map(
        &self,
        rtable: &'static mut Self::RootTable,
        allowed_range: impl Iterator<Item = usize>,
        flags: MemoryFlags,
        layout: Layout,
    ) -> Result<AlignedAddress<Virtual>, MemoryManagerError> {
        let p_addr = PHYSICAL_ALLOCATOR
            .allocate(layout)
            .map_err(MemoryManagerError::Allocator)?;

        let pages = align(layout.size(), 4096);

        let free_area = self
            .find_free_mapping_area(rtable, allowed_range, pages, layout.align())
            .ok_or(MemoryManagerError::VirtualMemoryExhausted)?;

        for (idx, addr) in (TryInto::<usize>::try_into(free_area.inner().into_raw())
            .map_err(|_| MemoryManagerError::Generic(GenericError::IntConversionError))?
            ..(TryInto::<usize>::try_into(free_area.inner().into_raw())
                .map_err(|_| MemoryManagerError::Generic(GenericError::IntConversionError))?
                + (pages * 4096)))
            .step_by(4096)
            .filter_map(|addr| AlignedAddress::<Virtual>::new(addr as *const ()).ok())
            .enumerate()
        {
            self.map(
                rtable,
                AlignedAddress::<Physical>::new(
                    TryInto::<usize>::try_into(p_addr.inner().into_raw()).map_err(|_| {
                        MemoryManagerError::Generic(GenericError::IntConversionError)
                    })? + (idx * 4096),
                )
                .map_err(MemoryManagerError::Address)?,
                addr,
                flags,
            )?;
        }

        Ok(free_area)
    }

    /// Unmap the area with a given (Layout)[Layout]
    ///
    /// # Safety
    /// The specified root table must be mapped in memory
    /// The memory must no longer be in use
    /// # Errors
    /// This will return an error if deallocation fails or unmapping fails
    unsafe fn deallocate_and_unmap(
        &self,
        rtable: &'static mut Self::RootTable,
        addr: AlignedAddress<Virtual>,
        layout: Layout,
    ) -> Result<(), MemoryManagerError> {
        let pages = align(layout.size(), 4096);

        let phys_addr = self
            .virtual_to_physical(&*rtable, addr.into())
            .ok_or(MemoryManagerError::AddressUnmapped)?
            .try_into()
            .map_err(MemoryManagerError::Address)?;
        PHYSICAL_ALLOCATOR.deallocate(phys_addr, layout);

        #[allow(clippy::cast_possible_truncation)]
        for addr in (addr.inner().into_raw() as usize
            ..(addr.inner().into_raw() as usize + (pages * 4096)))
            .step_by(4096)
            .filter_map(|addr| AlignedAddress::<Virtual>::new(addr as *const ()).ok())
        {
            self.unmap(&mut *rtable, addr)?;
        }

        Ok(())
    }
}
