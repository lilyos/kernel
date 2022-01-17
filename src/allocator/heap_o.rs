use super::align;
use crate::sync::Mutex;

use crate::println;

extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};

use core::{mem, ptr};

#[derive(Debug, Clone, Copy)]
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
    head: Mutex<FreeRegion>,
}

static DIV: &str = "================================================================";

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

        head.next = ptr;
    }

    pub fn find_region(&self, size: usize, alignment: usize) -> Option<(&FreeRegion, usize)> {
        let mut current: &mut FreeRegion = { &mut *self.head.lock() };
        while !current.next.is_null() {
            let region = unsafe { &mut *current.next };
            if let Ok(alloc) = self.check_region_allocation(region, size, alignment) {
                let next = region.next;
                let val = Some((&*region, alloc));
                current.next = next;
                return val;
            }
            current = region;
        }
        None
    }

    fn check_region_allocation(
        &self,
        region: &FreeRegion,
        size: usize,
        alignment: usize,
    ) -> Result<usize, ()> {
        let alloc_start = align(region.start(), alignment);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        /*
        println!(
            "Alloc Start: 0x{:x}, Alloc End: 0x{:x}, Region Start: 0x{:x}, Region End: 0x{:x}",
            alloc_start,
            alloc_end,
            region.start(),
            region.end()
        );
        */

        println!("Region: {:#?}", region);
        println!(
            "Alloc Start: {}, Alloc End: {}, Region Start: {}, Region End: {}",
            alloc_start,
            alloc_end,
            region.start(),
            region.end()
        );

        if alloc_end > region.end() {
            return Err(());
        }

        let spare = region.end() - alloc_end;
        if spare > 0 && spare < mem::size_of::<FreeRegion>() {
            return Err(());
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
        let mut current: *const FreeRegion = { &*self.head.lock() };
        while !current.is_null() {
            let region = unsafe { &*current };
            println!("Allocator Node (0x{:x}): {:#?}", region.start(), region);
            if !region.next.is_null() {
                println!();
            }
            current = region.next;
        }
        println!("{}", DIV);
    }

    /*
    fn sort_regions(&self) {
        let mut current = unsafe { *self.head.lock() }.next;
        let mut index: *mut FreeRegion;

        while !current.is_null() {
            index = unsafe { *current }.next;
            while !index.is_null() {
                let currentr = unsafe { &mut *current };
                let indexr = unsafe { &mut *index };
                assert!(currentr.start() != indexr.start());
                if currentr.next > indexr.next {
                    println!("Swapping:\n{:#?}\n{:#?}", currentr, indexr);
                    core::mem::swap(currentr, indexr);
                    core::mem::swap(&mut currentr.next, &mut indexr.next);
                    println!(
                        "Swapped:\n{:#?} (0x{:x})\n{:#?} (0x{:x})",
                        currentr,
                        currentr.start(),
                        indexr,
                        indexr.start()
                    );
                }
                index = indexr.next;
            }
            current = unsafe { &*current }.next;
        }
    }
    */

    fn join_nearby(&self) {
        let mut current = { &mut *self.head.lock() };
        while !current.next.is_null() {
            let region1 = unsafe { &mut *current.next };
            if !region1.next.is_null() {
                let region2 = unsafe { &mut *region1.next };
                if region1.next as usize == region2.start() {
                    region1.size += region2.size;
                    region1.next = region2.next;
                }
            }
            /*
             else {
                println!("Region 1 (0x{:x}): {:?}", region1.start(), region1);
                if !region1.next.is_null() {
                    let region2 = unsafe { &*region1.next };
                    println!("Region 2 (0x{:x}): {:?}", region2.start(), region2);
                }
            }
            */
            current = region1;
        }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = Self::layout_align(layout);
        if let Some((region, mut start)) = self.find_region(size, align) {
            let region = &*region;
            let mut rend = region.end();
            let end = start
                .checked_add(size)
                .expect("Integer overflow when calculating end of region");

            println!("Start ({}) + Size ({}) = End ({})", start, size, end);

            let spare = rend - end;
            println!("Region End ({}) - End ({}) = Spare ({})", rend, end, spare);
            // println!("Spare: 0x{:x}", spare);
            if spare > 0 {
                self.add_free_region(end, spare);
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
        self.add_free_region(ptr as usize, size);
        // println!("Joining nearby regions");
        self.join_nearby();
        // println!("Deallocated {} bytes", size);
        self.display();
    }
}
