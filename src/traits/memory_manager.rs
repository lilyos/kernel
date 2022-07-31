use crate::{
    errors::MemoryManagerError,
    macros::bitflags::bitflags,
    memory::addresses::{Address, AlignedAddress, Physical, Virtual},
};

bitflags! {
    pub struct MemoryFlags: u64 {
        const KERNEL_ONLY = 1 << 0;
        const READABLE = 1 << 1;
        const WRITABLE = 1 << 2;
        const EXECUTABLE = 1 << 3;
        const CACHABLE = 1 << 4;
    }
}

/// Trait for a [Platform](crate::traits::Platform)'s Memory Manager
pub unsafe trait MemoryManager {
    type Error = MemoryManagerError;

    /// The Root Table type for the Platform
    type RootTable;

    /// Set the current Root Table
    ///
    /// # Safety
    /// The caller must guarantee the root table will not be freed for the duration it is used
    unsafe fn current_table(&self, tr: &mut Self::RootTable) -> Result<(), Self::Error>;

    /// Get the current Root Table
    ///
    /// # Safety
    /// The returned reference must **not** be aliased, as that would violate exclusive access rules
    unsafe fn get_current_table(&self) -> Result<&mut Self::RootTable, Self::Error>;

    unsafe fn map(
        &self,
        rtable: &'static mut Self::RootTable,
        src: AlignedAddress<Physical>,
        dst: AlignedAddress<Virtual>,
        flags: MemoryFlags,
    ) -> Result<(), Self::Error>;

    unsafe fn unmap(
        &self,
        rtable: &'static mut Self::RootTable,
        addr: AlignedAddress<Virtual>,
    ) -> Result<(), Self::Error>;

    fn virtual_to_physical(
        &self,
        rtable: &'static mut Self::RootTable,
        addr: Address<Virtual>,
    ) -> Option<Address<Physical>>;

    /*
    fn map_in_current_table(
        &self,
        src: AlignedAddress<Physical>,
        dst: AlignedAddress<Virtual>,
    ) -> Result<(), Self::Error> {
        let rtable = self.get_current_root_table()?;
        self.map(rtable, src, dst)
    }

    fn unmap_in_current_table(&self, addr: AlignedAddress<Virtual>) -> Result<(), Self::Error> {
        let rtable = self.get_current_root_table()?;
        self.unmap(rtable, addr)
    }

    fn virtual_to_physical_in_current_table(
        &self,
        addr: Address<Virtual>,
    ) -> Option<Address<Physical>> {
        let rtable = self.get_current_root_table().map_or(None, |f| Some(f))?;
        self.virtual_to_physical(rtable, addr)
    }
    */
}
