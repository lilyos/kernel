#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    lang_items,
    asm_sym,
    naked_functions,
    asm_const,
    const_mut_refs,
    associated_type_bounds,
    generic_associated_types,
    associated_type_defaults,
    prelude_import,
    ptr_metadata,
    abi_x86_interrupt,
    stmt_expr_attributes,
    const_maybe_uninit_zeroed,
    const_for,
    const_trait_impl,
    const_convert,
    allocator_api,
    vec_into_raw_parts,
    const_result_drop
)]
#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    unused_features,
    missing_docs
)]
#![feature(default_alloc_error_handler)]

//! This is the Lotus kernel

// BEGIN: Modules, Macros, and Prelude

// Self-explanitory
extern "C" {
    static __KERNEL_START: *mut ();
    static __RODATA_START: *mut ();
    static __RODATA_END: *mut ();
    static __TEXT_START: *mut ();
    static __TEXT_END: *mut ();
    static __DATA_START: *mut ();
    static __DATA_END: *mut ();
    static __BSS_START: *mut ();
    static __BSS_END: *mut ();
    static __MISC_START: *mut ();
    static __MISC_END: *mut ();
    static __KERNEL_END: *mut ();
}

/// Architecture-specific structures
pub mod arch;

/// Collections used across the kernel
pub mod collections;

/// Error types
pub mod errors;

/// Interrupt handling
pub mod interrupts;

/// The Platform Logger
pub mod platform_logger;

/// Macros
mod macros;

/// Structures relating to memory management
pub mod memory;

/// Structures for atomic operations
pub mod sync;

/// Traits for various things
pub mod traits;

/// Multi-core related objects
pub mod smp;

mod prelude {
    pub mod rust_2021 {
        mod print_macros {
            #[macro_export]
            /// The print macro
            macro_rules! print {
                ($($arg:tt)*) => (
                    {
                        use $crate::{traits::Platform, arch::PLATFORM_MANAGER};
                        use core::fmt::Write;
                        PLATFORM_MANAGER.get_text_output().write_fmt(format_args!($($arg)*)).unwrap();
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

// END: Modules, Macros, and Prelude
// Actual code begins here

use log::{debug, error, info, trace};

use memory::allocators::{NeverAllocator, PageAllocator};

use smp::CoreManager;
use sync::lazy_static;
use traits::Platform;

use crate::{arch::PLATFORM_MANAGER, traits::Init};

use limine_protocol::{
    HHDMRequest, KernelAddressRequest, MemoryMapRequest, Request, SMPRequest, StackSizeRequest,
};

static MEMORY_MAP: Request<MemoryMapRequest> = MemoryMapRequest::default().into();

static KERNEL_ADDRESS: Request<KernelAddressRequest> = KernelAddressRequest::default().into();

static STACK_REQUEST: Request<StackSizeRequest> = StackSizeRequest {
    stack_size: 1024 * 64,
    ..StackSizeRequest::default()
}
.into();

static SMP_REQUEST: Request<SMPRequest> = SMPRequest {
    flags: 1,
    ..SMPRequest::default()
}
.into();

static HHDM_REQUEST: Request<HHDMRequest> = HHDMRequest::default().into();

lazy_static! {
    /// The Range for the Higher Half Direct Map
    pub lazy static HHDM_RANGE: core::ops::Range<usize> = {
        use crate::memory::utilities::align;
        unsafe {
            let offset = crate::HHDM_REQUEST.response.unwrap().as_ref().offset as usize;
            offset
                ..align(
                    offset
                        + crate::MEMORY_MAP.response
                            .expect("Memory Map didn't receive a response")
                            .as_ref()
                            .get_memory_map()
                            .expect("Memory Map pointer was invalid")
                            .iter()
                            .last()
                            .expect("No item was last")
                            .end() as usize,
                    4096,
                )
        }
    };

    /// The Safe range to use for upper half mapping
    pub lazy static SAFE_UPPER_HALF_RANGE: core::ops::Range<usize> = {
        HHDM_RANGE.end..(2 ^ 48) - 1
    };
}

/// This shouldn't be used, it immediately errors out. This is intentional
#[global_allocator]
static ALLOCATOR: NeverAllocator = NeverAllocator;

/// The Physical Allocator
static PHYSICAL_ALLOCATOR: PageAllocator = PageAllocator::new();

/// Get the Memory Manager
pub fn get_memory_manager() -> &'static <arch::PlatformType as Platform>::MemoryManager {
    PLATFORM_MANAGER.get_memory_manager()
}

static CORE_MANAGER: CoreManager = CoreManager::new();

/// The kernel entrypoint
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kentry()
}

/// The kernel main loop
fn kentry() -> ! {
    platform_logger::LOGGER.init().unwrap();
    info!("Logging enabled");
    trace!("{:?}", *MEMORY_MAP);

    assert_eq!(core::mem::size_of::<usize>(), core::mem::size_of::<<<crate::arch::PlatformType as crate::traits::Platform>::RawAddress as crate::traits::PlatformAddress>::UnderlyingType>());

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

    let smp = unsafe { SMP_REQUEST.response.unwrap().as_ref() };

    debug!("SMP Response: {:#?}", smp);

    PHYSICAL_ALLOCATOR.init(mmap).unwrap();

    info!("Initialized page allocator");

    CORE_MANAGER
        .init(smp.cpu_count.try_into().unwrap())
        .unwrap();

    info!("Initialized Core Manager");

    PLATFORM_MANAGER.init(()).unwrap();

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

    /*
    // let rsp = RSP::get();
    let rsp = RSP(core::ptr::null_mut());

    trace!("Got rsp");

    let tss = TaskStateSegment::new_no_ports(
        (rsp.0).try_into().unwrap(),
        core::ptr::null_mut::<u8>().try_into().unwrap(),
        core::ptr::null_mut::<u8>().try_into().unwrap(),
    );

    let mut sys_descriptor = SystemSegmentDescriptorLongMode::new_unused();
    sys_descriptor.set_flags(0x40);
    sys_descriptor.set_base(&tss as *const _ as u64);
    sys_descriptor.set_limit(core::mem::size_of_val(&tss) as u32);
    sys_descriptor.access_byte = SystemSegmentAccessByte::PRESENT
        .descriptor_privilege_level(0)
        .segment_type(SegmentType {
            long: SegmentType64Bit::Tss64BitAvailable,
        });

    trace!("Made tss and system descriptor");

    let as_segd = unsafe { &*(arch::structures::GDT.as_ptr() as *const [SegmentDescriptor; 9]) };
    debug!("{:#?}", as_segd);

    /*

    unsafe {
        (arch::structures::GDT.as_ptr().offset(7) as *mut SystemSegmentDescriptorLongMode)
            .write(sys_descriptor);
    }

    */

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
    */
    let mut num = 0u64;
    loop {
        if num % 13 == 0 {
            debug!("Heartbeat <3");
        }
        if num == u64::MAX {
            num = 0;
        } else {
            num += 1;
        }
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
