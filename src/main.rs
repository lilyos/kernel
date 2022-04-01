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
    ptr_metadata
)]
#![feature(default_alloc_error_handler)]
#![warn(missing_docs)]

//! This is the Lotus kernel

/// Collections used across the kernel
pub mod collections;

/// Structures relating to memory management
pub mod memory;

/// Peripheral code
pub mod peripherals;

/// Structures for task switching, the GDT, and so forth
pub mod structures;

/// Structures for atomic operations
pub mod sync;

/// Traits for various things
pub mod traits;

use memory::{
    allocators::{HeapAllocator, PageAllocator, PhysicalAllocator},
    paging::{MemoryManager, MemoryManagerImpl},
};

use stivale2::boot::{
    header::{
        Stivale2HeaderBootloaderToKernel, Stivale2HeaderFlagsBuilder,
        Stivale2HeaderKernelToBootloader,
    },
    tags::{
        headers::{AnyVideoHeader, UnmapNull, SMP},
        structures::{KernelBaseAddressStructure, MemoryMapStructure},
        BaseTag,
    },
};

mod prelude {
    pub mod rust_2021 {
        mod print_macros {
            #[macro_export]
            /// The print macro
            macro_rules! print {
                ($($arg:tt)*) => (
                    {
                        use core::fmt::Write;
                        let mut uart = crate::peripherals::UART.lock();
                        uart.write_fmt(format_args!($($arg)*)).unwrap();
                    }
                );
            }

            #[macro_export]
            /// The println macro, literally just the print macro + a line return
            macro_rules! println {
                () => (crate::print!("\n"));
                ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
            }
        }
        pub use crate::{print, println};
        pub use core::arch::asm;
        pub use core::prelude::rust_2021::*;
        pub use core::prelude::v1::*;
    }
}

#[prelude_import]
pub use prelude::rust_2021::*;

static STACK: [u8; 8192] = [0; 8192];

#[used]
#[no_mangle]
#[link_section = ".stivale2hdr"]
static HEADER: Stivale2HeaderKernelToBootloader = Stivale2HeaderKernelToBootloader {
    entry_point: 0,
    stack: &STACK[8191],
    flags: Stivale2HeaderFlagsBuilder::new()
        .protected_memory_regions(true)
        .upgrade_higher_half(true)
        .virtual_kernel_mappings(true)
        .finish(),
    tags: &ANY_VIDEO as *const AnyVideoHeader as *const (),
};

static ANY_VIDEO: AnyVideoHeader = AnyVideoHeader {
    identifier: AnyVideoHeader::IDENTIFIER,
    next: &UNMAP_NULL as *const UnmapNull as *const (),
    preference: 0,
};

static UNMAP_NULL: UnmapNull = UnmapNull {
    identifier: UnmapNull::IDENTIFIER,
    next: &SMP as *const SMP as *const (),
};

static SMP: SMP = SMP {
    identifier: SMP::IDENTIFIER,
    next: core::ptr::null(),
    flags: 1,
};

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

/// The kernel's GDT
static mut GDT: [SegmentDescriptor; 9] = [SegmentDescriptor::new_unused(); 9];

use crate::structures::{
    GlobalDescriptorTable, SaveGlobalDescriptorTableResult, SegmentDescriptor, TaskStateSegment,
};

use core::arch::asm;

extern crate alloc;

/// The kernel entrypoint, gets the memory descriptors then passes them to kernel main
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut ptr: *const Stivale2HeaderBootloaderToKernel;
    unsafe {
        asm!("mov {}, rdi", out(reg) ptr);
    }
    let hdr = unsafe { &*ptr };
    kentry(hdr)
}

