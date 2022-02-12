#![no_std]
#![no_main]
#![feature(
    const_fn_trait_bound,
    panic_info_message,
    lang_items,
    asm_sym,
    naked_functions,
    asm_const,
    const_slice_from_raw_parts,
    const_mut_refs,
    associated_type_bounds,
    generic_associated_types,
    associated_type_defaults
)]
#![feature(default_alloc_error_handler)]
#![warn(missing_docs)]

//! This is the Lotus kernel

mod collections;
mod memory;
use memory::{
    allocators::{HeapAllocator, MemoryDescriptor, PageAllocator, PhysicalAllocator},
    paging::{MemoryManager, MemoryManagerImpl},
};

mod peripherals;
mod sync;
mod traits;

/// The Heap Allocator
#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::new();

/// The Physical Memory Allocator
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
static PHYSICAL_ALLOCATOR: PhysicalAllocator<PageAllocator> =
    PhysicalAllocator::new(PageAllocator::new());

/// The Virtual Memory manager
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
static MEMORY_MANAGER: MemoryManager<MemoryManagerImpl> =
    MemoryManager::new(MemoryManagerImpl::new());

use crate::{
    memory::paging::{Flags, Frame},
    peripherals::uart::{print, println},
};

use core::arch::asm;

extern crate alloc;

/// The kernel entrypoint, gets the memory descriptors then passes them to kernel main
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mmap: *mut MemoryDescriptor;
    let len: usize;
    unsafe {
        asm!(
            "mov {}, r9",
            "mov {}, r10",
            out(reg) mmap,
            out(reg) len,
        );
    }
    kentry(mmap, len)
}

/// The kernel main loop
#[no_mangle]
extern "C" fn kentry(ptr: *mut MemoryDescriptor, len: usize) -> ! {
    println!("`Println!` functioning!");

    let mmap = unsafe { core::slice::from_raw_parts(ptr, len) };
    // println!("MMAP: {:#?}", mmap);

    unsafe { PHYSICAL_ALLOCATOR.init(mmap).unwrap() };

    // PHYSICAL_ALLOCATOR.get_buddies();

    println!("Initialized page allocator");

    let heap_size = 8;
    let (heap, _heap_block) = PHYSICAL_ALLOCATOR.alloc(heap_size).unwrap();
    println!("Heap Alloc: 0x{:x}", heap as usize);

    println!("Allocated pages");

    unsafe { ALLOCATOR.init(heap, heap_size * 1024).unwrap() };
    println!("Initialized Heap Allocator");

    {
        let mut uwu = alloc::vec::Vec::new();
        let mut owo = alloc::vec::Vec::new();
        uwu.push(1);
        owo.push(1);
        for i in 1..101 {
            uwu.push(i);
            owo.push(i);
        }
    }

    println!("Well, let's try to make some mappings :v");

    unsafe { MEMORY_MANAGER.init(mmap).unwrap() }

    let (ptr, size) = PHYSICAL_ALLOCATOR.alloc(4).unwrap();
    unsafe { ptr.write(4) }
    println!("Wroet");
    let to_thingy: usize = memory::allocators::align(0xdeadbeef, 4096);
    assert!(ptr as usize % 4096 == 0 && to_thingy % 4096 == 0);
    println!("0x{:x}", to_thingy);

    println!("should fail here");

    let mut cr0: u64;
    unsafe { asm!("mov {}, cr0", out(reg) cr0) }
    let mut cr0 = memory::paging::CR0(cr0);
    println!(
        "wp {}, pg {}, 0b{:b}",
        cr0.get_write_protect(),
        cr0.get_paging(),
        cr0.0
    );
    // cr0.set_write_protect(false);
    let cr0 = memory::paging::CR0(cr0.0 ^ memory::paging::CR0::WRITE_PROTECT);
    println!(
        "wp {}, pg {}, 0b{:b}",
        cr0.get_write_protect(),
        cr0.get_paging(),
        cr0.0
    );
    unsafe { asm!("mov cr0, {}", in(reg) cr0.0) }

    let cr3: u64;
    unsafe { asm!("mov {}, cr3", out(reg) cr3) }
    // let table4 = unsafe { &mut *(cr3 as *mut memory::paging::TableLevel4) };
    // println!("{:?}", table4);
    println!("made it past cr3 0x{:x}?", cr3);
    MEMORY_MANAGER
        .map(
            Frame::with_address(ptr),
            (to_thingy as u64).into(),
            Flags::WRITABLE | Flags::PRESENT,
        )
        .unwrap();
    println!("{}", unsafe { *(to_thingy as *mut u8) });

    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
        assert!(*lock == 9)
    }
    println!("Dropped mutex!");

    println!("Beginning echo...");

    loop {
        let mut uart = crate::peripherals::UART.lock();
        let byte = uart.read_byte();
        uart.write_byte(byte);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("KERNEL PANIC");
    if let Some(reason) = info.message() {
        println!("REASON: {}", reason);
    }
    if let Some(loc) = info.location() {
        println!("Line: {}\nFile: {}", loc.line(), loc.file());
    }
    loop {
        unsafe { asm!("nop") }
    }
}
