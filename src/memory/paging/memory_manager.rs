use core::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use super::{Frame, Page, VirtualAddress, VirtualMemoryManager};

pub struct MemoryManagerImpl {}

pub struct Table {
    data: [Page; 512],
}

impl Index<usize> for Table {
    type Output = Page;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for i in self.data.iter() {
            if !i.address().is_null() {
                write!(f, "{:?}", i)?;
            }
        }
        Ok(())
    }
}

impl Table {
    pub fn new() -> Self {
        Self {
            data: [Page::with_address(0 as *mut u8); 512],
        }
    }
}

impl MemoryManagerImpl {
    pub const fn new() -> Self {
        Self {}
    }
}

impl VirtualMemoryManager for MemoryManagerImpl {
    type VMMResult<T> = Result<T, super::VirtualMemoryManagerError>;

    unsafe fn init(
        &self,
        mmap: &[crate::memory::allocators::MemoryDescriptor],
    ) -> Self::VMMResult<()> {
        todo!()
    }

    fn virtual_to_physical(&self, src: VirtualAddress) -> Option<super::PhysicalAddress> {
        todo!()
    }

    fn map(&self, src: Frame, dst: Page, flags: u64) -> Self::VMMResult<()> {
        todo!()
    }

    fn unmap(&self, src: Page) -> Self::VMMResult<()> {
        todo!()
    }
}
