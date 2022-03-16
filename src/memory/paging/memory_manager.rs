use core::{arch::asm, fmt::Display, marker::PhantomData};

use kernel_macros::bit_field_accessors;
use stivale2::boot::tags::structures::MemoryMapStructure;

use crate::PHYSICAL_ALLOCATOR;

use super::{
    Frame, Page, PhysicalAddress, VirtualAddress, VirtualMemoryManager, VirtualMemoryManagerError,
};

/// I'm not gonna have this hold data rn, might later for reasons.
pub struct MemoryManagerImpl {}

#[repr(transparent)]
#[derive(Clone)]
/// An entry in a page table of type L
pub struct PageTableEntry<L>(pub u64, PhantomData<L>);

impl<L> PageTableEntry<L> {
    /// Address mask for Virtual Addresses
    pub const BIT_52_ADDRESS: u64 =
        0b0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_0000_0000_0000;

    bit_field_accessors! {
        present 0;
        writable 1;
        user_accessible 2;
        write_through_caching 3;
        disable_cache 4;
        accessed 5;
        dirty 6;
        huge_page 7;
        global 8;
        // 9-11 Free Use
        reserved 9;
        // 52-62 Free Use
        no_execute 63;
    }

    /// Create a new PageTable Entry
    ///
    /// # Example
    /// ```
    /// let address = 0x5000;
    /// let entry: PageTableEntry<Frame> = PageTableEntry::new(0x5000 & PageTableEntry::BIT_52_ADDRESS, 0);
    /// ```
    pub fn new(address: u64, extra_flags: u64) -> Self {
        // let mut tmp = Self((address & Self::BIT_52_ADDRESS) | extra_flags, PhantomData);
        let mut tmp = Self(address | extra_flags, PhantomData);
        tmp.set_present();
        tmp.set_writable();
        tmp
    }

    /// Returns if the entry is unused (equal to zero)
    pub fn unused(&self) -> bool {
        self.0 == 0
    }

    /// Get the masked physical address
    pub fn address(&self) -> *mut u8 {
        (self.0 & Self::BIT_52_ADDRESS) as *mut u8
    }

    /// Get a reference to the item if it's present
    ///
    /// # Example
    /// ```
    /// let item = PageTableEntry::new(0, 0);
    ///
    /// assert!(item.get_item().is_none())
    /// ```
    pub fn get_item(&self) -> Option<&L> {
        if self.get_present() {
            unsafe { (self.address() as *const L).as_ref() }
        } else {
            None
        }
    }

    /// Get a mutable reference to the item sif it's present
    ///
    /// # Example
    /// ```
    /// let item = PageTableEntry::new(0, 0);
    ///
    /// assert!(item.get_item_mut().is_none())
    /// ```
    pub fn get_item_mut(&mut self) -> Option<&mut L> {
        if self.get_present() {
            unsafe { (self.address() as *mut L).as_mut() }
        } else {
            None
        }
    }
}

impl<L: Display> core::fmt::Display for PageTableEntry<L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<L: core::fmt::Debug> core::fmt::Debug for PageTableEntry<L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Clone)]
#[repr(align(4096), C)]
/// Level 4 paging table
pub struct TableLevel4 {
    /// Entries in the table
    pub data: [PageTableEntry<TableLevel3>; 512],
}

