use limine_protocol::structures::MemoryMapEntry;
use log::info;

use crate::traits::{Init, Platform};

use self::{
    interrupt_manager::InterruptManager,
    memory::{addresses::RawAddress, memory_manager::MemoryManager, page_allocator::PageAllocator},
    peripherals::{SerialLogger, TimerManager, LOGGER},
    power_manager::PowerManager,
};

/// Architecture-specific structures, such as the IDT or GDT
pub mod structures;

/// Architecture-specific code relating to memory management and virtual memory
pub mod memory;

/// Peripherals
pub mod peripherals;

pub mod interrupt_manager;

mod power_manager;

pub struct X86_64<'a> {
    physical_allocator: PageAllocator<'a>,
    memory_manager: MemoryManager,
    interrupt_manager: InterruptManager,
    power_manager: PowerManager,
}

impl<'a> X86_64<'a> {
    const fn new() -> Self {
        Self {
            physical_allocator: PageAllocator::new(),
            memory_manager: MemoryManager::new(),
            interrupt_manager: InterruptManager::new(),
            power_manager: PowerManager::new(),
        }
    }
}

unsafe impl Platform for X86_64<'static> {
    type PhysicalAllocator = PageAllocator<'static>;

    type MemoryManager = MemoryManager;

    type InterruptManager = InterruptManager;

    type PowerManager = PowerManager;

    type TimerManager = TimerManager;

    type RawAddress = RawAddress;

    type Logger = SerialLogger;

    fn get_physical_allocator(&self) -> &'static Self::PhysicalAllocator {
        &self.physical_allocator
    }

    fn get_memory_manager(&self) -> &'static Self::MemoryManager {
        &self.memory_manager
    }

    fn get_interrupt_manager(&self) -> &'static Self::InterruptManager {
        &self.interrupt_manager
    }

    fn get_power_manager(&self) -> &'static Self::PowerManager {
        &self.power_manager
    }

    fn get_logger(&self) -> &'static Self::Logger {
        &LOGGER
    }
}

#[derive(Debug)]
pub enum X86_64InitError {
    PhysicalAllocator(<<X86_64<'static> as Platform>::PhysicalAllocator as Init>::Error),
    MemoryManager(<<X86_64<'static> as Platform>::MemoryManager as Init>::Error),
    InterruptManager(<<X86_64<'static> as Platform>::InterruptManager as Init>::Error),
    PowerManager(<<X86_64<'static> as Platform>::PowerManager as Init>::Error),
}

impl Init for X86_64<'static> {
    type Error = X86_64InitError;

    type Input = &'static [&'static MemoryMapEntry];

    fn init(&self, init_val: Self::Input) -> Result<(), Self::Error> {
        info!("Initializing Physical Allocator");
        if let Err(e) = self.physical_allocator.init(init_val) {
            return Err(X86_64InitError::PhysicalAllocator(e));
        }

        info!("Initializing Memory Manager");
        if let Err(e) = self.memory_manager.init(()) {
            return Err(X86_64InitError::MemoryManager(e));
        }

        info!("Initializing Interrupt Manager");
        if let Err(e) = self.interrupt_manager.init(()) {
            return Err(X86_64InitError::InterruptManager(e));
        }

        info!("Initializing Power Manager");
        if let Err(e) = self.power_manager.init(()) {
            return Err(X86_64InitError::PowerManager(e));
        }

        Ok(())
    }
}

pub static IMPLEMENTATION: X86_64 = X86_64::new();
