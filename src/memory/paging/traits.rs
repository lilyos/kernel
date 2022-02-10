use core::ops::{BitOr, BitOrAssign};

use kernel_macros::bit_field_accessors;

use crate::memory::allocators::MemoryDescriptor;

pub type VirtualAddress = *mut u8;
pub type PhysicalAddress = *mut u8;

/// Errors for the Virtual Memory Manager
#[derive(Debug)]
pub enum VirtualMemoryManagerError {}

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
pub struct MemoryManager<T>(T)
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
    pub fn map(&self, src: Frame, dst: Page, flags: u64) -> T::VMMResult<()> {
        self.0.map(src, dst, flags)
    }

    /// Unmap a virtual address
    ///
    /// # Arguments
    /// * `src` - The address to unmap
    pub fn unmap(&self, src: Page) -> T::VMMResult<()> {
        self.0.unmap(src)
    }
}

/// Flags for frames and pagess
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Flags(u64);

impl Flags {
    bit_field_accessors! {
        present 0
        writable 1
        user_accessible 2
        write_through_caching 3
        disable_cache 4
        accessed 5
        dirty 6
        huge_page 7
        global 8
        // 9-11 Free Use
        reserved 9
        // 52-62 Free Use
        no_execute 63
    }

    /// Create a new flags instance
    pub fn new<T: Into<u64>>(address: T, flags: u64) -> Self {
        Self(flags | address.into())
    }

    /// Get the address in the flags
    pub fn get_address(&self) -> u64 {
        self.0 & 0b0000000000001111111111111111111111111111111111111111000000000000
    }
}

impl BitOr<u64> for Flags {
    type Output = u64;

    fn bitor(self, rhs: u64) -> Self::Output {
        self.0 | rhs
    }
}

impl BitOrAssign for Flags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(4096))]
pub struct Frame {
    inner: Flags,
}

impl Frame {
    pub fn with_address(addr: PhysicalAddress) -> Self {
        Self {
            inner: Flags::new(addr as u64, 0),
        }
    }

    pub fn address(&self) -> *mut u8 {
        self.flags().get_address() as *mut u8
    }

    pub fn flags(&self) -> Flags {
        self.inner
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(4096))]
pub struct Page {
    inner: Flags,
}

impl Page {
    pub fn with_address(addr: VirtualAddress) -> Self {
        Self {
            inner: Flags::new(addr as u64, 0),
        }
    }

    pub fn address(&self) -> *mut u8 {
        self.flags().get_address() as *mut u8
    }

    pub fn flags(&self) -> Flags {
        self.inner
    }

    pub fn flags_mut(&mut self) -> &mut Flags {
        &mut self.inner
    }
}
