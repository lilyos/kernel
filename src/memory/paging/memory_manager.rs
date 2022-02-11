use core::{
    arch::asm,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::PHYSICAL_ALLOCATOR;

use super::{
    Flags, Frame, Page, PhysicalAddress, VirtualAddress, VirtualMemoryManager,
    VirtualMemoryManagerError,
};

pub struct MemoryManagerImpl {}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct PageTableEntry<L> {
    pub data: Flags,
    level: PhantomData<L>,
}

impl<L> PageTableEntry<L> {
    pub fn get_item(&self) -> Option<&L> {
        if self.data.unused() || !self.data.get_present() {
            None
        } else {
            Some(unsafe { &*(self.data.get_address() as *const L) })
        }
    }

    pub fn get_item_mut(&mut self) -> Option<&mut L> {
        if self.data.unused() || !self.data.get_present() {
            None
        } else {
            Some(unsafe { &mut *(self.data.get_address() as *mut L) })
        }
    }
}

impl<L> Deref for PageTableEntry<L> {
    type Target = L;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.data.get_address() as *mut L) }
    }
}

impl<L> DerefMut for PageTableEntry<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.data.get_address() as *mut L) }
    }
}

#[derive(Debug)]
#[repr(align(4096), C)]
pub struct TableLevel4 {
    pub data: [PageTableEntry<TableLevel3>; 512],
}

impl TableLevel4 {
    pub fn sub_table(&mut self, index: usize) -> Option<&mut TableLevel3> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    pub fn sub_table_create(&mut self, index: usize) -> &mut TableLevel3 {
        let entry = &mut self.data[index];
        if entry.data.unused() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            entry.data = Flags::new(ptr as u64, Flags::PRESENT | Flags::WRITABLE);
        }
        entry.get_item_mut().unwrap()
    }
}

#[derive(Debug)]
#[repr(align(4096), C)]
pub struct TableLevel3 {
    pub data: [PageTableEntry<TableLevel2>; 512],
}

impl TableLevel3 {
    pub fn sub_table(&mut self, index: usize) -> Option<&mut TableLevel2> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    pub fn sub_table_create(&mut self, index: usize) -> &mut TableLevel2 {
        let entry = &mut self.data[index];
        if entry.data.unused() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            entry.data = Flags::new(ptr as u64, Flags::PRESENT | Flags::WRITABLE);
        }
        entry.get_item_mut().unwrap()
    }
}

#[derive(Debug)]
#[repr(align(4096), C)]
pub struct TableLevel2 {
    pub data: [PageTableEntry<TableLevel1>; 512],
}

impl TableLevel2 {
    pub fn sub_table(&mut self, index: usize) -> Option<&mut TableLevel1> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    pub fn sub_table_create(&mut self, index: usize) -> &mut TableLevel1 {
        let mut entry = &mut self.data[index];
        if entry.data.unused() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            entry.data = Flags::new(ptr as u64, Flags::PRESENT | Flags::WRITABLE);
        }
        entry.get_item_mut().unwrap()
    }
}

#[derive(Debug)]
#[repr(align(4096), C)]
pub struct TableLevel1 {
    pub data: [Frame; 512],
}
impl TableLevel1 {
    pub fn frame(&mut self, index: usize) -> Option<&mut Frame> {
        let entry = &mut self.data[index];
        if entry.flags().unused() {
            None
        } else {
            Some(entry)
        }
    }

    pub fn frame_create(&mut self, index: usize) -> &mut Frame {
        let entry = &mut self.data[index];
        if entry.flags().unused() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            let new: u64 = Flags::new(ptr as u64, Flags::PRESENT | Flags::WRITABLE).into();
            *entry = new.into();
        }
        entry
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
        Ok(())
    }

    fn virtual_to_physical(&self, src: VirtualAddress) -> Option<PhysicalAddress> {
        None
    }

    fn map(&self, src: Frame, dst: Page, flags: u64) -> Self::VMMResult<()> {
        let cr3: u64;
        unsafe {
            asm!("mov {}, cr3", out(reg) cr3);
        }
        let p4 = unsafe { &mut *(cr3 as *mut TableLevel4) };
        let p3 = p4.sub_table_create(dst.p4_index());
        let p2 = p3.sub_table_create(dst.p3_index());
        let p1 = p2.sub_table_create(dst.p2_index());
        let mut frame = p1.frame_create(dst.p1_index());
        frame.inner = Flags::new(src.address() as u64, Flags::PRESENT | Flags::WRITABLE);
        Ok(())
    }

    fn unmap(&self, src: Page) -> Self::VMMResult<()> {
        Err(VirtualMemoryManagerError::NotImplemented)
    }
}
