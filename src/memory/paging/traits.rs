use core::marker::PhantomData;

use kernel_macros::bit_field_accessors;
use stivale2::boot::tags::structures::MemoryMapStructure;

use crate::memory::allocators::is_address_canonical;

/// A virtual address
pub type VirtualAddress = *mut u8;

/// A physical address
pub type PhysicalAddress = *mut u8;

/// Errors from PageAlignedAddress
#[derive(Debug)]
pub enum PageAlignedAddressError {
    /// The address was unaligned
    AddressUnaligned,
}

/// A page aligned address
#[derive(Debug, Clone, Copy)]
pub struct PageAlignedAddress<L>(pub *mut u8, PhantomData<L>);

impl<L> PageAlignedAddress<L> {
    /// Make a new page-aligned virtual address
    /// This will return an error if it's not page aligned. Duh.
    pub fn new(address: *mut u8) -> Result<Self, PageAlignedAddressError> {
        if address as usize % 4096 != 0 {
            Err(PageAlignedAddressError::AddressUnaligned)
        } else {
            Ok(Self(address, PhantomData))
        }
    }
}

impl<L> From<PageAlignedAddress<L>> for *mut u8 {
    fn from(val: PageAlignedAddress<L>) -> Self {
        val.0
    }
}

/// Errors for the Virtual Memory Manager
#[derive(Debug)]
pub enum VirtualMemoryManagerError {
    /// The requested feature isn't implemented
    NotImplemented,
    /// Huge pages can't have children
    AttemptedToMapToHugePage,
    /// The desired page in the virtual address doesn't exist
    PageNotFound,
}

/// The trait that a Virtual Memory Maager must implement
pub trait VirtualMemoryManager {
    /// Result type for the virtual memory manager
    type VMMResult<T> = Result<T, VirtualMemoryManagerError>;

    /// Initialize the virtual memory manager
    ///
    /// # Arguments
    /// * `mmap` - A slice of MemoryDescriptor
    ///
    /// # Safety
    /// The mmap must be correctly formed and all writes and reads must be able succeed
    unsafe fn init(&self, mmap: &MemoryMapStructure) -> Self::VMMResult<()>;

    /// Convert a virtual address to a physical address
    ///
    /// # Arguments
    /// * `src` - The virtual address to convert to a physical address
    fn virtual_to_physical(&self, src: VirtualAddress) -> Option<PhysicalAddress>;

    /// Map a physical address to a virtual address
    ///
    /// # Arguments
    /// * `src` - The physical address to map
    /// * `dst` - The address to map to
    /// * `flags` - Additional flags for the virtual address
    fn map(&self, src: Frame, dst: Page, flags: u64) -> Self::VMMResult<()>;

    /// Unmap a virtual address
    ///
    /// # Arguments
    /// * `src` - The address to unmap
    fn unmap(&self, src: Page) -> Self::VMMResult<()>;
}

/// Wrapper for virtual memory managers
pub struct MemoryManager<T>(pub T)
where
    T: VirtualMemoryManager;

impl<T> MemoryManager<T>
where
    T: VirtualMemoryManager,
{
    /// Create a new virtal memory manager wrapper
    pub const fn new(i: T) -> Self {
        Self(i)
    }

    /// Initialize the virtual memory manager
    ///
    /// # Arguments
    /// * `mmap` - A slice of MemoryDescriptor
    ///
    /// # Safety
    /// The mmap must be properly formed and all reads and writes must be able to succeed
    pub unsafe fn init(&self, mmap: &MemoryMapStructure) -> T::VMMResult<()> {
        self.0.init(mmap)
    }

    /// Convert a virtual address to a physical address
    ///
    /// # Arguments
    /// * `src` - The virtual address to convert to a physical address
    pub fn virtual_to_physical(&self, src: VirtualAddress) -> Option<VirtualAddress> {
        self.0.virtual_to_physical(src)
    }

    /// Map a physical address to a virtual address
    ///
    /// # Arguments
    /// * `src` - The physical address to map
    /// * `dst` - The address to map to
    /// * `flags` - Additional flags for the virtual address
    pub fn map(
        &self,
        src: PageAlignedAddress<PhysicalAddress>,
        dst: PageAlignedAddress<VirtualAddress>,
        flags: u64,
    ) -> T::VMMResult<()> {
        self.0
            .map(Frame::new(src.into()), Page::new(dst.into()), flags)
    }

    /// Unmap a virtual address
    ///
    /// # Arguments
    /// * `src` - The address to unmap
    pub fn unmap(&self, src: PageAlignedAddress<VirtualAddress>) -> T::VMMResult<()> {
        self.0.unmap(Page::new(src.into()))
    }
}

/// An address to a frame
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Frame(pub u64);

impl From<u64> for Frame {
    fn from(item: u64) -> Self {
        Self(item)
    }
}

impl Frame {
    /// The physical address mask
    pub const BIT_52_ADDRESS: u64 = 0x000F_FFFF_FFFF_F000;

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

    /// Create a new frame from a physical address and set it present and writable
    pub fn new(address: PhysicalAddress) -> Self {
        let mut tmp = Self(address as u64);
        tmp.set_present();
        tmp.set_writable();
        tmp
    }

    /// Get the contained address
    pub fn address(&self) -> PhysicalAddress {
        (self.0 & Self::BIT_52_ADDRESS) as *mut u8
    }
}

/// An address to a page
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Page(pub u64);

impl From<u64> for Page {
    fn from(item: u64) -> Self {
        Self(item)
    }
}

impl Page {
    /// The physical address mask
    pub const BIT_52_ADDRESS: u64 = 0x000F_FFFF_FFFF_F000;

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

    /// Create a new page
    pub fn new(address: VirtualAddress) -> Self {
        if !is_address_canonical(address as usize, 48) {
            panic!("The address is non-canonical")
        }
        let mut tmp = Self(address as u64);
        tmp.set_present();
        tmp.set_writable();
        tmp
    }

    /// Get the contained address
    pub fn address(&self) -> VirtualAddress {
        (self.0) as *mut u8
    }

    /// Get a page-aligned address containing the stored value
    pub fn base_address(&self) -> PageAlignedAddress<VirtualAddress> {
        PageAlignedAddress::new((self.0 & Self::BIT_52_ADDRESS) as *mut u8).unwrap()
    }

    /// Bits 39-47
    pub fn p4_index(&self) -> usize {
        (self.0 as usize >> 39) & 0x1FF
    }

    /// Bits 30-38
    pub fn p3_index(&self) -> usize {
        (self.0 as usize >> 30) & 0x1FF
    }

    /// Bits 21-29
    pub fn p2_index(&self) -> usize {
        (self.0 as usize >> 21) & 0x1FF
    }

    /// Bits 12-20
    pub fn p1_index(&self) -> usize {
        (self.0 as usize >> 12) & 0x1FF
    }

    /// Bits 0-11
    pub fn frame_offset(&self) -> usize {
        self.0 as usize & 0xFFFF
    }
}

impl core::fmt::Debug for Page {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Page {{\n\tLevel4Index: {},\n\tLevel3Index: {},\n\tLevel2Index: {},\n\tLevel1Index: {},\n\tFrameOffset: {} (0x{:x}),\n\tBaseAddress: {:?},\n}}", self.p4_index(), self.p3_index(), self.p2_index(), self.p1_index(), self.frame_offset(), self.frame_offset(), (self.address() as u64 & Self::BIT_52_ADDRESS) as *mut u8)
        // f.debug_tuple("Page").finish()
    }
}
