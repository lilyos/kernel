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

use core::{arch::asm, fmt::Write};

macro_rules! print {
    ($($arg:tt)*) => (
        {
            let mut uart = crate::peripherals::UART.lock();
            uart.write_fmt(format_args!($($arg)*)).unwrap();
        }
    );
}

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

static mut MMAP: *mut allocator::MemoryEntry = core::ptr::null_mut();
static mut MMAP_LEN: usize = 0;

#[no_mangle]
#[naked]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!(
            "jmp {}",
            sym kentry,
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

}

#[no_mangle]
extern "C" fn kentry() -> ! {
    println!("`Println!` functioning!");
    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
        assert!(*lock == 9)
    }
    println!("Dropped mutex!");

    let mmap = unsafe { core::slice::from_raw_parts(MMAP, MMAP_LEN) };
    println!("uh {:#?}", mmap);

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
