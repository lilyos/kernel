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

mod info;
mod peripherals;
mod sync;
mod traits;

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

#[no_mangle]
#[naked]
pub extern "sysv64" fn _start() -> ! {
    unsafe {
        asm!(
            "jmp {}",
            sym kentry,
            options(noreturn),
        )
    }
}

#[no_mangle]
extern "sysv64" fn kentry() -> ! {
    println!("`Println!` functioning!");
    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
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
