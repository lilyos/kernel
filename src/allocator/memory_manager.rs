use super::paging::{VirtAddr, PhysAddr};

pub struct MemoryManager {}

impl MemoryManager {
    pub unsafe fn init() -> Self {
        Self {}
    }

    pub fn virt_to_phys(&self, addr: VirtAddr) -> Option<usize> {
        // TODO: Setup Virtual to Physical translation
        None
    }

    /// Source and destination must be page-aligned
    pub fn map(&self, src: PhysAddr, dest: VirtAddr, flags: u32) -> Result<(), ()> {
        // TODO: Setup virtual mapping
        let pdindex = dest.0 >> 22;
        let ptindex = dest.0 >> 12 & 0x03FF;

        let pd = unsafe { *0xFFFFF000 };
        let pt = unsafe { *0xFFC00000 }.offset(0x400 * pdindex);

        unsafe { core::ptr::write_volatile(pt.offset(ptindex)), (src.0 | (flags & 0xFFF) | 0x01)) }
        Ok(())
    }

    /// This will fail if the address isn't mapped to any page
    pub fn unmap(&self, addr: VirtAddr) -> Result<(), ()> {
        // TODO: Setup virtual unmapping
        let pdindex = dest.0 >> 22;
        let ptindex = dest.0 >> 12 & 0x03FF;

        let pd = unsafe { *0xFFFFF000 };
        let pt = unsafe { *0xFFC00000 }.offset(0x400 * pdindex);

        unsafe { core::ptr::write_volatile(pt.offset(ptindex)), 0) }
        Ok(())
    }
}