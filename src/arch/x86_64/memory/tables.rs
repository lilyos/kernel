use core::{alloc::Layout, fmt::Display, marker::PhantomData};

use crate::{
    errors::{MemoryManagerError, PhysicalAllocatorError},
    get_memory_manager,
    memory::addresses::{AlignedAddress, Physical},
    traits::{MemoryFlags, MemoryManager, PhysicalAllocator, PlatformAddress},
};

use super::addresses::AddressWithFlags;

const FRAME_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(4096, 4096) };

pub fn allocate_frame<A: PhysicalAllocator>(
    allocator: A,
) -> Result<AlignedAddress<Physical>, PhysicalAllocatorError> {
    allocator.allocate(FRAME_LAYOUT)
}

pub fn allocate_frame_platform_alloc() -> Result<AlignedAddress<Physical>, PhysicalAllocatorError> {
    allocate_frame(&crate::PHYSICAL_ALLOCATOR)
}

pub fn deallocate_frame<A: PhysicalAllocator>(allocator: A, frame: AlignedAddress<Physical>) {
    unsafe { allocator.deallocate(frame, FRAME_LAYOUT) }
}

pub fn deallocate_frame_platform_alloc(frame: AlignedAddress<Physical>) {
    deallocate_frame(&crate::PHYSICAL_ALLOCATOR, frame)
}

#[repr(transparent)]
#[derive(Clone)]
/// An entry in a page table of type L
pub struct PageTableEntry<L>(pub AddressWithFlags, PhantomData<L>);

impl<L> PageTableEntry<L> {
    /// Address mask for Virtual Addresses
    pub const BIT_52_ADDRESS: usize = 0x000F_FFFF_FFFF_F000;

    /// Create a new PageTable Entry
    ///
    /// # Example
    /// ```
    /// let address = 0x5000;
    /// let entry: PageTableEntry<Frame> = PageTableEntry::new(0x5000 & PageTableEntry::BIT_52_ADDRESS, 0);
    /// ```
    pub fn new<T>(address: AlignedAddress<T>, flags: MemoryFlags) -> Self {
        // let mut tmp = Self((address & Self::BIT_52_ADDRESS) | extra_flags, PhantomData);
        let mut tmp = Self(
            unsafe { AddressWithFlags::from_bits_unchecked(address.inner().into_raw()) },
            PhantomData,
        );
        {
            let addr_flags = tmp.get_flags_mut();
            addr_flags.insert(AddressWithFlags::PRESENT);

            addr_flags.insert(
                AddressWithFlags::PRESENT
                    | AddressWithFlags::USER_ACCESSIBLE
                    | AddressWithFlags::NO_EXECUTE
                    | AddressWithFlags::DISABLE_CACHE
                    | AddressWithFlags::WRITE_THROUGH_CACHING,
            );

            for (_, flag) in flags.iter() {
                match flag {
                    MemoryFlags::READABLE => {}
                    MemoryFlags::WRITABLE => addr_flags.insert(AddressWithFlags::WRITABLE),
                    MemoryFlags::KERNEL_ONLY => {
                        addr_flags.remove(AddressWithFlags::USER_ACCESSIBLE)
                    }
                    MemoryFlags::CACHABLE => addr_flags.remove(
                        AddressWithFlags::DISABLE_CACHE | AddressWithFlags::WRITE_THROUGH_CACHING,
                    ),
                    MemoryFlags::EXECUTABLE => addr_flags.remove(AddressWithFlags::NO_EXECUTE),
                    _ => unreachable!(),
                }
            }
        }
        tmp
    }

    /// Return the flags
    pub const fn get_flags(&self) -> &AddressWithFlags {
        &self.0
    }

    /// Return the flags as a mutable reference
    pub const fn get_flags_mut(&mut self) -> &mut AddressWithFlags {
        &mut self.0
    }

    /// Return the address as a usize
    pub const fn get_address(&self) -> usize {
        self.get_flags().bits() as usize
    }

    /// Returns if the entry is unused (equal to zero)
    pub const fn is_unused(&self) -> bool {
        self.get_flags().is_empty()
    }

    /// Get the virtual address of the contained item
    pub const fn get_ptr(&self) -> *const L {
        (self.get_address() & Self::BIT_52_ADDRESS) as *const L
    }

    /// Get the virtual address of the contained item mutably
    pub const fn get_ptr_mut(&mut self) -> *mut L {
        (self.get_address() & Self::BIT_52_ADDRESS) as *mut L
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
        if self.get_flags().contains(AddressWithFlags::PRESENT) {
            unsafe { self.get_ptr().as_ref() }
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
        if self.get_flags().contains(AddressWithFlags::PRESENT) {
            unsafe { self.get_ptr_mut().as_mut() }
        } else {
            None
        }
    }
}

impl<L: Display> core::fmt::Display for PageTableEntry<L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
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
    /// Get a reference to the page 3 table at `index`, if it's present
    pub fn sub_table(&self, index: usize) -> Option<&TableLevel3> {
        let entry = &self.data[index];
        entry.get_item()
    }

