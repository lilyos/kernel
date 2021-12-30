#![no_std]
#![no_main]
#![feature(
    const_fn_trait_bound,
    panic_info_message,
    lang_items,
    asm_sym,
    naked_functions,
    asm_const
)]
#![feature(default_alloc_error_handler)]

mod allocator;
mod peripherals;
mod sync;
mod traits;

#[global_allocator]
static ALLOCATOR: allocator::HeapAllocator = allocator::HeapAllocator::new();

static PAGE_ALLOCATOR: allocator::PageAllocator = allocator::PageAllocator::new();

use crate::peripherals::uart::{print, println};

use core::arch::asm;

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
    let mmap: *mut allocator::MemoryEntry;
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
extern "C" fn kentry(ptr: *mut allocator::MemoryEntry, len: usize) -> ! {
    let mmap = unsafe { core::slice::from_raw_parts(ptr, len) };
    unsafe { PAGE_ALLOCATOR.init(mmap) };

    PAGE_ALLOCATOR.display();

    println!("`Println!` functioning!");
    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
        assert!(*lock == 9)
    }
    println!("Dropped mutex!");

    println!("uh {:#?}", mmap);

    let x: *mut u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) x);
    }
    println!(":v {:#?}", x);

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