impl Display for TableLevel4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self.data.iter().filter_map(|i| i.get_item()) {
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl TableLevel4 {
    /// Get a mutable reference to the page 3 table at index, if it's present
    pub fn sub_table(&mut self, index: usize) -> Option<&mut TableLevel3> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the page 3 table at the index, allocating a new frame if it's not present
    pub fn sub_table_create(&mut self, index: usize) -> &mut TableLevel3 {
        let entry = &mut self.data[index];
        if entry.unused() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            *entry = PageTableEntry::new(ptr as u64, 0);
        }
        entry.get_item_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
#[repr(align(4096), C)]
/// Level 3 paging table
pub struct TableLevel3 {
    /// Entries in the table
    pub data: [PageTableEntry<TableLevel2>; 512],
}

impl Display for TableLevel3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self.data.iter().filter_map(|i| i.get_item()) {
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl TableLevel3 {
    /// Get a mutable reference to the page 2 table at index, if it's present
    pub fn sub_table(&mut self, index: usize) -> Option<&mut TableLevel2> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the page 2 table at the index, allocating a new frame if it's not present
    pub fn sub_table_create(&mut self, index: usize) -> &mut TableLevel2 {
        let entry = &mut self.data[index];
        if entry.unused() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            *entry = PageTableEntry::new(ptr as u64, 0);
        }
        entry.get_item_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
#[repr(align(4096), C)]
/// Level 2 paging table
pub struct TableLevel2 {
    /// Entries in the table
    pub data: [PageTableEntry<TableLevel1>; 512],
}

impl Display for TableLevel2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self.data.iter().filter_map(|i| i.get_item()) {
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl TableLevel2 {
    /// Get a mutable reference to the page 1 table at index, if it's present
    pub fn sub_table(&mut self, index: usize) -> Option<&mut TableLevel1> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the page 1 table at the index, allocating a new frame if it's not present
    pub fn sub_table_create(&mut self, index: usize) -> &mut TableLevel1 {
        let entry = &mut self.data[index];
        if entry.unused() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            *entry = PageTableEntry::new(ptr as u64, 0);
        }
        entry.get_item_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
#[repr(align(4096), C)]
/// Level 1 paging table
pub struct TableLevel1 {
    /// Entries in the table
    pub data: [PageTableEntry<Frame>; 512],
}

impl Display for TableLevel1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self.data.iter().filter(|i| !i.get_present()) {
            write!(f, "{:?}", item)?;
        }
        Ok(())
    }
}

impl TableLevel1 {
    /// Get a mutable reference to the frame at index, if it's present
    pub fn frame(&mut self, index: usize) -> Option<&mut Frame> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the frame at the index, allocating a new frame if it's not present
    pub fn frame_create(&mut self, index: usize) -> &mut Frame {
        let entry = self.data[index].clone();
        if !entry.get_present() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            self.data[index] = PageTableEntry::new(ptr as u64, 0);
        }
        self.data[index].get_item_mut().unwrap()
    }

    /// Set the frame at the specified index to the provided one
    ///
    /// # Example
    /// ```
    /// let mut p1 = { /* code getting the desired level 1 paging table */ };
    /// let frame = Frame::new(0x5000);
    /// let _ = p1.frame_set_specified(0, frame);
    /// ```
    pub fn frame_set_specified(&mut self, index: usize, src: Frame) -> &mut Frame {
        let new =
            (src.0 | PageTableEntry::<Frame>::PRESENT | PageTableEntry::<Frame>::WRITABLE) as u64;

        unsafe { core::ptr::write_volatile(self.data.as_ptr().add(index) as *mut u64, new) }

        let entry = &mut self.data[index];

        entry.get_item_mut().unwrap()
    }
}

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
    type VMMResult<T> = Result<T, super::VirtualMemoryManagerError>;

    /// Initialize the virtual memory manager
    unsafe fn init(&self, mmap: &MemoryMapStructure) -> Self::VMMResult<()> {
        let max = mmap.memmap.iter().last().unwrap();
        let begin_at = u64::MAX - max.end() as u64;
        println!("Beginning at 0x{:x}", begin_at);
        for i in (4096..max.end()).step_by(4096) {
            let frame = Frame::new(i as *mut u8);
            let page = Page::new((begin_at + i as u64) as *mut u8);
            self.map(frame, page, 0).unwrap();
        }
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
    fn virtual_to_physical(&self, src: VirtualAddress) -> Option<PhysicalAddress> {
        let src = Page::new(src);

        let p4 = unsafe { Self::get_p4_table() };

        let p3 = p4.sub_table(src.p4_index())?;
        println!("Got P3");

        let p2_raw = p3.data[src.p3_index()].clone();

        if p2_raw.get_huge_page() && p2_raw.get_present() {
            println!("P2 Huge: {:?}", p2_raw.address());
            return unsafe { Some(p2_raw.address().add((src.0 & 0x3FFF_FFFF) as usize)) };
        }

        let p2 = p3.sub_table(src.p3_index())?;

        println!("Got P2");

        let p1_raw = p2.data[src.p2_index()].clone();

        if p1_raw.get_present() && p1_raw.get_huge_page() {
            println!("P1 Huge: 0x{:x}", p1_raw.0);
            println!("P1 Offset: 0x{:x}", p1_raw.0 & 0x1F_FFFF);
            return unsafe { Some(p1_raw.address().add((p1_raw.0 & 0x1F_FFFF) as usize)) };
        }

        let p1 = p2.sub_table(src.p2_index())?;

        println!("Got P1");

        let frame = p1.frame(src.p1_index())?;
        let offset = src.frame_offset();

        println!("Offset: 0x{:x}", offset);

        Some(unsafe { frame.address().add(offset) } as *mut u8)
    }

    /// Map the specified frame to the destination, with the option to provide additional flags
    ///
    /// # Example
    /// ```
    /// let frame = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
    /// let page = Page::new(0xdeadc000).unwrap();
    ///
    /// let _ = MEMORY_MANAGER.map(frame, page, 0).unwrap();
    fn map(&self, src: Frame, dst: Page, flags: u64) -> Self::VMMResult<()> {
        let mut src: Frame = (src.0 | flags).into();
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

    fn unmap(&self, src: Page) -> Self::VMMResult<()> {
        let base = Page::new(src.base_address().0);
        let p4 = unsafe { Self::get_p4_table() };
        let p3 = p4
            .sub_table(base.p4_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        if p3.data[base.p3_index()].get_huge_page() {
            p3.data[base.p3_index()].0 = 0;
        }

        let p2 = p3
            .sub_table(base.p3_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        if p2.data[base.p2_index()].get_huge_page() {
            p2.data[base.p2_index()].0 = 0;
        }

        let p1 = p2
            .sub_table(base.p2_index())
            .ok_or(VirtualMemoryManagerError::PageNotFound)?;

        p1.data[base.p1_index()].0 = 0;

        Ok(())
    }
}
