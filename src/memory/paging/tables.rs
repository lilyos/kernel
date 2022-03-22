use core::{fmt::Display, marker::PhantomData};

use kernel_macros::bit_field_accessors;

use crate::PHYSICAL_ALLOCATOR;

use super::addresses::{Address, AlignedAddress, Physical};

#[repr(transparent)]
#[derive(Clone)]
/// An entry in a page table of type L
pub struct PageTableEntry<L>(pub usize, PhantomData<L>);

impl<L> PageTableEntry<L> {
    /// Address mask for Virtual Addresses
    pub const BIT_52_ADDRESS: usize = 0x000F_FFFF_FFFF_F000;

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
    pub fn new(address: usize, extra_flags: usize) -> Self {
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
            *entry = PageTableEntry::new(ptr as usize, 0);
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
            *entry = PageTableEntry::new(ptr as usize, 0);
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
            *entry = PageTableEntry::new(ptr as usize, 0);
        }
        entry.get_item_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
#[repr(align(4096), C)]
/// Level 1 paging table
pub struct TableLevel1 {
    /// Entries in the table
    pub data: [PageTableEntry<AlignedAddress<Physical>>; 512],
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
    pub fn frame(&mut self, index: usize) -> Option<&mut AlignedAddress<Physical>> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the frame at the index, allocating a new frame if it's not present
    pub fn frame_create(&mut self, index: usize) -> &mut AlignedAddress<Physical> {
        let entry = self.data[index].clone();
        if !entry.get_present() {
            let (ptr, _) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
            self.data[index] = PageTableEntry::new(ptr as usize, 0);
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
    pub fn frame_set_specified(
        &mut self,
        index: usize,
        src: AlignedAddress<Physical>,
    ) -> &mut AlignedAddress<Physical> {
        self.data[index] = PageTableEntry::new(
            src.get_inner() | Address::<()>::PRESENT | Address::<()>::WRITABLE,
            0,
        );

        let entry = &mut self.data[index];

        entry.get_item_mut().unwrap()
    }
}
