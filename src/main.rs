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
mod structures;
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
    memory::paging::{PageAlignedAddress, CR0},
    peripherals::uart::{print, println},
    structures::{GlobalDescriptorTable, SaveGlobalDescriptorTableResult, TaskStateSegment},
};

use core::arch::asm;

extern crate alloc;

/// The kernel entrypoint, gets the memory descriptors then passes them to kernel main
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mmap_ptr: *mut MemoryDescriptor;
    let len: usize;
    unsafe {
        asm!(
            "mov {}, r9",
            "mov {}, r10",
            out(reg) mmap_ptr,
            out(reg) len,
        );
    }
    let mmap = unsafe { core::slice::from_raw_parts(mmap_ptr, len) };
    kentry(mmap)
}

/// The kernel main loop
#[no_mangle]
fn kentry(mmap: &[MemoryDescriptor]) -> ! {
    println!("`Println!` functioning!");

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
    let mut cr0 = CR0::get();
    cr0.clear_write_protect();
    cr0.update();

    let cr3: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3);
    }
    println!("CR3: 0x{:x}", cr3);

    let x = 9u64;
    let x_ptr = &x as *const u64;
    assert!(unsafe { *x_ptr } == x);

    let x_got_ptr = MEMORY_MANAGER.virtual_to_physical(x_ptr as *mut u8);

    println!(
        "The actual pointer 0x{:x}, what we got: {:?}",
        x_ptr as usize, x_got_ptr
    );

    assert!(x_ptr == x_got_ptr.unwrap() as *const u64);

    println!("that was equal, so my virt to phys works ig :v");

    unsafe { MEMORY_MANAGER.init(mmap).unwrap() }

    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
        assert!(*lock == 9)
    }
    println!("Dropped mutex!");

    println!("Beginning echo...");

    let esp: u64;
    unsafe {
        asm!("mov {}, rsp", out(reg) esp);
    }
    let tss = TaskStateSegment::new_no_ports(
        esp as *mut u8,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let gdt_res = SaveGlobalDescriptorTableResult::get();

    let gdt = GlobalDescriptorTable::from_existing(gdt_res);

    println!("{:#?}", gdt);

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
        let mut i: u64 = 0;
        (0..u64::MAX).for_each(|_| {
            i += 1;
        });
        unsafe {
            asm!("int 0x00");
        }
    }
}
