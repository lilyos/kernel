use crate::{
    allocator::{AllocatorError, MemoryDescriptor, MemoryEntry, MemoryKind},
    println,
    sync::Mutex,
};

use core::{
    fmt::{Display, Formatter},
    ptr,
};

static DIV: &str = "================================================================";

#[repr(transparent)]
pub struct PageAllocation {
    pub pages: usize,
}

impl PageAllocation {
    pub const fn new(pages: usize) -> Self {
        Self { pages }
    }

    pub fn start(&self) -> *mut u8 {
        self as *const Self as *mut u8
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FreePage {
    pub usable: bool,
    pub last: *mut FreePage,
    pub next: *mut FreePage,
}

impl Display for FreePage {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(
            f,
            "FreePage {{\n\taddr: 0x{:x},\n\tstart: {:?},\n\tnext: {:?},\n}}",
            self as *const FreePage as usize,
            self.start(),
            self.next
        )
    }
}

impl FreePage {
    pub const fn new(usable: bool) -> Self {
        Self {
            usable,
            last: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    pub fn start(&self) -> *const u8 {
        self as *const Self as *const u8
    }

    pub fn end(&self) -> *const u8 {
        unsafe { self.start().offset(4095) }
    }
}

#[derive(Clone, Copy)]
pub struct FreePageIter {
    data: FreePage,
    head: FreePage,
}

impl FreePageIter {
    pub const fn new(start: FreePage) -> Self {
        Self {
            data: start,
            head: start,
        }
    }

    pub fn reset(&mut self) {
        self.data = self.head;
    }
}

impl Iterator for FreePageIter {
    type Item = FreePage;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.data.next.is_null() {
            let val = unsafe { *self.data.next };
            if val.usable {
                self.data = val;
                Some(val)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for FreePageIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.data.last.is_null() {
            let val = unsafe { *self.data.last };
            if val.usable {
                self.data = val;
                Some(val)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[repr(u32)]
#[derive(PartialEq, Clone, Copy)]
pub enum PageSize {
    Normal,   // 4KiB,
    Lvl3Huge, // 2MiB,
    Lvl2Huge, // 2GiB,
}

pub struct PageAllocator {
    head: Mutex<FreePage>,
}

impl PageAllocator {
    pub const fn new() -> Self {
        Self {
            head: Mutex::new(FreePage::new(false)),
        }
    }

    /// Must be run before allocator can be used
    /// Unsafe because I'm not sure I'm doing it right
    /// and it could kill basically everything
    pub unsafe fn init(&self, mmap: &[MemoryDescriptor]) {
        mmap.iter()
            .map(|i| i.into())
            .filter(|i: &MemoryEntry| i.kind == MemoryKind::Reclaim)
            .map(|i| {
                println!("Claiming {:#?}", i);
                for addr in (i.start..i.end).step_by(4096) {
                    self.add_free_page(addr as *mut u8);
                }
            })
            .for_each(|_| {});
        let mut head = self.head.lock();
        Self::sort_pages(&mut *head);
    }

    pub fn alloc(&self, kind: PageSize, count: usize) -> Result<(*mut u8, usize), AllocatorError> {
        let mut current = { *self.head.lock() };
        Self::sort_pages(&mut current as *mut FreePage);

        if kind == PageSize::Normal {
            let mut iter = FreePageIter::new(current);
            for page in iter {
                if let Some(found) = Self::get_contiguous(&page, count) {
                    if found.start() as usize % 4096 != 0 {
                        continue;
                    }

                    let p_alloc = PageAllocation::new(count);

                    if !page.last.is_null() {
                        let mut last = unsafe { &mut *found.last };
                        last.next = page.next;
                    }

                    let dst = page.start() as *mut PageAllocation;
                    unsafe { dst.write(p_alloc) };

                    println!("Page Allocation: {:?}", dst);

                    assert!(dst as usize % 4096 == 0);

                    return Ok((
                        unsafe { dst.add(1) as *mut u8 },
                        4096 * count - core::mem::size_of::<PageAllocation>(),
                    ));
                }
            }

            iter.reset();

            if iter.count() > 0 {
                Err(AllocatorError::NoLargeEnoughRegion)
            } else {
                Err(AllocatorError::OutOfMemory)
            }
        } else {
            Err(AllocatorError::NotImplemented)
        }
    }

    pub fn alloc_specific_address(
        &self,
        address: *const u8,
    ) -> Result<(*mut u8, usize), AllocatorError> {
        let current = { *self.head.lock() };

        let mut iter = FreePageIter::new(current);
        for page in iter {
            if page.start() == address {
                let p_alloc = PageAllocation::new(1);

                if !page.last.is_null() {
                    let mut last = unsafe { &mut *page.last };
                    last.next = page.next;
                }

                let dst = page.start() as *mut PageAllocation;
                unsafe { dst.write(p_alloc) };

                println!("Page Allocation: {:?}", dst);
                assert!(dst as usize % 4096 == 0); // This works

                return Ok((
                    unsafe { dst.add(1) as *mut u8 },
                    4096 - core::mem::size_of::<PageAllocation>(),
                ));
            }
        }

        iter.reset();

        if iter.count() > 0 {
            Err(AllocatorError::SpecifiedRegionNotFree)
        } else {
            Err(AllocatorError::OutOfMemory)
        }
    }

    pub fn display(&self) {
        let iter = FreePageIter::new(*self.head.lock());
        println!("{}", DIV);
        for page in iter {
            println!("Node: {}", page);
        }
        println!("{}", DIV);
    }

    /// Sort MUST be called beforehand
    fn get_contiguous(page: &FreePage, count: usize) -> Option<FreePage> {
        assert!(count > 0);
        let mut iter = FreePageIter::new(*page);
        iter.nth(count - 1)
    }

    fn sort_pages(head: *mut FreePage) {
        let mut current = head;
        let mut index: *mut FreePage;

        while !current.is_null() {
            index = unsafe { *current }.next;
            while !index.is_null() {
                let currentr = unsafe { &mut *current };
                let indexr = unsafe { &mut *index };
                if currentr.start() < indexr.start() {
                    core::mem::swap(&mut currentr.next, &mut indexr.next);
                }
                index = indexr.next;
            }
            current = unsafe { &*current }.next;
        }
    }

    pub fn dealloc(&self, addr: *mut u8) {
        self.add_free_page(addr);
    }

    pub fn add_free_page(&self, addr: *mut u8) {
        let mut head = self.head.lock();

        assert!(
            addr as usize % 4096 == 0
                || unsafe { (addr as *mut PageAllocation).sub(1) as usize % 4096 == 0 }
        );

        if addr as usize % 4096 == 0 {
            assert!(addr as usize % core::mem::align_of::<FreePage>() == 0);
            println!("Allocating new page: {:?}", addr);
            let mut new = FreePage::new(true);
            new.last = head.start() as *mut FreePage;
            new.next = head.next;

            let ptr = addr as *mut FreePage;
            unsafe { ptr.write(new) };

            head.next = ptr;
        } else {
            println!("Allocating previous area");
            let p_alloc = unsafe { &*((addr as *const PageAllocation).sub(1)) };

            let p_base = p_alloc.start() as *mut u8;
            assert!(p_base as usize % 4096 == 0);

            for i in (p_base as usize..unsafe { p_base.add(p_alloc.pages * 4096) as usize })
                .step_by(4096)
            {
                println!(
                    "Adding free page at {:?}, {} total",
                    i as *mut u8, p_alloc.pages
                );
                let mut new = FreePage::new(true);
                new.last = head.start() as *mut FreePage;
                new.next = head.next;

                let ptr = i as *mut FreePage;
                unsafe { ptr.write(new) };

                head.next = ptr;
            }
        }
    }
}
