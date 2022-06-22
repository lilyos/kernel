#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    lang_items,
    asm_sym,
    naked_functions,
    asm_const,
    const_slice_from_raw_parts,
    const_mut_refs,
    associated_type_bounds,
    generic_associated_types,
    associated_type_defaults,
    prelude_import,
    ptr_metadata,
    abi_x86_interrupt,
    stmt_expr_attributes
)]
#![feature(default_alloc_error_handler)]
#![warn(missing_docs)]

//! This is the Lotus kernel

/// Collections used across the kernel
pub mod collections;

/// Structures relating to memory management
pub mod memory;

/// Architecture-specific structures
pub mod arch;

/// Structures for atomic operations
pub mod sync;

/// Traits for various things
pub mod traits;

/// Interrupt handling
pub mod interrupts;

/// The logger
pub mod logger;

/// Macros
mod macros;

use crate::{
    arch::{
        peripherals::cpu::RSP,
        structures::{install_interrupt_handler, SystemSegmentDescriptor},
        MEMORY_MANAGER, PHYSICAL_ALLOCATOR,
    },
    interrupts::InterruptType,
    traits::{PhysicalMemoryAllocator, VirtualMemoryManager},
};

use log::{debug, error, info, trace};
use memory::allocators::HeapAllocator;

use limine_protocol::{
    requests::{KernelAddressRequest, MemoryMapRequest, SMPRequest, StackSizeRequest},
    LimineRequest,
};

use crate::arch::structures::{GlobalDescriptorTable, SizedDescriptorTable, TaskStateSegment};

mod prelude {
    pub mod rust_2021 {
        mod print_macros {
            #[macro_export]
            /// The print macro
            macro_rules! print {
                ($($arg:tt)*) => (
                    {
                        use core::fmt::Write;
                        let mut uart = $crate::arch::peripherals::UART.lock();
                        uart.write_fmt(format_args!($($arg)*)).unwrap();
                    }
                );
            }

            #[macro_export]
            /// The println macro, literally just the print macro + a line return
            macro_rules! println {
                () => ($crate::print!("\n"));
                ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
            }
        }
        pub use crate::{print, println};
        pub use core::arch::asm;
        pub use core::prelude::rust_2021::*;
        pub use core::prelude::v1::*;

        extern crate alloc;
        pub use alloc::{
            borrow::*, boxed::Box, collections::*, slice::*, str::*, string::*, sync::*, task::*,
            vec::*,
        };
    }
}

#[prelude_import]
pub use prelude::rust_2021::*;

#[used]
static MEMORY_MAP: LimineRequest<MemoryMapRequest> = MemoryMapRequest {
    id: MemoryMapRequest::ID,
    revision: 0,
    response: None,
}
.into_request();

#[used]
static KERNEL_ADDRESS: LimineRequest<KernelAddressRequest> = KernelAddressRequest {
    id: KernelAddressRequest::ID,
    revision: 0,
    response: None,
}
.into_request();

#[used]
static STACK_REQUEST: LimineRequest<StackSizeRequest> = StackSizeRequest {
    stack_size: 1024 * 64,
    id: StackSizeRequest::ID,
    revision: 0,
    response: None,
}
.into_request();

#[used]
static SMP_REQUEST: LimineRequest<SMPRequest> = SMPRequest {
    flags: 1,
    id: SMPRequest::ID,
    revision: 0,
    response: None,
}
.into_request();

/// The Heap Allocator
#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::new();

extern crate alloc;

/// The kernel entrypoint
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kentry()
}

