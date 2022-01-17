use crate::{
    allocator::{PhysAddr, VirtAddr},
    println,
};

use kernel_macros::bit_field_accessors;

use core::arch::asm;

#[derive(Debug)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
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
        is_reserved 9
        no_execute 63
    }

    fn get_address(&self) -> u64 {
        let x = self.0 & 0b0000000000001111111111111111111111111111111111111111000000000000;
        assert!(x < 2u64.pow(52));
        x
    }
}

#[derive(Debug)]
#[repr(align(4096), C)]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

pub struct MemoryManager {}

impl MemoryManager {
    pub const fn new() -> Self {
        Self {}
    }

    pub unsafe fn init(&self) {}

    pub fn uwu(&self) {
        let x: u64;
        unsafe { asm!("mov {}, cr3", out(reg) x) }
        let x = x as *const PageTable;
        let mut table = unsafe { &*x };
        for _ in (1..=3).rev() {
            table = unsafe { &*(table.entries[0].get_address() as *const PageTable) };
        }
        println!(
            "{:#?}\nPhysical address: 0x{:x}",
            table,
            table.entries[0].get_address()
        );
        println!("Done")
    }

    pub fn virt_to_phys(&self, _addr: VirtAddr) -> Option<usize> {
        // TODO: Setup Virtual to Physical translation
        None
    }

    /// Source and destination must be page-aligned
    pub fn map(&self, _src: PhysAddr, _dest: VirtAddr, _flags: u64) -> Result<(), ()> {
        Err(())
    }

    /// This will fail if the address isn't mapped to any page
    pub fn unmap(&self, _addr: VirtAddr) -> Result<(), ()> {
        Err(())
    }
}
