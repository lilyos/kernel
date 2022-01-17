// static OFFSET_BASE: isize = 0xFFFF_FFFF_C000;
static OFFSET_BASE: isize = 0;

pub trait Address {
    fn addr(&self) -> *mut u8;
}

pub struct PhysAddr(*mut u8);

impl Address for PhysAddr {
    fn addr(&self) -> *mut u8 {
        unsafe { self.0.offset(OFFSET_BASE) }
    }
}

pub struct VirtAddr(*mut u8);

impl Address for VirtAddr {
    fn addr(&self) -> *mut u8 {
        self.0
    }
}

/// Must be power of 2
pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[derive(Debug)]
pub enum AllocatorError {
    InternalStorageFull,
    NotImplemented,
    NoLargeEnoughRegion,
    SpecifiedRegionNotFree,
    SpareTooSmall,
    RegionTooSmall,
    InternalError(u32),
    OutOfMemory,
}