/// The kernel main loop
#[no_mangle]
fn kentry() -> ! {
    crate::logger::LOGGER.init();
    info!("Logging enabled");
    trace!("{:?}", *MEMORY_MAP);

    let mmap_res = unsafe {
        MEMORY_MAP
            .response
            .expect("The memory map wasn't present")
            .as_ref()
    };

    let mmap = unsafe {
        mmap_res
            .get_memory_map()
            .expect("The memory map wasn't present")
    };

    debug!("Memory Map: {:#?}", mmap);

    let addrs = unsafe {
        KERNEL_ADDRESS
            .response
            .expect("The kernel address struct wasn't present")
            .as_ref()
    };

    debug!("Kernel Addresses: {:#?}", addrs);

    unsafe { PHYSICAL_ALLOCATOR.init(mmap).unwrap() };

    info!("Initialized page allocator");

    const INITIAL_HEAP_SIZE: usize = 8;
    let heap_alloc = PHYSICAL_ALLOCATOR.alloc(INITIAL_HEAP_SIZE).unwrap();
    info!(
        "Allocated Heap space starting at {:#X}",
        heap_alloc.address_mut() as usize
    );

    unsafe {
        ALLOCATOR
            .init(heap_alloc.address_mut(), INITIAL_HEAP_SIZE * 1024)
            .unwrap()
    };
    info!("Initialized Heap Allocator");

    {
        let mut uwu = Vec::new();
        let mut owo = Vec::new();
        uwu.push(1);
        owo.push(1);
        for i in 1..101 {
            uwu.push(i);
            owo.push(i);
        }
    }

    assert!(MEMORY_MANAGER
        .virtual_to_physical(core::ptr::null::<u8>().try_into().unwrap())
        .is_none());

    let x = 9u64;
    let x_ptr = &x as *const u64;
    debug!("X-PTR: {:?}", x_ptr);

    let x_got_ptr = MEMORY_MANAGER.virtual_to_physical((x_ptr as *const u8).try_into().unwrap());
    debug!("Got: {:?}", x_got_ptr);

    unsafe { MEMORY_MANAGER.init(mmap).unwrap() }

    {
        let mutex = sync::Mutex::new(9);
        {
            debug!("Locking mutex!");
            let lock = mutex.lock();
            debug!("Locked mutex! Got value {}", *lock);
            assert!(*lock == 9)
        }
        debug!("Dropped mutex!");
    }

    // let rsp = RSP::get();
    let rsp = RSP(core::ptr::null_mut());

    trace!("Got rsp");

    let tss = TaskStateSegment::new_no_ports(
        (rsp.0).try_into().unwrap(),
        core::ptr::null_mut::<u8>().try_into().unwrap(),
        core::ptr::null_mut::<u8>().try_into().unwrap(),
    );

    let mut sys_descriptor = SystemSegmentDescriptor::new_unused();
    sys_descriptor.set_flags(0x40);
    sys_descriptor.set_base(&tss as *const _ as u64);
    sys_descriptor.set_limit(core::mem::size_of_val(&tss) as u32);
    sys_descriptor.access_byte = arch::structures::AccessByte { raw: 0x40 };

    trace!("Made tss and system descriptor");

    unsafe {
        (arch::structures::GDT.as_ptr().offset(7) as *mut SystemSegmentDescriptor)
            .write(sys_descriptor);
    }

    info!("Loading GDT");

    let gdt_ldr = SizedDescriptorTable {
        limit: { 7 * 8 } - 1,
        base: unsafe { arch::structures::GDT.as_ptr() as usize as u64 },
    };

    let gdt_ldr_ptr = &gdt_ldr as *const _ as usize;

    debug!(
        "Is address canonical: {}",
        memory::utilities::is_address_canonical(gdt_ldr.base as usize, 48)
    );

    debug!("Made object: {:#?}", gdt_ldr);

    GlobalDescriptorTable::apply(gdt_ldr_ptr);

    let gdt_res = SizedDescriptorTable::get_gdt();
    debug!("{:#?}", gdt_res);

    // debug!("{:#?}", gdt);

    unsafe { install_interrupt_handler() };

    fn handler(it: InterruptType) {
        error!("We got an interrupt! {it:?}");
        loop {
            unsafe {
                asm!("pause");
            }
        }
    }

    unsafe { arch::structures::INTERRUPT_HANDLER = Some(handler) }

    debug!("We installed the handler?");

    unsafe fn unsafe_divide(a: u64, b: u64) -> (u64, u64) {
        let mut out = a;
        let mut rem = 0;
        asm!(
            "cdqe",
            "div {}",
            in(reg) b,
            inout("rax") out,
            inout("rdx") rem,
        );
        (out, rem)
    }

    trace!("`unsafe_divide` address: {:#X}", unsafe_divide as usize);

    let a = unsafe { unsafe_divide(4, 2) }.0;
    assert_eq!(a, 2);
    let b = unsafe { unsafe_divide(4, 0) }.0;
    assert_eq!(b, 0);

    info!("Beginning echo...");
    loop {
        let mut uart = crate::arch::peripherals::UART.lock();
        let byte = uart.read_byte();
        uart.write_byte(byte);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("KERNEL PANIC");
    if let Some(reason) = info.message() {
        error!("REASON: {}", reason);
    }
    if let Some(loc) = info.location() {
        error!("IN: {}:{}", loc.file(), loc.line());
    }
    loop {
        unsafe { asm!("pause") }
    }
}
