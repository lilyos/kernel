use crate::{
    allocator::{align, AllocatorError, GrowableSlice},
    println,
    sync::Mutex,
};

extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};

use core::{mem, ptr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreeRegion {
    start: *mut u8,
    size: usize,
}

impl FreeRegion {
    pub const fn new(start: *mut u8, size: usize) -> Self {
        Self { start, size }
    }

    pub fn end(&self) -> *const u8 {
        unsafe { self.start.add(self.size) }
    }
}

pub struct HeapAllocator {
    storage: Mutex<GrowableSlice<FreeRegion>>,
}

static DIV: &str = "================================================================";

impl HeapAllocator {
    pub const fn new() -> Self {
        HeapAllocator {
            storage: Mutex::new(GrowableSlice::new()),
        }
    }

    pub unsafe fn init(
        &self,
        start: *mut u8,
        size: usize,
        scratch_start: *mut u8,
        scratch_size: usize,
    ) -> Result<(), AllocatorError> {
        {
            println!("Locking, setting new storage");
            let mut storage = self.storage.lock();
            storage.init(scratch_start, scratch_size);
            println!("Done; Adding new region");
        }
        self.add_free_region(start, size)?;
        Ok(())
    }

    pub unsafe fn add_free_region(&self, addr: *mut u8, size: usize) -> Result<(), AllocatorError> {
        self.join_nearby();
        println!("Locking storage");
        let items = &mut *self.storage.lock();
        println!("Pushing");
        let v = items.push(FreeRegion::new(addr, size));
        println!("Returned");
        v
    }

    pub fn find_region(&self, size: usize, alignment: usize) -> Option<(FreeRegion, usize)> {
        let items = &mut *self.storage.lock();
        if let Some(found) =
            items.find(|item| self.check_region_allocation(&item, size, alignment).is_ok())
        {
            return Some((
                found,
                self.check_region_allocation(&found, size, alignment)
                    .unwrap(),
            ));
        }
        None
    }

    fn check_region_allocation(
        &self,
        region: &FreeRegion,
        size: usize,
        alignment: usize,
    ) -> Result<usize, AllocatorError> {
        let alloc_start = align(region.start as usize, alignment);
        let alloc_end = alloc_start
            .checked_add(size)
            .ok_or(AllocatorError::InternalError(94))?;

        println!("Region: {:#?}", region);
        println!(
            "Alloc Start: {alloc_start}, Alloc End: {alloc_end}, Region Start: {:?}, Region End: {:?}",
            region.start,
            region.end()
        );

        if alloc_end > region.end() as usize {
            return Err(AllocatorError::RegionTooSmall);
        }

        let spare = region.end() as usize - alloc_end;
        if spare > 0 && spare < mem::size_of::<FreeRegion>() {
            return Err(AllocatorError::SpareTooSmall);
        }

        Ok(alloc_start)
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

    pub fn display(&self) {
        println!("{}", DIV);

        let items = self.storage.lock();

        for item in items.storage.iter() {
            println!("Allocator Node {:#?}", item);
        }

        println!("{}", DIV);
    }

    fn join_nearby(&self) {}
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = Self::layout_align(layout);
        if let Some((region, start)) = self.find_region(size, align) {
            let rend = region.end();
            let end = start
                .checked_add(size)
                .expect("Integer overflow when calculating end of region");

            println!("Start ({}) + Size ({}) = End ({})", start, size, end);

            let spare = rend as usize - end;
            println!(
                "Region End ({:?}) - End ({:?}) = Spare ({:?})",
                rend, end, spare
            );

            // println!("Spare: 0x{:x}", spare);
            if spare > 0 {
                match self.add_free_region((end + 1) as *mut u8, spare) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Allocation failed: {e:?}");
                        return ptr::null_mut();
                    }
                }
            }

            // println!("Sorting regions");
            // self.sort_regions();
            // println!("Allocated {} bytes", size);
            self.display();

            start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = Self::layout_align(layout);

        // println!("Adding free region");
        self.add_free_region(ptr, size)
            .expect("Failed to deallocate memory, {layout:#?}");
        // println!("Joining nearby regions");
        self.join_nearby();
        // println!("Deallocated {} bytes", size);
        self.display();
    }
}
