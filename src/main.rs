#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_utils;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    
    lotus::init();

    x86_64::instructions::interrupts::int3();

    println!("Didn't crash UwU");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