    /// Get a mutable reference to the page 3 table at `index`, if it's present
    pub fn sub_table_mut(&mut self, index: usize) -> Option<&mut TableLevel3> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the page 3 table at the index, allocating a new frame if it's not present
    pub fn sub_table_create(
        &mut self,
        index: usize,
        flags: MemoryFlags,
    ) -> Result<&mut TableLevel3, MemoryManagerError> {
        let entry = &mut self.data[index];
        if entry.is_unused() {
            let virt_addr = unsafe {
                get_memory_manager().allocate_and_map(
                    get_memory_manager().get_current_table()?,
                    (*crate::SAFE_UPPER_HALF_RANGE).clone(),
                    flags,
                    FRAME_LAYOUT,
                )
            }?;

            *entry = PageTableEntry::new(virt_addr, flags);
        }

        Ok(entry.get_item_mut().unwrap())
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
    /// Get a reference to the page 2 table at `index`, if it's present
    pub fn sub_table(&self, index: usize) -> Option<&TableLevel2> {
        let entry = &self.data[index];
        entry.get_item()
    }

    /// Get a mutable reference to the page 2 table at `index`, if it's present
    pub fn sub_table_mut(&mut self, index: usize) -> Option<&mut TableLevel2> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the page 2 table at the index, allocating a new frame if it's not present
    pub fn sub_table_create(
        &mut self,
        index: usize,
        flags: MemoryFlags,
    ) -> Result<&mut TableLevel2, MemoryManagerError> {
        let entry = &mut self.data[index];
        if entry.is_unused() {
            let virt_addr = unsafe {
                get_memory_manager().allocate_and_map(
                    get_memory_manager().get_current_table()?,
                    (*crate::SAFE_UPPER_HALF_RANGE).clone(),
                    flags,
                    FRAME_LAYOUT,
                )
            }?;

            *entry = PageTableEntry::new(virt_addr, flags);
        }
        Ok(entry.get_item_mut().unwrap())
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
    /// Get a reference to the page 1 table at `index`, if it's present
    pub fn sub_table(&self, index: usize) -> Option<&TableLevel1> {
        let entry = &self.data[index];
        entry.get_item()
    }

    /// Get a mutable reference to the page 1 table at `index`, if it's present
    pub fn sub_table_mut(&mut self, index: usize) -> Option<&mut TableLevel1> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the page 1 table at the index, allocating a new frame if it's not present
    pub fn sub_table_create(
        &mut self,
        index: usize,
        flags: MemoryFlags,
    ) -> Result<&mut TableLevel1, MemoryManagerError> {
        let entry = &mut self.data[index];
        if entry.is_unused() {
            let virt_addr = unsafe {
                get_memory_manager().allocate_and_map(
                    get_memory_manager().get_current_table()?,
                    (*crate::SAFE_UPPER_HALF_RANGE).clone(),
                    flags,
                    FRAME_LAYOUT,
                )
            }?;

            *entry = PageTableEntry::new(virt_addr, flags);
        }
        Ok(entry.get_item_mut().unwrap())
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
        for item in self
            .data
            .iter()
            .filter(|i| !i.get_flags().contains(AddressWithFlags::PRESENT))
        {
            write!(f, "{:?}", item)?;
        }

        Ok(())
    }
}

impl TableLevel1 {
    /// Get a reference to the frame at `index`, if it's present
    pub fn frame(&self, index: usize) -> Option<&AlignedAddress<Physical>> {
        let entry = &self.data[index];
        entry.get_item()
    }

    /// Get a mutable reference to the frame at `index`, if it's present
    pub fn frame_mut(&mut self, index: usize) -> Option<&mut AlignedAddress<Physical>> {
        let entry = &mut self.data[index];
        entry.get_item_mut()
    }

    /// Get a mutable reference to the frame at the index, allocating a new frame if it's not present
    pub fn frame_create(
        &mut self,
        index: usize,
        flags: MemoryFlags,
    ) -> Result<&mut AlignedAddress<Physical>, MemoryManagerError> {
        let entry = self.data[index].clone();
        if !entry.get_flags().contains(AddressWithFlags::PRESENT) {
            let addr = allocate_frame_platform_alloc()
                .map_err(|e| MemoryManagerError::PhysicalAllocator(e))?;
            self.data[index] = PageTableEntry::new(addr, flags);
        }

        Ok(self.data[index].get_item_mut().unwrap())
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
        flags: MemoryFlags,
    ) -> &mut AlignedAddress<Physical> {
        self.data[index] = PageTableEntry::new(src.into(), flags);

        let entry = &mut self.data[index];

        entry.get_item_mut().unwrap()
    }
}
