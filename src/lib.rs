#![no_std]
#![feature(asm, const_fn_trait_bound, panic_info_message)]

mod info;
mod peripherals;
mod sync;
mod traits;

use crate::peripherals::{Channel, MailboxMessage, Tags};

use core::fmt::Write;

macro_rules! print {
    ($($arg:tt)*) => (unsafe {
        //peripherals::PERIPHERALS.uart.write_fmt(format_args!($($arg)*)).unwrap();
        (&mut *crate::peripherals::UART).write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[no_mangle]
pub extern "C" fn kentry() -> ! {
    println!("`Println!` functioning!");
    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
    }
    println!("Dropped mutex!");

    let mut message_fnl = MailboxMessage::new_exact(Channel::Property, Tags::SerialNumber, 8);

    let mailbox = unsafe { &mut *peripherals::MAILBOX };

    println!("Getting serial number...");
    let res = mailbox.send(&mut message_fnl);

    if res {
        println!(
            "Serial Number: {:x}{:x}",
            message_fnl.0[6], message_fnl.0[5]
        );
        println!("Array: {:?}", message_fnl.0);
    } else {
        println!("Didn't get the serial number!");
    }

    let mut message_fnl_2 = MailboxMessage::new_exact(Channel::Property, Tags::MACAddress, 6);

    let res = mailbox.send(&mut message_fnl_2);

    if res {
        let bytes1 = message_fnl_2.0[6].to_be_bytes();
        let bytes2 = message_fnl_2.0[5].to_be_bytes();
        println!(
            "MAC Address {:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            bytes1[0], bytes1[1], bytes1[2], bytes1[3], bytes2[0], bytes2[1]
        );
        println!("{:?}{:?}", bytes1, bytes2);
    } else {
        println!("Didn't get MAC Address!");
    }

    println!(
        "Current exception level: {}",
        peripherals::cpu::get_exception_level()
    );

    println!("Beginning echo...");
    loop {
        let uart = unsafe { &mut *peripherals::UART };
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
    loop {}
}
