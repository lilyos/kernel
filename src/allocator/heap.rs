use crate::sync::Mutex;

extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};

use core::{mem, ptr};

#[derive(Clone, Copy)]
pub struct FreeRegion {
    size: usize,
    next: *mut Self,
}

impl FreeRegion {
    pub const fn new(size: usize) -> Self {
        Self {
            size,
            next: ptr::null_mut(),
        }
    }

    pub fn start(&self) -> usize {
        self as *const Self as usize
    }

    pub fn end(&self) -> usize {
        self.start() + self.size
    }
}

pub struct HeapAllocator {
    //  _data: Mutex<HeapAllocatorInternalData>,
    head: Mutex<FreeRegion>,
}

impl HeapAllocator {
    pub const fn new() -> Self {
        HeapAllocator {
            head: Mutex::new(FreeRegion::new(0)),
        }
    }

    pub unsafe fn init(&self, start: usize, size: usize) {
        self.add_free_region(start, size);
    }

    pub unsafe fn add_free_region(&self, addr: usize, size: usize) {
        assert!(align(addr, mem::align_of::<FreeRegion>()) == addr);
        assert!(size >= mem::size_of::<FreeRegion>());

        let mut head = self.head.lock();
        let mut new = FreeRegion::new(size);
        new.next = head.next;

        let ptr = addr as *mut FreeRegion;
        ptr.write(new);

        head.next = addr as *mut FreeRegion;
    }

    pub fn find_region(&self, size: usize, alignment: usize) -> Option<(*mut FreeRegion, usize)> {
        let mut current = {
            let x = self.head.lock();
            (*x).clone()
        };
        while !current.next.is_null() {
            let region = unsafe { &mut *current.next };
            if let Ok(alloc) = self.check_region_allocation(&current, size, alignment) {
                let next = region.next;
                let val = Some((current.next, alloc));
                current.next = next;
                return val;
            } else {
                if !current.next.is_null() {
                    current = unsafe { *current.next };
                } else {
                    break;
                }
            }
        }
        None
    }

    fn check_region_allocation(
        &self,
        region: *const FreeRegion,
        size: usize,
        alignment: usize,
    ) -> Result<usize, ()> {
        let region = unsafe { *region };

        let start = align(region.start(), alignment);
        let end = start.checked_add(size).ok_or(())?;

        if end > region.end() {
            return Err(());
        }

        let spare = region.end() - end;
        if spare > 0 && spare < mem::size_of::<FreeRegion>() {
            return Err(());
        }

        Ok(start)
    }

    fn layout_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<FreeRegion>())
            .expect("Failed to adjust alignment")
            .pad_to_align();
        (
            layout.size().max(mem::size_of::<FreeRegion>()),
            layout.align(),
        )
    }
}

/// Must be power of 2
fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = Self::layout_align(layout);
        if let Some((region, start)) = self.find_region(size, align) {
            let region = &*region;
            let end = start
                .checked_add(size)
                .expect("Integer overflow when calculating end of region");
            let spare = region.end() - end;
            if spare > 0 {
                self.add_free_region(end, spare);
            }
            start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = Self::layout_align(layout);

        self.add_free_region(ptr as usize, size)
    }
}
