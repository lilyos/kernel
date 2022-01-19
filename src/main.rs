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
    const_mut_refs
)]
#![feature(default_alloc_error_handler)]

mod allocator;
mod peripherals;
mod sync;
mod traits;

#[global_allocator]
static ALLOCATOR: allocator::HeapAllocator = allocator::HeapAllocator::new();

static PAGE_ALLOCATOR: allocator::PageAllocator = allocator::PageAllocator::new();

static MEMORY_MANAGER: allocator::MemoryManager = allocator::MemoryManager::new();

use crate::peripherals::uart::{print, println};

use core::arch::asm;

extern crate alloc;

#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!(
            "jmp {}",
            sym _start2,
            options(noreturn),
        )
    }
}

extern "C" fn _start2() -> ! {
    let mmap: *mut allocator::MemoryDescriptor;
    let len: usize;
    unsafe {
        asm!(
            "mov {}, r9",
            "mov {}, r10",
            out(reg) mmap,
            out(reg) len,
        )
    }
    kentry(mmap, len)
}

#[no_mangle]
extern "C" fn kentry(ptr: *mut allocator::MemoryDescriptor, len: usize) -> ! {
    println!("`Println!` functioning!");

    let mmap = unsafe { core::slice::from_raw_parts(ptr, len) };
    // println!("MMAP: {:#?}", mmap);

    unsafe { PAGE_ALLOCATOR.init(&mmap[0..3]) };

    println!("Initialized page allocator");

    let (heap, heap_size) = PAGE_ALLOCATOR
        .alloc(allocator::PageSize::Normal, 2)
        .unwrap();

    let (heap_s, heap_s_size) = PAGE_ALLOCATOR
        .alloc(allocator::PageSize::Normal, 1)
        .unwrap();

    let (test, _) = PAGE_ALLOCATOR
        .alloc(allocator::PageSize::Normal, 1)
        .unwrap();

    println!(
        "Allocated test page: {:?}, actual address: {:?}",
        test,
        unsafe { test.sub(core::mem::size_of::<usize>()) }
    );

    PAGE_ALLOCATOR.dealloc(test);

    println!("Deallocated test page: {:?}", test);

    let (test2, _) = PAGE_ALLOCATOR.alloc_specific_address(test).unwrap();
    println!("(Tried to) Allocate specific address: {:?}", test2);
    assert!(test == test2);
    println!("Allocating specific address worked");
    PAGE_ALLOCATOR.dealloc(test2);

    println!("Allocated pages");

    // MEMORY_MANAGER.uwu();
    unsafe {
        ALLOCATOR
            .init(heap, heap_size, heap_s, heap_s_size)
            .unwrap()
    };
    println!("Initialized Heap Allocator");
    ALLOCATOR.display();
    println!("Finished display");

    {
        let mut uwu = alloc::vec::Vec::new();
        let mut owo = alloc::vec::Vec::new();
        uwu.push(1);
        owo.push(1);
        println!("Pushed 1 to vec\n{:#?}\n{:#?}", uwu, owo);
        println!("Pushing a lot");
        for i in 1..101 {
            uwu.push(i);
            owo.push(i);
            println!("Pushed {}", i);
        }
        println!("Dropping");
    }

    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
        assert!(*lock == 9)
    }
    println!("Dropped mutex!");

    // println!("uh {:#?}", mmap);

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