/// The kernel main loop
#[no_mangle]
fn kentry(header: &Stivale2HeaderBootloaderToKernel) -> ! {
    println!("`Println!` functioning!");
    println!("Bootloader header: {:?}", HEADER);

    let mut mmap: Option<&MemoryMapStructure> = None;
    let mut addrs: Option<&KernelBaseAddressStructure> = None;

    let mut ptr = header.tags;

    while !ptr.is_null() {
        if let Ok(val) = MemoryMapStructure::try_from_base(ptr) {
            mmap = Some(unsafe { &*val });
        }

        if let Ok(val) = KernelBaseAddressStructure::try_from(ptr) {
            addrs = Some(val);
        }

        ptr = unsafe { (*ptr).next as *const BaseTag };
    }

    println!("This should be some: {}", mmap.is_some());

    unsafe { PHYSICAL_ALLOCATOR.init(mmap.unwrap()).unwrap() };

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

    assert!(MEMORY_MANAGER
        .virtual_to_physical(core::ptr::null::<u8>().try_into().unwrap())
        .is_none());

    let x = 9u64;
    let x_ptr = &x as *const u64;
    let x_ptr_phys =
        (addrs.unwrap().phys_base + (x_ptr as u64 - addrs.unwrap().virt_base)) as *const u64;
    println!("X-PTR: {:?}, X-PTR-PHYS: {:?}", x_ptr, x_ptr_phys);
    assert!(unsafe { *x_ptr } == x);

    let x_got_ptr =
        MEMORY_MANAGER.virtual_to_physical((x_ptr_phys as *const u8).try_into().unwrap());
    println!("Got: {:?}", x_got_ptr);

    assert!(x_ptr_phys as usize == x_got_ptr.unwrap().get_address());

    println!("that was equal, so my virt to phys works ig :v");

    unsafe { MEMORY_MANAGER.init(mmap.unwrap()).unwrap() }

    let mutex = sync::Mutex::new(9);
    {
        println!("Locking mutex!");
        let lock = mutex.lock();
        println!("Locked mutex! Got value {}", *lock);
        assert!(*lock == 9)
    }
    println!("Dropped mutex!");

    let esp: u64;
    unsafe {
        asm!("mov {}, rsp", out(reg) esp);
    }

    let _tss = TaskStateSegment::new_no_ports(
        (esp as *mut u8).try_into().unwrap(),
        core::ptr::null_mut::<u8>().try_into().unwrap(),
        core::ptr::null_mut::<u8>().try_into().unwrap(),
    );

    unsafe {
        let mut kcode = SegmentDescriptor::new_unused();
        kcode.set_limit(0xFFFFF);
        {
            kcode.access_byte.set_present(true);
            kcode.access_byte.set_is_code_or_data(true);
        }
        let kcab = &mut kcode.access_byte.code;
        kcab.set_executable();
        kcab.set_read_write();

        GDT[1] = kcode;

        let mut kdata = SegmentDescriptor::new_unused();
        kdata.set_limit(0xFFFFF);
        {
            kdata.access_byte.set_present(true);
            kdata.access_byte.set_is_code_or_data(true);
        }
        let kdab = &mut kdata.access_byte.code;
        kdab.set_read_write();
        kdab.set_present();

        GDT[2] = kdata;

        let mut ucode32 = SegmentDescriptor::new_unused();
        ucode32.set_limit(0xFFFFF);
        {
            ucode32.access_byte.set_present(true);
            ucode32.access_byte.set_is_code_or_data(true);
        }
        let ucab32 = &mut ucode32.access_byte.code;
        ucab32.set_read_write();
        ucab32.set_present();
        ucab32.set_descriptor_level(3);

        GDT[3] = ucode32;

        let mut udata32 = SegmentDescriptor::new_unused();
        udata32.set_limit(0xFFFFF);
        {
            udata32.access_byte.set_present(true);
            udata32.access_byte.set_is_code_or_data(true);
        }
        let udab32 = &mut udata32.access_byte.code;
        udab32.set_read_write();
        udab32.set_present();
        udab32.set_descriptor_level(3);

        GDT[4] = udata32;

        let mut ucode64 = SegmentDescriptor::new_unused();
        ucode64.set_flags(0b0010);
        ucode64.set_limit(0xFFFFF);
        {
            ucode64.access_byte.set_present(true);
            ucode64.access_byte.set_is_code_or_data(true);
        }
        let ucab64 = &mut ucode64.access_byte.code;
        ucab64.set_read_write();
        ucab64.set_present();
        ucab64.set_descriptor_level(3);

        GDT[5] = ucode64;

        let mut udata64 = SegmentDescriptor::new_unused();
        udata64.set_limit(0xFFFFF);
        {
            udata32.access_byte.set_present(true);
            udata32.access_byte.set_is_code_or_data(true);
        }
        let udab64 = &mut udata64.access_byte.code;
        udab64.set_read_write();
        udab64.set_present();
        udab64.set_descriptor_level(3);

        GDT[6] = udata64;
    }

    println!("Loading GDT");

    let gdt_ldr = SaveGlobalDescriptorTableResult {
        limit: { 7 * 8 } - 1,
        base: unsafe { GDT.as_ptr() as usize as u64 },
    };

    println!(
        "Is address canonical: {}",
        memory::allocators::is_address_canonical(0x0000ffffffffb541, 64)
    );

    println!("Made object: {:#?}", gdt_ldr);

    GlobalDescriptorTable::apply(gdt_ldr);

    let gdt_res = SaveGlobalDescriptorTableResult::get();
    println!("{:#?}", gdt_res);

    let gdt = GlobalDescriptorTable::from_existing(gdt_res);

    println!("GDT entries len: {}", gdt.entries.len());
    println!("{:#?}", gdt);

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
        let mut i: u64 = 0;
        (0..u64::MAX).for_each(|_| {
            i += 1;
        });
        unsafe {
            asm!("int 0x00");
        }
    }
}
