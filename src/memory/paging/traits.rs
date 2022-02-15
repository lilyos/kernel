use core::marker::PhantomData;

use kernel_macros::bit_field_accessors;

use crate::memory::allocators::MemoryDescriptor;

pub type VirtualAddress = *mut u8;
pub type PhysicalAddress = *mut u8;

#[derive(Debug, Clone, Copy)]
pub struct PageAlignedAddress<L>(pub *mut u8, PhantomData<L>);

impl<L> PageAlignedAddress<L> {
    /// Make a new page-aligned virtual address
    /// This will return an error if it's not page aligned. Duh.
    pub fn new(address: *mut u8) -> Result<Self, ()> {
        if address as usize % 4096 != 0 {
            Err(())
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

pub trait VirtualMemoryManager {
    /// Result type for the virtual memory manager
    type VMMResult<T> = Result<T, VirtualMemoryManagerError>;

    /// Initialize the virtual memory manager
    ///
    /// # Arguments
    /// * `mmap` - A slice of MemoryDescriptor
    unsafe fn init(&self, mmap: &[MemoryDescriptor]) -> Self::VMMResult<()>;

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
    pub unsafe fn init(&self, mmap: &[MemoryDescriptor]) -> T::VMMResult<()> {
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

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Frame(pub u64);
impl From<u64> for Frame {
    fn from(item: u64) -> Self {
        Self(item)
    }
}

impl Frame {
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

    pub fn new(address: PhysicalAddress) -> Self {
        assert_eq!(address as u64 & !0x000F_FFFF_FFFF_F000, 0);
        let mut tmp = Self(address as u64);
        tmp.set_present();
        tmp.set_writable();
        tmp
    }

    pub fn address(&self) -> PhysicalAddress {
        (self.0 & Self::BIT_52_ADDRESS) as *mut u8
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Page(pub u64);

impl From<u64> for Page {
    fn from(item: u64) -> Self {
        Self(item)
    }
}

impl Page {
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

    pub fn new(address: VirtualAddress) -> Self {
        let mut tmp = match address as u64 >> 47 {
            0 | 0x1FFF => Self(address as u64),
            _ => panic!("This shouldn't happen"),
        };
        tmp.set_present();
        tmp.set_writable();
        tmp
    }

    pub fn address(&self) -> VirtualAddress {
        (self.0) as *mut u8
    }

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
