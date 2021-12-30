use super::{align, MemoryEntry, MemoryKind};
use crate::sync::Mutex;
extern crate alloc;

#[derive(Debug, Clone, Copy)]
pub struct PageAllocation {
    pub start: usize,
    pub next: *mut PageAllocation,
}

impl PageAllocation {
    const PAGE_SIZE: usize = 4096;

    pub const fn new(start: usize) -> Self {
        Self {
            start,
            next: core::ptr::null_mut(),
        }
    }

    pub fn end(&self) -> usize {
        self.start + Self::PAGE_SIZE - 1
    }
}

pub struct PhysAddr(usize);
pub struct VirtAddr(usize);

#[derive(PartialEq)]
pub enum PageType {
    Normal = 4096,
    Huge = 2_097_152,
}

pub struct PageAllocator {
    head: Mutex<PageAllocation>,
}

impl PageAllocator {
    pub const fn new() -> Self {
        Self {
            head: Mutex::new(PageAllocation::new(0)),
        }
    }

    /// Must be run before allocator can be used
    /// Unsafe because I'm not sure I'm doing it right
    /// and it could kill basically everything
    pub unsafe fn init(&self, mmap: &[MemoryEntry]) {
        mmap.iter()
            .filter(|i| i.kind == MemoryKind::Reclaim)
            .map(|i| {
                for addr in (i.start..i.end).step_by(4096) {
                    self.add_free_page(PhysAddr(addr))
                }
            })
            .for_each(|_| {});
        let mut head = self.head.lock();
        Self::sort_pages(&mut *head);
    }

    pub fn allocate(&self, kind: PageType) -> Option<PhysAddr> {
        let mut current = {
            let x = self.head.lock();
            *x
        };
        Self::sort_pages(&mut current);
        while !current.next.is_null() {
            let page = unsafe { &mut *current.next };
            if kind == PageType::Normal {
                let next = page.next;
                let val = Some(PhysAddr(current.next as usize));
                current.next = next;
                return val;
            } else if kind == PageType::Huge {
                return None;
            }
        }
        None
    }

    pub fn display(&self) {
        let mut current = { &(*self.head.lock()) as *const PageAllocation };
        while !current.is_null() {
            let currentr = unsafe { &*current };
            crate::println!("Item: {:#?}", currentr);
            current = currentr.next;
        }
    }

    /// Sort MUST be called beforehand
    fn find_contiguous(page: &PageAllocation, count: usize) -> Option<&PageAllocation> {
        let mut current = page;
        let mut counter = 0;
        while !current.next.is_null() {
            counter += 1;
            if counter == count - 1 {
                return Some(page);
            }
            current = unsafe { &*current.next };
        }
        None
    }

    fn sort_pages(head: *mut PageAllocation) {
        let mut current = head;
        let mut index: *mut PageAllocation;

        if !current.is_null() {
            while !current.is_null() {
                index = unsafe { *current }.next;
                while !index.is_null() {
                    let currentr = unsafe { &mut *current };
                    let indexr = unsafe { &mut *index };
                    if currentr.start > indexr.start {
                        core::mem::swap(&mut currentr.start, &mut indexr.start);
                    }
                    index = indexr.next;
                }
                current = unsafe { &*current }.next;
            }
        }
    }

    pub fn add_free_page(&self, addr: PhysAddr) {
        assert!(addr.0 % 4096 == 0);
        let mut head = self.head.lock();
        let mut new = PageAllocation::new(addr.0);
        new.next = head.next;

        let ptr = addr.0 as *mut PageAllocation;
        unsafe { ptr.write(new) };

        head.next = addr.0 as *mut PageAllocation;
        Self::sort_pages(addr.0 as *mut PageAllocation)
    }
}
